use std::collections::HashMap;
use crate::finding::Finding;

struct FindingsCache {
    by_file: HashMap<String, FileFindings>,
}

#[derive(Debug, Clone)]
struct PostProcFinding {
    detector: String,
    file_name: String,
    span: (usize, usize),
    allowed_lint: Option<String>,
}

struct FileFindings {
    unnecessary_allows: Vec<PostProcFinding>,
    other_findings: Vec<PostProcFinding>,
}

fn parse_finding(finding: &Finding) -> Option<PostProcFinding> {
    let detector = finding.code();
    let spans = finding.spans()?;
    let span = spans.get(0)?;
    let file_name = span.get("file_name")?.as_str()?;
    let line_start = span.get("line_start")?.as_u64()?;
    let line_end = span.get("line_end")?.as_u64()?;

    let allowed_lint = if detector == "unnecessary_lint_allow" {
        finding.children()?
            .get(0)?
            .get("message")?
            .as_str()
            .and_then(|msg| msg.split('`').nth(1).map(String::from))
    } else {
        None
    };

    let start = usize::try_from(line_start).ok()?;
    let end = usize::try_from(line_end).ok()?;

    Some(PostProcFinding {
        detector: detector.to_owned(),
        file_name: file_name.to_owned(),
        span: (start, end),
        allowed_lint,
    })
}

impl FindingsCache {
    fn new(all_findings: &[Finding]) -> Self {
        let mut by_file: HashMap<String, FileFindings> = HashMap::new();

        for finding in all_findings {
            if let Some(parsed) = parse_finding(finding) {
                by_file
                    .entry(parsed.file_name.clone())
                    .or_insert_with(|| FileFindings {
                        unnecessary_allows: Vec::new(),
                        other_findings: Vec::new(),
                    })
                    .add_finding(parsed);
            }
        }

        FindingsCache { by_file }
    }
}

impl FileFindings {
    fn add_finding(&mut self, finding: PostProcFinding) {
        if finding.detector == "unnecessary_lint_allow" {
            self.unnecessary_allows.push(finding);
        } else {
            self.other_findings.push(finding);
        }
    }
}

fn spans_overlap(span1: (usize, usize), span2: (usize, usize)) -> bool {
    span1.0 <= span2.1 && span2.0 <= span1.1
}

fn should_include_finding(finding: &Finding, cache: &FindingsCache) -> bool {
    let current_finding = match parse_finding(finding) {
        Some(f) => f,
        None => return false, // If we can't parse the finding, we don't include it
    };

    if let Some(file_findings) = cache.by_file.get(&current_finding.file_name) {
        if current_finding.detector == "unnecessary_lint_allow" {
            if let Some(allowed_lint) = &current_finding.allowed_lint {
                !file_findings.other_findings.iter().any(|f| {
                    &f.detector == allowed_lint && spans_overlap(f.span, current_finding.span)
                })
            } else {
                true // Include if we can't determine the allowed lint
            }
        } else {
            !file_findings.unnecessary_allows.iter().any(|allow| {
                allow
                    .allowed_lint
                    .as_ref()
                    .map_or(false, |lint| lint == &current_finding.detector)
                    && spans_overlap(allow.span, current_finding.span)
            })
        }
    } else {
        true // If we can't find the file, we include it by default
    }
}

pub fn process(
    successful_findings: Vec<Finding>,
    output: Vec<Finding>,
    inside_vscode: bool,
) -> (Vec<Finding>, String) {
    let findings_cache = FindingsCache::new(&successful_findings);

    let console_findings: Vec<_> = successful_findings
        .into_iter()
        .filter(|finding| should_include_finding(finding, &findings_cache))
        .collect();

    let output_vscode: Vec<_> = if inside_vscode {
        output
            .into_iter()
            .filter(|val| should_include_finding(val, &findings_cache))
            .collect()
    } else {
        Vec::new()
    };

    let output_string_vscode = output_vscode
        .into_iter()
        .filter_map(|finding| serde_json::to_string(&finding.json()).ok())
        .collect::<Vec<_>>()
        .join("\n");

    (console_findings, output_string_vscode)
}
