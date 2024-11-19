#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use cargo_scout_audit::startup::{run_scout, OutputFormat, Scout};
    use std::{collections::HashMap, fs, path::PathBuf};

    fn get_detectors_dir(blockchain: &str) -> Result<PathBuf> {
        let mut ret = std::env::current_dir()?;
        ret.pop();
        ret.pop();
        ret.push("detectors");
        ret.push(blockchain);
        Ok(ret)
    }

    fn get_test_cases() -> Vec<(PathBuf, String)> {
        let contracts_dir = PathBuf::from("tests").join("contracts");
        let mut contract_paths: Vec<(PathBuf, String)> = fs::read_dir(contracts_dir)
            .expect("Should read contracts directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| {
                (
                    entry.path().join("Cargo.toml"),
                    entry.file_name().to_str().unwrap().to_string(),
                )
            })
            .filter(|(path, _)| path.exists())
            .collect();
        contract_paths.sort_by_key(|x| x.0.clone());
        contract_paths
    }

    #[test]
    fn test_default_scout() {
        // Given
        let contract_paths = get_test_cases();
        for (contract_path, chain) in contract_paths {
            // When
            let scout_opts = Scout {
                manifest_path: Some(contract_path),
                local_detectors: Some(get_detectors_dir(&chain).unwrap()),
                ..Scout::default()
            };
            let result = run_scout(scout_opts);

            // Then
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_scout_with_exclude() {
        // Given
        let contract_paths = get_test_cases();
        for (contract_path, chain) in contract_paths {
            // When
            let scout_opts = Scout {
                manifest_path: Some(contract_path),
                local_detectors: Some(get_detectors_dir(&chain).unwrap()),
                exclude: Some("unsafe-unwrap".to_string()),
                ..Scout::default()
            };
            let result = run_scout(scout_opts);

            // Then
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_scout_with_filter() {
        // Given
        let contract_paths = get_test_cases();

        // When
        for (contract_path, chain) in contract_paths {
            let scout_opts = Scout {
                manifest_path: Some(contract_path),
                local_detectors: Some(get_detectors_dir(&chain).unwrap()),
                filter: Some("unsafe-unwrap".to_string()),
                ..Scout::default()
            };
            let result = run_scout(scout_opts);

            // Then
            assert!(result.is_ok());
        }
    }

    // #[test]
    // fn test_scout_with_profile() {
    //     // TODO
    // }

    #[test]
    fn test_scout_list_detectors() {
        // Given
        let contract_paths = get_test_cases();

        // When
        for (contract_path, chain) in contract_paths {
            let scout_opts = Scout {
                manifest_path: Some(contract_path),
                local_detectors: Some(get_detectors_dir(&chain).unwrap()),
                list_detectors: true,
                ..Scout::default()
            };
            let result = run_scout(scout_opts);

            // Then
            assert!(result.is_ok());
        }
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

        let (contract_path, chain) = get_test_cases().iter().first().unwrap();

        // Given
        let scout_opts = Scout {
            manifest_path: Some(contract_path),
            output_format: vec![format.clone()],
            output_path: Some(PathBuf::from(output_file)),
            local_detectors: Some(get_detectors_dir(&chain)?),
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
    fn test_finding_presence() {
        // Given
        let (contract_path, chain) = get_test_cases()
            .iter()
            .find(|y| y.1 == "soroban")
            .unwrap()
            .clone();

        // When
        let scout_opts = Scout {
            manifest_path: Some(contract_path.to_path_buf()),
            local_detectors: Some(get_detectors_dir(&chain).unwrap()),
            ..Scout::default()
        };
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok(), "Scout should run");
        let result = result.unwrap();

        let findings = result.iter().map(|value| value.code()).collect::<Vec<_>>();
        let counts = count_strings(&findings);
        assert!(counts.is_some(), "Scout returned data in an invalid format");
        let counts = counts.unwrap();
        let expected = [
            ("overflow_check", 1_usize),
            ("soroban_version", 1_usize),
            ("integer_overflow_or_underflow", 1_usize),
            ("divide_before_multiply", 1_usize),
        ];
        check_counts(&counts, &expected);
    }

    fn count_strings(strings: &[String]) -> Option<HashMap<String, usize>> {
        let mut ret = HashMap::<String, usize>::new();
        for i in strings.iter() {
            if i.is_empty() {
                return None;
            }
            let value = ret.get(i).unwrap_or(&0) + 1;
            ret.insert(i.clone(), value);
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
}
