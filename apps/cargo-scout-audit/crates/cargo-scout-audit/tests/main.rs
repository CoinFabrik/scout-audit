#![feature(rustc_private)]
extern crate rustc_driver;

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use cargo_scout_audit::run::run_scout;
    use cli_args::{OutputFormat, Scout};
    use lazy_static::lazy_static;
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
        process::Command,
    };
    use tempfile::TempDir;
    use uuid::Uuid;

    lazy_static! {
        static ref TEST_DIR: TempDir = TempDir::new().expect("Failed to create temp directory");
    }

    lazy_static! {
        static ref DETECTORS_DIR: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .map(|ancestor| ancestor.join("nightly"))
            .find(|candidate| candidate.is_dir())
            .expect("Failed to locate the 'nightly' detectors directory");
    }

    fn create_cargo_command() -> Command {
        Command::new("cargo")
    }

    fn get_test_cases() -> Vec<PathBuf> {
        let contracts_dir = PathBuf::from("tests").join("contracts");
        let mut contract_paths = fs::read_dir(&contracts_dir)
            .unwrap_or_else(|_| panic!("Failed to read directory: {}", contracts_dir.display()))
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let manifest = e.path().join("Cargo.toml");
                    if manifest.is_file() {
                        Some(manifest)
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        contract_paths.sort();
        contract_paths
    }

    #[test]
    fn test_default_scout() {
        // Given
        let contract_paths = get_test_cases();
        for contract_path in contract_paths {
            // When
            let scout_opts = Scout {
                manifest_path: Some(contract_path),
                local_detectors: Some(DETECTORS_DIR.clone()),
                ..Scout::default()
            };
            let result = run_scout(scout_opts);

            if let Err(e) = &result {
                println!("Error: {}", e);
            }

            // Then
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_scout_with_exclude() {
        // Given
        let contract_paths = get_test_cases();
        for contract_path in contract_paths {
            // When
            let scout_opts = Scout {
                manifest_path: Some(contract_path),
                local_detectors: Some(DETECTORS_DIR.clone()),
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
        for contract_path in contract_paths {
            let scout_opts = Scout {
                manifest_path: Some(contract_path),
                local_detectors: Some(DETECTORS_DIR.clone()),
                filter: Some("unsafe-unwrap".to_string()),
                ..Scout::default()
            };
            let result = run_scout(scout_opts);

            // Then
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_scout_list_detectors() {
        // Given
        let contract_paths = get_test_cases();

        // When
        for contract_path in contract_paths {
            let scout_opts = Scout {
                manifest_path: Some(contract_path),
                local_detectors: Some(DETECTORS_DIR.clone()),
                list_detectors: true,
                ..Scout::default()
            };
            let result = run_scout(scout_opts);

            // Then
            assert!(result.is_ok());
        }
    }

    fn test_output_fn(base_name: &str, format: OutputFormat) -> Result<()> {
        let unique_name = format!("{}_{}", Uuid::new_v4(), base_name);
        let file_path = TEST_DIR.path().join(unique_name);

        let result = test_output_format(&file_path.to_string_lossy(), &format);
        let _ = fs::remove_file(&file_path);

        result
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

        let contract_path = get_test_cases().first().unwrap().clone();

        // Given
        let scout_opts = Scout {
            manifest_path: Some(contract_path),
            output_format: vec![format.clone()],
            output_path: Some(PathBuf::from(output_file)),
            local_detectors: Some(DETECTORS_DIR.clone()),
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
        let contract_path = get_test_cases()
            .iter()
            .find(|y| y.to_str().unwrap().contains("soroban"))
            .unwrap()
            .clone();

        // When
        let scout_opts = Scout {
            manifest_path: Some(contract_path.to_path_buf()),
            local_detectors: Some(DETECTORS_DIR.clone()),
            ..Scout::default()
        };
        let result = run_scout(scout_opts);

        // Then
        assert!(result.is_ok(), "Scout should run");
        let result = result.unwrap();

        let findings = result
            .findings
            .iter()
            .map(|value| value.code())
            .filter(|x| x != "known_vulnerabilities")
            .collect::<Vec<_>>();
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

    #[test]
    fn test_message_format() {
        let path = "tests/contracts/substrate-pallets/";

        let scout_output = create_cargo_command()
            .args([
                "scout-audit",
                "--local-detectors",
                DETECTORS_DIR.to_str().unwrap(),
                "--",
                "--message-format=json",
            ])
            .current_dir(path)
            .output()
            .unwrap();

        let output = std::str::from_utf8(&scout_output.stdout).unwrap();
        let output = output
            .lines()
            .map(serde_json::from_str::<serde_json::Value>)
            .collect::<Vec<_>>();
        assert!(
            output.iter().all(|x| x.is_ok()),
            "Output should be valid JSON",
        );
    }

    #[test]
    fn test_metadata() {
        let path = "tests/contracts/substrate-pallets/";

        let scout_output = create_cargo_command()
            .args([
                "scout-audit",
                "--metadata",
                "--local-detectors",
                DETECTORS_DIR.to_str().unwrap(),
            ])
            .current_dir(path)
            .output()
            .unwrap();
        let scout_stdout = std::str::from_utf8(&scout_output.stdout).unwrap();
        let json_result: Result<serde_json::Value, _> =
            serde_json::from_slice(&scout_output.stdout);
        assert!(
            json_result.is_ok(),
            "Output should be valid JSON, got: {:?}",
            scout_stdout
        );
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
            assert!(
                actual == expected,
                "Scout should return exactly {expected} findings for the test contract, but it returned {actual}"
            );
        }
        for &(name, count) in expected.iter() {
            let actual = *counts.get(name).unwrap_or(&0);
            assert!(
                actual == count,
                "Scout should return exactly {count} {name} for the test contract, but it returned {actual}"
            );
        }
    }
}
