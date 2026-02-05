use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};
use walkdir::WalkDir;

fn main() {
    if let Err(e) = write_digest_file() {
        println!("cargo:warning={}", e);
        std::process::exit(1);
    }
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

fn hash_file_allow_missing(path: &Path) -> std::io::Result<String> {
    if !path.exists() {
        Ok("".into())
    } else {
        hash_file(path)
    }
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

fn write_file_lazy(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension("rs.tmp");
    {
        let mut output = File::create(&temporary)?;
        output.write_all(contents)?;
        output.sync_all()?;
    }
    let old = hash_file_allow_missing(path)?;
    let new = hash_file(&temporary)?;
    if old == new {
        std::fs::remove_file(temporary)?;
    } else {
        std::fs::rename(temporary, path)?;
    }
    Ok(())
}

fn write_digest_file() -> std::io::Result<()> {
    let hash = hash_directory(&["./src"], r"\.rs$")?;
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("digest.rs");
    write_file_lazy(
        &dest_path,
        format!("pub const SOURCE_DIGEST: &str = \"{hash}\";").as_bytes(),
    )?;
    Ok(())
}
