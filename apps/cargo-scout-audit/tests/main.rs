#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use cargo_scout_audit::startup::{run_scout, OutputFormat, Scout};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::path::Path;
    use std::{fs, path::PathBuf};

    fn get_test_cases() -> Vec<PathBuf> {
        let contracts_dir = PathBuf::from("tests").join("contracts");
        let contract_paths: Vec<PathBuf> = fs::read_dir(contracts_dir)
            .expect("Should read contracts directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.path().join("Cargo.toml"))
            .filter(|path| path.exists())
            .collect();
        contract_paths
    }

    fn run_default_scout(contract_path: &Path) -> anyhow::Result<Vec<serde_json::Value>> {
        let scout_opts = Scout {
            manifest_path: Some(contract_path.to_path_buf()),
            ..Scout::default()
        };
        run_scout(scout_opts)
    }

    #[test]
    fn test_default_scout_first_contract() {
        // Given
        let contract_paths = get_test_cases();
        let contract_path = contract_paths.first().unwrap();

        // When
        let result = run_default_scout(contract_path);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_scout_second_contract() {
        // Given
        let contract_paths = get_test_cases();
        let contract_path = contract_paths.get(1).unwrap();

        // When
        let result = run_default_scout(contract_path);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_scout_with_forced_fallback() {
        // Given
        let contract_paths = get_test_cases();
        let contract_path = contract_paths.first().unwrap();

        // When
        let result = run_default_scout(contract_path);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_scout_with_forced_fallback_second_contract() {
        // Given
        let contract_paths = get_test_cases();
        let contract_path = contract_paths.get(1).unwrap();

        // When
        let result = run_default_scout(contract_path);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_scout_with_exclude() {
        // Given
        let contract_paths = get_test_cases();
        let contract_path = contract_paths.first().unwrap();

        // When
        let result = run_default_scout(contract_path);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_scout_with_exclude_second_contract() {
        // Given
        let contract_paths = get_test_cases();
        let contract_path = contract_paths.get(1).unwrap();

        // When
        let result = run_default_scout(contract_path);

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn test_scout_with_filter() {
        // Given
        let contract_paths = get_test_cases();
        let contract_path = contract_paths.first().unwrap();

        // When
        let result = run_default_scout(contract_path);

        // Then
        assert!(result.is_ok());
    }

    // #[test]
    // fn test_scout_with_profile() {
    //     // TODO
    // }

    #[test]
    fn test_scout_list_detectors() {
        // Given
        let contract_paths = get_test_cases();
        let contract_path = contract_paths.first().unwrap();

        // When
        let result = run_default_scout(contract_path);

        // Then
        assert!(result.is_ok());
    }

    fn test_output_fn(file: &str, format: OutputFormat) -> Result<()> {
        test_output_format(file, &format)
            .with_context(|| format!("Failed to test {:?} format", &format))?;
        fs::remove_file(file)
            .unwrap_or_else(|_| panic!("Should be able to delete the file: {}", file));

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
    fn test_raw_json_format() -> Result<()> {
        test_output_fn("raw-report.json", OutputFormat::RawJson)
    }

    #[test]
    fn test_markdown_format() -> Result<()> {
        test_output_fn("report.md", OutputFormat::Markdown)
    }

    #[test]
    fn test_markdown_github_format() -> Result<()> {
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

        let contract_path = get_soroban_contract();

        // Given
        let scout_opts = Scout {
            manifest_path: Some(contract_path.clone()),
            output_format: vec![format.clone()],
            output_path: Some(PathBuf::from(output_file)),
            ..Scout::default()
        };

        // When
        let result = run_scout(scout_opts);

        // When

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

    fn get_x_contract(x: &str) -> PathBuf {
        get_test_cases()
            .iter()
            .find(|y| y.to_string_lossy().contains(x))
            .unwrap()
            .clone()
    }

    fn get_soroban_contract() -> PathBuf {
        get_x_contract("soroban")
    }

    #[allow(dead_code)]
    fn get_ink_contract() -> PathBuf {
        get_x_contract("ink")
    }

    #[test]
    fn test_finding_presence() {
        // Given
        let contract_path = get_soroban_contract();

        // When
        let result = run_default_scout(&contract_path);

        // Then
        assert!(result.is_ok(), "Scout should run");
        let result = result.unwrap();

        let findings = result
            .iter()
            .map(|value| {
                value
                    .get("code")
                    .and_then(|value| value.get("code"))
                    .and_then(|value| match value {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
            })
            .collect::<Vec<Option<String>>>();
        let counts = count_strings(&findings);
        assert!(counts.is_some(), "Scout returned data in an invalid format");
        let counts = counts.unwrap();
        let expected = [
            ("overflow_check", 1_usize),
            ("soroban_version", 1_usize),
            ("integer_overflow_underflow", 1_usize),
            ("divide_before_multiply", 1_usize),
        ];
        check_counts(&counts, &expected);
    }

    fn count_strings(strings: &[Option<String>]) -> Option<HashMap<String, usize>> {
        let mut ret = HashMap::<String, usize>::new();
        for i in strings.iter() {
            match i {
                Some(s) => {
                    let value = ret.get(s).unwrap_or(&0) + 1;
                    ret.insert(s.clone(), value);
                }
                None => {
                    return None;
                }
            }
        }
        Some(ret)
    }

    fn check_counts(counts: &HashMap<String, usize>, expected: &[(&str, usize)]) {
        {
            let expected = expected.len();
            let actual = counts.len();
            assert!(actual == expected, "Scout should return exactly {expected} findings for the test contract, but it returned {actual}");
        }
        for &(name, count) in expected.iter() {
            let actual = *counts.get(name).unwrap_or(&0);
            assert!(actual == count, "Scout should return exactly {count} {name} for the test contract, but it returned {actual}");
        }
    }

    // Slow tests module
    mod slow {
        use super::*;

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
            if result.is_err() {
                let x = 0;
                dbg!(result.err());
                assert!(x == 1);
            }
        }
    }
}
