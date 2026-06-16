use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ExperimentSummary {
    pub records: usize,
    pub success: usize,
    pub failed: usize,
    pub interrupted_or_unknown: usize,
    pub malformed_lines: usize,
    pub average_latency_ms: Option<f64>,
    pub method_counts: BTreeMap<String, usize>,
    pub error_counts: BTreeMap<String, usize>,
}

impl ExperimentSummary {
    pub fn merge_record(&mut self, record: ParsedRecord) {
        self.records += 1;
        match record.status.as_deref() {
            Some("success") | Some("ok") | Some("passed") => self.success += 1,
            Some("failed") | Some("failure") | Some("error") => self.failed += 1,
            _ => self.interrupted_or_unknown += 1,
        }

        if let Some(method) = record.method {
            *self.method_counts.entry(method).or_insert(0) += 1;
        }
        if let Some(error) = record.error {
            *self.error_counts.entry(error).or_insert(0) += 1;
        }

        if let Some(latency) = record.latency_ms {
            let old_count = self.records.saturating_sub(1) as f64;
            let old_average = self.average_latency_ms.unwrap_or(0.0);
            self.average_latency_ms =
                Some((old_average * old_count + latency) / self.records as f64);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedRecord {
    pub status: Option<String>,
    pub method: Option<String>,
    pub error: Option<String>,
    pub latency_ms: Option<f64>,
}

pub fn summarize(root: impl AsRef<Path>) -> io::Result<ExperimentSummary> {
    let root = root.as_ref();
    let mut summary = ExperimentSummary::default();
    summarize_dir(root, &mut summary)?;
    Ok(summary)
}

fn summarize_dir(path: &Path, summary: &mut ExperimentSummary) -> io::Result<()> {
    let mut entries = fs::read_dir(path)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            summarize_dir(&path, summary)?;
        } else if metadata.is_file() && has_extension(&path, "jsonl") {
            summarize_jsonl(&path, summary)?;
        }
    }

    Ok(())
}

fn summarize_jsonl(path: &Path, summary: &mut ExperimentSummary) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match parse_record(&line) {
            Some(record) => summary.merge_record(record),
            None => summary.malformed_lines += 1,
        }
    }

    Ok(())
}

pub fn parse_record(line: &str) -> Option<ParsedRecord> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return None;
    }

    Some(ParsedRecord {
        status: string_field(trimmed, &["status", "result", "outcome"]).map(normalize_status),
        method: string_field(trimmed, &["method", "attack", "algorithm", "strategy"]),
        error: string_field(trimmed, &["error", "error_type", "exception"]),
        latency_ms: number_field(
            trimmed,
            &["latency_ms", "duration_ms", "elapsed_ms", "time_ms"],
        ),
    })
}

pub fn to_json(summary: &ExperimentSummary) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!("  \"records\": {},\n", summary.records));
    output.push_str(&format!("  \"success\": {},\n", summary.success));
    output.push_str(&format!("  \"failed\": {},\n", summary.failed));
    output.push_str(&format!(
        "  \"interrupted_or_unknown\": {},\n",
        summary.interrupted_or_unknown
    ));
    output.push_str(&format!(
        "  \"malformed_lines\": {},\n",
        summary.malformed_lines
    ));
    match summary.average_latency_ms {
        Some(value) => output.push_str(&format!("  \"average_latency_ms\": {:.2},\n", value)),
        None => output.push_str("  \"average_latency_ms\": null,\n"),
    }
    output.push_str("  \"method_counts\": ");
    output.push_str(&map_to_json(&summary.method_counts));
    output.push_str(",\n  \"error_counts\": ");
    output.push_str(&map_to_json(&summary.error_counts));
    output.push_str("\n}");
    output
}

fn string_field(line: &str, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| extract_json_string(line, key))
        .map(|text| text.trim().to_ascii_lowercase())
        .filter(|text| !text.is_empty())
}

fn number_field(line: &str, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| extract_json_number(line, key))
        .and_then(|text| text.parse::<f64>().ok())
}

fn extract_json_string(line: &str, key: &str) -> Option<String> {
    let marker = format!("\"{}\"", key);
    let start = line.find(&marker)? + marker.len();
    let after_key = line[start..].trim_start();
    let after_colon = after_key.strip_prefix(':')?.trim_start();
    let value_body = after_colon.strip_prefix('"')?;
    let end = value_body.find('"')?;
    Some(value_body[..end].to_string())
}

fn extract_json_number<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("\"{}\"", key);
    let start = line.find(&marker)? + marker.len();
    let after_key = line[start..].trim_start();
    let after_colon = after_key.strip_prefix(':')?.trim_start();
    let end = after_colon
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-'))
        .unwrap_or(after_colon.len());
    if end == 0 {
        None
    } else {
        Some(&after_colon[..end])
    }
}

fn map_to_json(map: &BTreeMap<String, usize>) -> String {
    let pairs = map
        .iter()
        .map(|(key, value)| format!("\"{}\": {}", escape_json(key), value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{}}}", pairs)
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|extension| extension.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn normalize_status(status: String) -> String {
    match status.as_str() {
        "true" | "1" | "succeeded" | "pass" => "success".to_string(),
        "false" | "0" | "fail" => "failed".to_string(),
        _ => status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_record_fields() {
        let record =
            parse_record(r#"{"result":"succeeded","attack":"bap","duration_ms":12.5}"#).unwrap();
        assert_eq!(record.status.as_deref(), Some("success"));
        assert_eq!(record.method.as_deref(), Some("bap"));
        assert_eq!(record.latency_ms, Some(12.5));
    }
}
