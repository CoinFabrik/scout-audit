use axum::{http::StatusCode, routing::post, Router};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

fn port_is_available_on_localhost(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn find_available_port() -> Option<u16> {
    (49152..65535).find(|port| port_is_available_on_localhost(*port))
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
    let port = find_available_port();
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
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    let future =
        axum::serve(listener, app).with_graceful_shutdown(graceful_shutdown(state.clone()));

    *state.running_state.lock().unwrap() = 1;

    future.await.unwrap();
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
