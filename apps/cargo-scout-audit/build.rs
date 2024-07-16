use std::process::Command;

mod build_config;
use build_config::TOOLCHAIN;

fn main() {
    match ensure_toolchain() {
        Ok(_) => {}
        Err(e) => {
            println!("cargo:warning={}", e);
            std::process::exit(1);
        }
    }

    match ensure_rust_src() {
        Ok(_) => {}
        Err(e) => {
            println!("cargo:warning={}", e);
            std::process::exit(1);
        }
    }
}

fn ensure_toolchain() -> Result<(), String> {
    let output = Command::new("rustup")
        .arg("toolchain")
        .arg("list")
        .output()
        .map_err(|e| format!("Failed to execute rustup: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "rustup command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let toolchain_list = String::from_utf8_lossy(&output.stdout);

    if !toolchain_list.contains(TOOLCHAIN) {
        println!("cargo:warning=Installing toolchain '{}'...", TOOLCHAIN);
        let status = Command::new("rustup")
            .arg("toolchain")
            .arg("install")
            .arg(TOOLCHAIN)
            .status()
            .map_err(|e| format!("Failed to execute rustup toolchain install: {}", e))?;

        if !status.success() {
            return Err(format!(
                "Failed to install toolchain '{}'. Please install it manually.",
                TOOLCHAIN
            ));
        }
    }

    Ok(())
}

fn ensure_rust_src() -> Result<(), String> {
    let output = Command::new("rustup")
        .arg("component")
        .arg("list")
        .arg("--toolchain")
        .arg(TOOLCHAIN)
        .output()
        .map_err(|e| format!("Failed to execute rustup: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "rustup command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let component_list = String::from_utf8_lossy(&output.stdout);

    if !component_list.contains("rust-src (installed)") {
        println!("cargo:warning=Installing rust-src component...");
        let status = Command::new("rustup")
            .arg("component")
            .arg("add")
            .arg("rust-src")
            .arg("--toolchain")
            .arg(TOOLCHAIN)
            .status()
            .map_err(|e| format!("Failed to execute rustup component add: {}", e))?;

        if !status.success() {
            return Err(
                "Failed to install rust-src component. Please install it manually.".to_string(),
            );
        }
    }

    Ok(())
}
