#[cfg(test)]
mod tests {
    use anyhow::{Context, Ok, Result};
    use cargo_scout_audit::startup::{run_scout, OutputFormat, Scout};
    use lazy_static::lazy_static;
    use std::{fs, path::PathBuf};

    lazy_static! {
        static ref CONTRACT_PATH: PathBuf = {
            let mut path = PathBuf::from("tests");
            path.push("contract");
            path.push("Cargo.toml");
            path
        };
    }

    #[test]
    fn test_default_scout() {
        // Given
        let scout_opts = Scout {
            manifest_path: Some(CONTRACT_PATH.clone()),
            ..Scout::default()
        };

        // When
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_scout_with_exclude() {
        // Given
        let scout_opts = Scout {
            manifest_path: Some(CONTRACT_PATH.clone()),
            exclude: Some("avoid-panic-error".to_string()),
            ..Scout::default()
        };

        // When
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_scout_with_filter() {
        // Given
        let scout_opts = Scout {
            manifest_path: Some(CONTRACT_PATH.clone()),
            filter: Some("avoid-panic-error".to_string()),
            ..Scout::default()
        };

        // When
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_scout_with_profile() {
        // TODO
    }

    #[test]
    fn test_scout_list_detectors() {
        // Given
        let scout_opts = Scout {
            manifest_path: Some(CONTRACT_PATH.clone()),
            list_detectors: true,
            ..Scout::default()
        };

        // When
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_output_formats() -> Result<()> {
        let formats = vec![
            ("report.html", OutputFormat::Html),
            ("report.json", OutputFormat::Json),
            ("report.json", OutputFormat::RawJson),
            ("report.md", OutputFormat::Markdown),
            ("report.md", OutputFormat::MarkdownGithub),
            ("report.sarif", OutputFormat::Sarif),
            ("report.pdf", OutputFormat::Pdf),
        ];

        for (file, format) in formats {
            test_output_format(file, &format)
                .with_context(|| format!("Failed to test {:?} format", &format))?;

            // Clean up
            fs::remove_file(file).unwrap_or_else(|_| panic!("Should be able to delete the file"));
        }

        Ok(())
    }

    fn test_output_format(output_file: &str, format: &OutputFormat) -> Result<()> {
        // Given
        let scout_opts = Scout {
            manifest_path: Some(CONTRACT_PATH.clone()),
            output_format: Some(format.clone()),
            output_path: Some(PathBuf::from(output_file)),
            ..Scout::default()
        };

        // When
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok(), "Scout should run");

        // Check if file exists and is a file
        let metadata = fs::metadata(output_file);
        assert!(metadata.is_ok(), "Metadata should be readable",);
        let metadata = metadata.unwrap();
        assert!(metadata.is_file(), "Output should be a file",);

        // Check file size
        assert!(metadata.len() > 0, "File should not be empty",);

        if format == &OutputFormat::Pdf {
            return Ok(());
        }
        // Read file contents
        let contents = fs::read_to_string(output_file);
        assert!(contents.is_ok(), "File should be readable",);
        let contents = contents.unwrap();

        // Check file contents
        assert!(!contents.is_empty(), "File contents should not be empty",);

        Ok(())
    }
}
