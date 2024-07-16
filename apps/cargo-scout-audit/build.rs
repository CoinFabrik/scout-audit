use std::process::Command;

mod build_config;
use build_config::TOOLCHAINS;

fn main() {
    for toolchain in TOOLCHAINS.iter() {
        match ensure_toolchain(toolchain) {
            Ok(_) => {}
            Err(e) => {
                println!("cargo:warning={}", e);
                std::process::exit(1);
            }
        }

        match ensure_rust_src(toolchain) {
            Ok(_) => {}
            Err(e) => {
                println!("cargo:warning={}", e);
                std::process::exit(1);
            }
        }
    }
}

fn ensure_toolchain(toolchain: &str) -> Result<(), String> {
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

    if !toolchain_list.contains(toolchain) {
        println!("cargo:warning=Installing toolchain '{}'...", toolchain);
        let status = Command::new("rustup")
            .arg("toolchain")
            .arg("install")
            .arg(toolchain)
            .status()
            .map_err(|e| format!("Failed to execute rustup toolchain install: {}", e))?;

        if !status.success() {
            return Err(format!(
                "Failed to install toolchain '{}'. Please install it manually.",
                toolchain
            ));
        }
    }

    Ok(())
}

fn ensure_rust_src(toolchain: &str) -> Result<(), String> {
    let output = Command::new("rustup")
        .arg("component")
        .arg("list")
        .arg("--toolchain")
        .arg(toolchain)
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
            .arg(toolchain)
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
