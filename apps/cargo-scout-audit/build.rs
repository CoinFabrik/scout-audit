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

        match ensure_components(toolchain, &["rust-src", "llvm-tools", "rustc-dev"]) {
            Ok(_) => {}
            Err(e) => {
                println!("cargo:warning={}", e);
                std::process::exit(1);
            }
        }

        match ensure_dylint_link(toolchain) {
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


fn ensure_components(toolchain: &str, components: &[&str]) -> Result<(), String> {
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

    for component in components {
        let is_installed = match *component {
            "rust-src" => component_list.contains("rust-src (installed)"),
            "llvm-tools" | "rustc-dev" => component_list.lines().any(|line|
                line.starts_with(component) && line.ends_with("(installed)")
            ),
            _ => false,
        };

        if !is_installed {
            println!("cargo:warning=Installing {} component...", component);
            let status = Command::new("rustup")
                .arg("component")
                .arg("add")
                .arg(component)
                .arg("--toolchain")
                .arg(toolchain)
                .status()
                .map_err(|e| format!("Failed to execute rustup component add: {}", e))?;

            if !status.success() {
                return Err(format!(
                    "Failed to install {} component. Please install it manually.",
                    component
                ));
            }
        }
    }

    Ok(())
}

fn ensure_dylint_link(toolchain: &str) -> Result<(), String> {
    let toolchain_arg = format!("+{}", toolchain);

    let status = Command::new("cargo")
        .arg(&toolchain_arg)
        .arg("install")
        .arg("dylint-link")
        .status()
        .map_err(|e| format!("Failed to execute cargo install dylint-link: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Failed to install dylint-link for toolchain '{}'. Please install it manually.",
            toolchain
        ));
    }

    Ok(())
}
