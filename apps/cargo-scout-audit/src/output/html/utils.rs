use std::collections::HashMap;

use crate::output::report;

pub fn get_analytics(report: &report::Report) -> HashMap<String, u32> {
    let mut analytics = HashMap::new();

    for finding in &report.findings {
        let count = analytics.entry(finding.file.clone()).or_insert(0);
        *count += 1;
    }

    analytics
}
