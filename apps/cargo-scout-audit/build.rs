use regex::Regex;
use sha2::{Digest, Sha256};
use std::process::Command;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};
use walkdir::WalkDir;

mod build_config;
use build_config::TOOLCHAIN;

fn main() {
    match ensure_toolchain(TOOLCHAIN) {
        Ok(_) => {}
        Err(e) => {
            println!("cargo:warning={}", e);
            std::process::exit(1);
        }
    }

    match ensure_components(TOOLCHAIN, &["rust-src", "llvm-tools", "rustc-dev"]) {
        Ok(_) => {}
        Err(e) => {
            println!("cargo:warning={}", e);
            std::process::exit(1);
        }
    }

    match ensure_dylint_link(TOOLCHAIN) {
        Ok(_) => {}
        Err(e) => {
            println!("cargo:warning={}", e);
            std::process::exit(1);
        }
    }

    match write_digest_file() {
        Ok(_) => {}
        Err(e) => {
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

fn hash_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    // Open the file
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut hash = Sha256::new();

    // Read the file in chunks and hash
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    hash.update(&buffer);

    // Return hash as a hex string
    let result = hash.finalize();
    Ok(format!("{:x}", result))
}

fn hash_matching_files<P: AsRef<Path>>(
    dir: P,
    pattern: &Regex,
) -> std::io::Result<Vec<(String, String)>> {
    // Sort entries for deterministic order
    let mut entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .collect();

    entries.sort_by_key(|e| e.path().to_path_buf());

    // Collect hashes for files that match the pattern
    let mut file_hashes = Vec::new();
    for entry in entries {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
            if pattern.is_match(file_name) {
                let hash = hash_file(path)?;
                let path: String = path.to_str().unwrap().into();
                let path = path.replace("\\", "/");
                if path == "./src/digest.rs" {
                    continue;
                }
                file_hashes.push((path, hash));
            }
        }
    }
    Ok(file_hashes)
}

fn hash_directory<P: AsRef<Path>, const N: usize>(
    directories: &[P; N],
    pattern: &str,
) -> std::io::Result<String> {
    let mut hash = Sha256::new();
    let pattern = Regex::new(pattern).unwrap();
    for directory in directories.iter() {
        let hashes = hash_matching_files(directory, &pattern)?;
        for (path, digest) in hashes {
            hash.update(format!("{} {}\n", digest, path).as_bytes());
        }
    }
    let result = hash.finalize();
    Ok(format!("{:x}", result))
}

fn write_file_lazy(path: &str, contents: &[u8]) -> std::io::Result<()> {
    let temporary = format!("{path}.tmp");
    {
        let mut output = File::create(&temporary)?;
        output.write_all(contents)?;
        output.sync_all()?;
    }
    let old = hash_file(path)?;
    let new = hash_file(&temporary)?;
    if old == new {
        std::fs::remove_file(temporary)?;
    } else {
        std::fs::rename(temporary, path)?;
    }
    Ok(())
}

fn write_digest_file() -> std::io::Result<()> {
    let hash = hash_directory(&["./src", "./build_config"], r"\.rs$")?;
    write_file_lazy(
        "src/digest.rs",
        format!("pub const SOURCE_DIGEST: &str = \"{hash}\";").as_bytes(),
    )?;
    Ok(())
}
