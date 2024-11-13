#![allow(dead_code)]
use axum::{http::StatusCode, routing::post, Router};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

fn port_is_available_on_localhost(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn find_available_port(first: Option<u16>) -> Option<u16> {
    (first.unwrap_or(49152)..65535).find(|port| port_is_available_on_localhost(*port))
}

pub(crate) struct AppState {
    pub findings: Mutex<Vec<String>>,
    pub running_state: Mutex<u32>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            findings: Mutex::new(Vec::<String>::new()),
            running_state: Mutex::new(0),
        }
    }
}

async fn vuln_handler(state: Arc<AppState>, body: String) {
    state.findings.lock().unwrap().push(body);
}

async fn print_handler(body: String) {
    println!("/print: {body}");
}

async fn test_handler2(body: String) -> Result<(), (StatusCode, String)> {
    println!("Got a call to /vuln: {body:?}");
    Result::<(), (StatusCode, String)>::Ok(())
}

async fn wait_for_termination(state: Arc<AppState>) {
    let running = || *state.running_state.lock().unwrap() < 2;
    while running() {
        std::thread::sleep(Duration::from_millis(100));
    }
}

async fn graceful_shutdown(state: Arc<AppState>) {
    tokio::select! {
        _ = async{
            wait_for_termination(state).await
        } => {},
    }
}

#[tokio::main]
async fn server_thread(state: Arc<AppState>) {
    let mut first: Option<u16> = None;
    loop {
        let port = find_available_port(first);
        if port.is_none() {
            return;
        }
        let port = port.unwrap();
        std::env::set_var("SCOUT_PORT_NUMBER", port.to_string());
        // build our application with a route
        let app = Router::new()
            .route(
                "/vuln",
                post({
                    let state2 = state.clone();
                    move |body| vuln_handler(state2, body)
                }),
            )
            .route("/print", post(print_handler))
            .route("/vuln2", post(test_handler2));

        let address = "127.0.0.1:".to_string() + port.to_string().as_str();

        // run it
        let result = tokio::net::TcpListener::bind(address).await;
        if result.is_err() {
            first = Some(port + 1);
            continue;
        }
        let listener = result.unwrap();

        let future =
            axum::serve(listener, app).with_graceful_shutdown(graceful_shutdown(state.clone()));

        *state.running_state.lock().unwrap() = 1;

        future.await.unwrap();
        break;
    }
}

fn start_server(state: Arc<AppState>) -> std::thread::JoinHandle<()> {
    let state2 = state.clone();
    let ret = std::thread::spawn(|| server_thread(state2));
    let not_running = || *state.running_state.lock().unwrap() < 1;
    //let not_running = || true;
    while not_running() {
        std::thread::sleep(Duration::from_millis(100));
    }
    ret
}

#[cfg(unix)]
pub(crate) fn capture_output<T, E, F: FnOnce() -> Result<T, E>>(
    cb: F,
) -> Result<(Vec<String>, T), E> {
    let state = Arc::new(AppState::new());
    let handle = start_server(state.clone());

    let result = cb();

    *state.running_state.lock().unwrap() = 2;
    let _ = handle.join();

    match result {
        Ok(r) => {
            let ret = state.findings.lock().unwrap().clone();
            Ok((ret, r))
        }
        Err(e) => Err(e),
    }
}

fn temp_file_to_string(file: &mut tempfile::NamedTempFile) -> anyhow::Result<String> {
    let mut ret = String::new();
    std::io::Read::read_to_string(file, &mut ret)?;
    Ok(ret)
}

fn transform_value(v: &serde_json::Value) -> Option<serde_json::Value> {
    use crate::output::raw_report::{json_to_string, json_to_string_opt};

    if !json_to_string_opt(v.get("reason")).is_some_and(|x| x == "compiler-message") {
        return None;
    }

    let message = v.get("message");
    if let Some(message) = message {
        if !message.get("code").is_some_and(|x| x.is_object()) {
            return None;
        }

        let krate = v.get("target").and_then(|x| x.get("name"));
        let ret = if let Some(krate) = krate {
            serde_json::json!({
                "crate": crate::startup::normalize_crate_name(json_to_string(krate).as_str()),
                "message": message.clone(),
            })
        } else {
            serde_json::json!({
                "message": message.clone(),
            })
        };
        Some(ret)
    } else {
        None
    }
}

#[cfg(windows)]
pub(crate) fn capture_output<
    E: std::convert::From<anyhow::Error>,
    F: FnOnce() -> Result<(bool, tempfile::NamedTempFile), E>,
>(
    cb: F,
) -> Result<(Vec<String>, (bool, tempfile::NamedTempFile)), E> {
    let (failed_build, mut stdout) = cb()?;

    let output_string =
        temp_file_to_string(&mut stdout).map_err(|_| anyhow::anyhow!("internal error"))?;
    let mut output_json = crate::startup::output_to_json(&output_string);

    let ret = output_json
        .iter_mut()
        .map(|x| transform_value(x).and_then(|y| Some(y.to_string())))
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect::<Vec<String>>();

    Ok((ret, (failed_build, stdout)))
}
