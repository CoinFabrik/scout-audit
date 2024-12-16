use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
    str::FromStr,
};

// Writes data to a file at the specified path, creating the path if it doesn't exist.
pub fn write_to_file(path: &PathBuf, data: &[u8]) -> io::Result<()> {
    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to a temporary file first
    let temp_path = path.with_extension("tmp");
    let mut temp_file = File::create(&temp_path)?;

    {
        temp_file.write_all(data)?;
        temp_file.sync_all()?;
    }

    // Rename temporary file to the target path
    match fs::rename(&temp_path, path) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Attempt to clean up the temporary file in case of an error
            let _ = fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

pub fn capitalize(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}

pub fn sanitize_category_name(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

fn exists(path: &String) -> bool {
    std::path::Path::new(path).exists()
}

pub fn get_template<F: FnOnce() -> (String, String)>(get_path: F, template: &str) -> String {
    let (dir, file) = get_path();
    if !exists(&dir) {
        let _ = fs::create_dir_all(&dir);
    }
    let path = dir + file.as_str();
    if exists(&path) {
        let path = PathBuf::from_str(path.as_str());
        if let Ok(path) = path {
            let _ = crate::output::utils::write_to_file(&path, template.as_bytes());
        }
        return template.to_string();
    }
    let file = fs::File::open(path);
    if file.is_err() {
        return template.to_string();
    }
    let mut file = file.unwrap();
    let mut ret = String::new();
    if file.read_to_string(&mut ret).is_err() {
        return template.to_string();
    }
    ret
}
