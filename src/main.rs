use exp_audit_rs::diff::{diff, render_text};
use exp_audit_rs::report::render_markdown;
use exp_audit_rs::scanner::scan;
use exp_audit_rs::summary::{summarize, to_json as summary_to_json};
use exp_audit_rs::validate::{issues_to_json, validate, Severity};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {}", error);
        process::exit(1);
    }
}

fn run() -> CliResult<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }

    match args[0].as_str() {
        "scan" => {
            // scan 只建立文件清单
            let root = required_path(&args, 1, "scan <root>")?;
            let manifest = scan(root)?;
            if has_flag(&args, "--json") {
                println!("{}", manifest_to_json(&manifest));
            } else {
                print_manifest(&manifest);
            }
        }
        "validate" => {
            // validate 同时使用文件清单和日志统计结果
            let root = required_path(&args, 1, "validate <root>")?;
            let manifest = scan(&root)?;
            let summary = summarize(&root)?;
            let issues = validate(&manifest, &summary);
            if has_flag(&args, "--json") {
                println!("{}", issues_to_json(&issues));
            } else {
                print_issues(&issues);
            }
            if issues.iter().any(|issue| issue.severity == Severity::Error) {
                process::exit(2);
            }
        }
        "summarize" => {
            let root = required_path(&args, 1, "summarize <root>")?;
            let summary = summarize(root)?;
            if has_flag(&args, "--json") {
                println!("{}", summary_to_json(&summary));
            } else {
                print_summary(&summary);
            }
        }
        "report" => {
            let root = required_path(&args, 1, "report <root> [--output <path>]")?;
            let output = output_path(&args).unwrap_or_else(|| PathBuf::from("audit-report.md"));
            let manifest = scan(&root)?;
            let summary = summarize(&root)?;
            let issues = validate(&manifest, &summary);
            let markdown = render_markdown(&manifest, &summary, &issues);
            if let Some(parent) = output.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }
            fs::write(&output, markdown)?;
            println!("wrote {}", output.display());
        }
        "diff" => {
            let left = required_path(&args, 1, "diff <left> <right>")?;
            let right = required_path(&args, 2, "diff <left> <right>")?;
            let left_manifest = scan(&left)?;
            let left_summary = summarize(&left)?;
            let right_manifest = scan(&right)?;
            let right_summary = summarize(&right)?;
            let audit_diff = diff(
                &left_manifest,
                &left_summary,
                &right_manifest,
                &right_summary,
            );
            if has_flag(&args, "--json") {
                println!("{}", exp_audit_rs::diff::to_json(&audit_diff));
            } else {
                print!("{}", render_text(&audit_diff));
            }
        }
        command => {
            return Err(format!("unknown command: {}", command).into());
        }
    }

    Ok(())
}

fn required_path(args: &[String], index: usize, usage: &str) -> CliResult<PathBuf> {
    args.get(index)
        .filter(|value| !value.starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| format!("usage: exp-audit-rs {}", usage).into())
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn output_path(args: &[String]) -> Option<PathBuf> {
    // 同时支持 --output 和 -o
    args.iter()
        .position(|arg| arg == "--output" || arg == "-o")
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
}

fn print_help() {
    println!(
        "\
exp-audit-rs

USAGE:
  exp-audit-rs scan <root> [--json]
  exp-audit-rs summarize <root> [--json]
  exp-audit-rs validate <root> [--json]
  exp-audit-rs report <root> [--output <path>]
  exp-audit-rs diff <left> <right> [--json]
"
    );
}

fn print_manifest(manifest: &exp_audit_rs::artifact::AuditManifest) {
    println!("root: {}", manifest.root.display());
    println!("artifacts: {}", manifest.artifacts.len());
    println!("total_size_bytes: {}", manifest.total_size_bytes());
    for (kind, count) in manifest.count_by_kind() {
        println!("{}: {}", kind.as_str(), count);
    }
}

fn manifest_to_json(manifest: &exp_audit_rs::artifact::AuditManifest) -> String {
    let artifacts = manifest
        .artifacts
        .iter()
        .map(|artifact| {
            format!(
                "    {{\"path\":\"{}\",\"kind\":\"{}\",\"size_bytes\":{}}}",
                escape_json(&artifact.path.display().to_string()),
                artifact.kind.as_str(),
                artifact.size_bytes
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "{{\n  \"root\": \"{}\",\n  \"artifact_count\": {},\n  \"total_size_bytes\": {},\n  \"artifacts\": [\n{}\n  ]\n}}",
        escape_json(&manifest.root.display().to_string()),
        manifest.artifacts.len(),
        manifest.total_size_bytes(),
        artifacts
    )
}

fn print_summary(summary: &exp_audit_rs::summary::ExperimentSummary) {
    println!("records: {}", summary.records);
    println!("success: {}", summary.success);
    println!("failed: {}", summary.failed);
    println!("interrupted_or_unknown: {}", summary.interrupted_or_unknown);
    println!("malformed_lines: {}", summary.malformed_lines);
    if let Some(latency) = summary.average_latency_ms {
        println!("average_latency_ms: {:.2}", latency);
    }
    println!("method_counts:");
    for (method, count) in &summary.method_counts {
        println!("  {}: {}", method, count);
    }
    println!("error_counts:");
    for (error, count) in &summary.error_counts {
        println!("  {}: {}", error, count);
    }
}

fn print_issues(issues: &[exp_audit_rs::validate::ValidationIssue]) {
    if issues.is_empty() {
        println!("no validation issues");
        return;
    }
    for issue in issues {
        println!("{:?}: {}", issue.severity, issue.message);
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
