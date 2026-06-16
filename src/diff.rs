use crate::artifact::{ArtifactKind, AuditManifest};
use crate::summary::ExperimentSummary;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct AuditDiff {
    pub left_root: String,
    pub right_root: String,
    pub artifact_delta: isize,
    pub record_delta: isize,
    pub success_delta: isize,
    pub failed_delta: isize,
    pub only_left_kinds: Vec<ArtifactKind>,
    pub only_right_kinds: Vec<ArtifactKind>,
}

pub fn diff(
    left_manifest: &AuditManifest,
    left_summary: &ExperimentSummary,
    right_manifest: &AuditManifest,
    right_summary: &ExperimentSummary,
) -> AuditDiff {
    let left_kinds: BTreeSet<_> = left_manifest.count_by_kind().keys().cloned().collect();
    let right_kinds: BTreeSet<_> = right_manifest.count_by_kind().keys().cloned().collect();

    AuditDiff {
        left_root: left_manifest.root.display().to_string(),
        right_root: right_manifest.root.display().to_string(),
        artifact_delta: right_manifest.artifacts.len() as isize
            - left_manifest.artifacts.len() as isize,
        record_delta: right_summary.records as isize - left_summary.records as isize,
        success_delta: right_summary.success as isize - left_summary.success as isize,
        failed_delta: right_summary.failed as isize - left_summary.failed as isize,
        only_left_kinds: left_kinds.difference(&right_kinds).cloned().collect(),
        only_right_kinds: right_kinds.difference(&left_kinds).cloned().collect(),
    }
}

pub fn render_text(diff: &AuditDiff) -> String {
    format!(
        "\
left:  {}
right: {}
artifact_delta: {}
record_delta: {}
success_delta: {}
failed_delta: {}
only_left_kinds: {:?}
only_right_kinds: {:?}
",
        diff.left_root,
        diff.right_root,
        diff.artifact_delta,
        diff.record_delta,
        diff.success_delta,
        diff.failed_delta,
        diff.only_left_kinds,
        diff.only_right_kinds
    )
}

pub fn to_json(diff: &AuditDiff) -> String {
    format!(
        "{{\n  \"left_root\": \"{}\",\n  \"right_root\": \"{}\",\n  \"artifact_delta\": {},\n  \"record_delta\": {},\n  \"success_delta\": {},\n  \"failed_delta\": {},\n  \"only_left_kinds\": {},\n  \"only_right_kinds\": {}\n}}",
        escape_json(&diff.left_root),
        escape_json(&diff.right_root),
        diff.artifact_delta,
        diff.record_delta,
        diff.success_delta,
        diff.failed_delta,
        kinds_json(&diff.only_left_kinds),
        kinds_json(&diff.only_right_kinds),
    )
}

fn kinds_json(kinds: &[ArtifactKind]) -> String {
    let items = kinds
        .iter()
        .map(|kind| format!("\"{}\"", kind.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", items)
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
