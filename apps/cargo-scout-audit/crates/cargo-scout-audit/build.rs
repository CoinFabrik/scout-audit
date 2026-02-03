use std::process::Command;

const TOOLCHAIN: [&str; 1] = ["nightly-2025-08-07"];
const COMPONENTS: [&str; 3] = ["rust-src", "llvm-tools", "rustc-dev"];

fn main() {
    for toolchain in TOOLCHAIN {
        if let Err(e) = ensure_toolchain(toolchain) {
            println!("cargo:warning={}", e);
            std::process::exit(1);
        }

        if let Err(e) = ensure_components(toolchain, &COMPONENTS) {
            println!("cargo:warning={}", e);
            std::process::exit(1);
        }

        if let Err(e) = ensure_dylint_link(toolchain) {
            println!("cargo:warning={}", e);
            std::process::exit(1);
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
            "llvm-tools" | "rustc-dev" => component_list
                .lines()
                .any(|line| line.starts_with(component) && line.ends_with("(installed)")),
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
    let status = Command::new("rustup")
        .env_remove("RUSTUP_TOOLCHAIN")
        .arg("run")
        .arg(toolchain)
        .arg("cargo")
        .arg("install")
        .arg("dylint-link")
        .status()
        .map_err(|e| format!("Failed to execute rustup run cargo install dylint-link: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Failed to install dylint-link for toolchain '{}'. Please install it manually.",
            toolchain
        ));
    }

    Ok(())
}
