use crate::artifact::{ArtifactKind, AuditManifest};
use crate::summary::ExperimentSummary;
use crate::validate::{Severity, ValidationIssue};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn render_markdown(
    manifest: &AuditManifest,
    summary: &ExperimentSummary,
    issues: &[ValidationIssue],
) -> String {
    // 生成 Markdown
    let mut output = String::new();
    output.push_str("# Experiment Audit Report\n\n");
    output.push_str(&format!("- Generated unix time: `{}`\n", unix_time()));
    output.push_str(&format!(
        "- Experiment root: `{}`\n",
        manifest.root.display()
    ));
    output.push_str(&format!(
        "- Artifact count: `{}`\n",
        manifest.artifacts.len()
    ));
    output.push_str(&format!(
        "- Total size: `{}` bytes\n\n",
        manifest.total_size_bytes()
    ));

    output.push_str("## Artifact Inventory\n\n");
    output.push_str("| Kind | Count |\n| --- | ---: |\n");
    for (kind, count) in manifest.count_by_kind() {
        output.push_str(&format!("| {} | {} |\n", kind_label(&kind), count));
    }

    output.push_str("\n## Experiment Summary\n\n");
    output.push_str(&format!("- Parsed records: `{}`\n", summary.records));
    output.push_str(&format!("- Success: `{}`\n", summary.success));
    output.push_str(&format!("- Failed: `{}`\n", summary.failed));
    output.push_str(&format!(
        "- Interrupted/unknown: `{}`\n",
        summary.interrupted_or_unknown
    ));
    output.push_str(&format!(
        "- Malformed log lines: `{}`\n",
        summary.malformed_lines
    ));
    if let Some(latency) = summary.average_latency_ms {
        output.push_str(&format!("- Average latency: `{:.2}` ms\n", latency));
    }

    output.push_str("\n## Method Distribution\n\n");
    if summary.method_counts.is_empty() {
        output.push_str("No method field was detected in parsed JSONL records.\n");
    } else {
        output.push_str("| Method | Records |\n| --- | ---: |\n");
        for (method, count) in &summary.method_counts {
            output.push_str(&format!("| `{}` | {} |\n", method, count));
        }
    }

    output.push_str("\n## Validation Issues\n\n");
    if issues.is_empty() {
        output.push_str("No blocking validation issues were detected.\n");
    } else {
        output.push_str("| Severity | Message |\n| --- | --- |\n");
        for issue in issues {
            output.push_str(&format!(
                "| {} | {} |\n",
                severity_label(&issue.severity),
                issue.message
            ));
        }
    }

    output.push_str("\n## Representative Artifacts\n\n");
    output.push_str("| Path | Kind | Size |\n| --- | --- | ---: |\n");
    // 限制展示数量以保持报告可读
    for artifact in manifest.artifacts.iter().take(30) {
        output.push_str(&format!(
            "| `{}` | {} | {} |\n",
            artifact.path.display(),
            kind_label(&artifact.kind),
            artifact.size_bytes
        ));
    }
    if manifest.artifacts.len() > 30 {
        output.push_str(&format!(
            "| ... | ... | {} more files omitted |\n",
            manifest.artifacts.len() - 30
        ));
    }

    output
}

fn kind_label(kind: &ArtifactKind) -> &'static str {
    kind.as_str()
}

fn severity_label(severity: &Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

fn unix_time() -> u64 {
    // 使用标准库时间戳
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
