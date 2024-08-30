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
    fn test_scout_with_forced_fallback() {
        // Given
        let scout_opts = Scout {
            manifest_path: Some(CONTRACT_PATH.clone()),
            force_fallback: true,
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


    fn test_output_fn(file: &str, format: OutputFormat) -> Result<()> {
        test_output_format(file, &format)
            .with_context(|| format!("Failed to test {:?} format", &format))?;
        fs::remove_file(file).unwrap_or_else(|_| panic!("Should be able to delete the file"));
    
        Ok(())
    }
    
    #[test]
    fn test_html_format() -> Result<()> {
        test_output_fn("report.html", OutputFormat::Html)
    }

    #[test]
    fn test_json_format() -> Result<()> {
        test_output_fn("report.json", OutputFormat::Json)
    }

    #[test]
    fn test_rawJson_format() -> Result<()> {
        test_output_fn("raw-report.json", OutputFormat::RawJson)
    }

    #[test]
    fn test_markdown_format() -> Result<()> {
        test_output_fn("report.md", OutputFormat::Markdown)
    }

    #[test]
    fn test_markdownGithub_format() -> Result<()> {
        test_output_fn("report.md", OutputFormat::MarkdownGithub)
    }

    #[test]
    fn test_sarif_format() -> Result<()> {
        test_output_fn("report.sarif", OutputFormat::Sarif)
    }

    #[test]
    fn test_pdf_format() -> Result<()> {
        test_output_fn("report.pdf", OutputFormat::Pdf)
    }
    
   

    fn test_output_format(output_file: &str, format: &OutputFormat) -> Result<()> {
        // For debugging purposes
        let output_format = format.clone();

        // Given
        let scout_opts = Scout {
            manifest_path: Some(CONTRACT_PATH.clone()),
            output_formats: vec![format.clone()],
            output_path: Some(PathBuf::from(output_file)),
            ..Scout::default()
        };

        // When
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok(), "[{:?}] Scout should run", output_format);

        // Check if file exists and is a file
        let metadata = fs::metadata(output_file);
        assert!(
            metadata.is_ok(),
            "[{:?}] Metadata should be readable",
            output_format
        );
        let metadata = metadata.unwrap();
        assert!(
            metadata.is_file(),
            "[{:?}] Output should be a file",
            output_format
        );

        // Check file size
        assert!(
            metadata.len() > 0,
            "[{:?}] File should not be empty",
            output_format
        );

        if format == &OutputFormat::Pdf {
            return Ok(());
        }
        // Read file contents
        let contents = fs::read_to_string(output_file);
        assert!(
            contents.is_ok(),
            "[{:?}] File should be readable",
            output_format
        );
        let contents = contents.unwrap();

        // Check file contents
        assert!(
            !contents.is_empty(),
            "[{:?}] File contents should not be empty",
            output_format
        );

        Ok(())
    }


    
    
    #[test]
    fn test_scout_soroban_coverage() {
        // Given

        
        let scout_opts = Scout {
            manifest_path: Some("./tests/test-cases/avoid-unsafe-block/Cargo.toml".into()),
            force_fallback: true,
            ..Scout::default()
        };

        // When
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok());
    }
}
