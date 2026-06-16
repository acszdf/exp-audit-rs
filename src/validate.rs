use crate::artifact::{ArtifactKind, AuditManifest};
use crate::summary::ExperimentSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
}

/// Convert scanned files and parsed records into actionable audit warnings.
pub fn validate(manifest: &AuditManifest, summary: &ExperimentSummary) -> Vec<ValidationIssue> {
    let counts = manifest.count_by_kind();
    let mut issues = Vec::new();

    if !counts.contains_key(&ArtifactKind::Config) {
        issues.push(error(
            "missing configuration file: expected yaml/yml/toml/json config artifact",
        ));
    }
    if !counts.contains_key(&ArtifactKind::JsonLog) {
        issues.push(error(
            "missing jsonl log: no structured experiment records were found",
        ));
    }
    if !counts.contains_key(&ArtifactKind::ResultJson) {
        issues.push(warning(
            "missing result json: final evaluation outputs may be hard to audit",
        ));
    }
    if summary.records == 0 {
        // No records is blocking because later statistics would be meaningless.
        issues.push(error(
            "no parsed experiment records: jsonl logs are empty or unavailable",
        ));
    }
    if summary.malformed_lines > 0 {
        issues.push(warning(format!(
            "{} malformed jsonl lines were skipped",
            summary.malformed_lines
        )));
    }
    if summary.interrupted_or_unknown > 0 {
        issues.push(warning(format!(
            "{} records have unknown/interrupted status",
            summary.interrupted_or_unknown
        )));
    }
    if summary.method_counts.len() <= 1 {
        issues.push(warning(
            "only one or zero methods detected: cross-run comparison may be limited",
        ));
    }

    issues
}

fn error(message: impl Into<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Error,
        message: message.into(),
    }
}

fn warning(message: impl Into<String>) -> ValidationIssue {
    ValidationIssue {
        severity: Severity::Warning,
        message: message.into(),
    }
}

pub fn issues_to_json(issues: &[ValidationIssue]) -> String {
    // Manual JSON keeps the tool dependency-free for simple CLI use.
    let items = issues
        .iter()
        .map(|issue| {
            format!(
                "{{\"severity\":\"{}\",\"message\":\"{}\"}}",
                severity_label(&issue.severity),
                escape_json(&issue.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n  ");
    format!("[\n  {}\n]", items)
}

fn severity_label(severity: &Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
