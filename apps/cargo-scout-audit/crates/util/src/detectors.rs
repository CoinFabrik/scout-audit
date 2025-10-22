use std::collections::HashSet;

use anyhow::bail;
use anyhow::Result;

fn parse_detectors(detectors: &str) -> Vec<String> {
    detectors
        .to_lowercase()
        .split(',')
        .map(|detector| detector.trim().replace('_', "-"))
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn get_filtered_detectors(filter: &str, detectors_names: &[String]) -> Result<Vec<String>> {
    let detectors_set: HashSet<_> = detectors_names.iter().collect();
    let parsed_detectors = parse_detectors(filter);

    parsed_detectors
        .iter()
        .try_fold(Vec::new(), |mut acc, detector| {
            if detectors_set.contains(detector) {
                acc.push(detector.clone());
                Ok(acc)
            } else {
                bail!("The detector '{}' does not exist. Use the `--list` flag to see available detectors.", detector)
            }
        })
}

pub fn get_excluded_detectors(excluded: &str, detectors_names: &[String]) -> Vec<String> {
    let excluded_set: HashSet<_> = parse_detectors(excluded).into_iter().collect();

    detectors_names
        .iter()
        .filter(|&name| !excluded_set.contains(name))
        .cloned()
        .collect()
}

pub fn list_detectors(detectors_names: &[String]) {
    let separator = "─".repeat(48);
    let upper_border = format!("┌{}┐", separator);
    let lower_border = format!("└{}┘", separator);
    let empty_line = format!("│{:48}│", "");

    println!("{}", upper_border);
    println!("│{:^47}│", "🔍 Available detectors:");
    println!("{}", empty_line);

    for (index, detector_name) in detectors_names.iter().enumerate() {
        println!("│ {:>2}. {:<43}│", index + 1, detector_name);
    }

    println!("{}", empty_line);
    println!("{}", lower_border);
}
