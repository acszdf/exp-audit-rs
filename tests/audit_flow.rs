use exp_audit_rs::report::render_markdown;
use exp_audit_rs::scanner::scan;
use exp_audit_rs::summary::summarize;
use exp_audit_rs::validate::{validate, Severity};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn audits_a_minimal_experiment_directory() {
    let root = std::env::temp_dir().join(format!(
        "exp-audit-rs-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("configs")).unwrap();
    fs::create_dir_all(root.join("logs")).unwrap();
    fs::create_dir_all(root.join("outputs")).unwrap();
    fs::write(root.join("configs/run.yaml"), "seed: 1\nmethod: bap\n").unwrap();
    fs::write(
        root.join("logs/events.jsonl"),
        r#"{"method":"bap","status":"success","latency_ms":10.0}"#,
    )
    .unwrap();
    fs::write(root.join("outputs/result.json"), r#"{"success":1}"#).unwrap();

    let manifest = scan(&root).unwrap();
    let summary = summarize(&root).unwrap();
    let issues = validate(&manifest, &summary);
    let report = render_markdown(&manifest, &summary, &issues);

    assert_eq!(manifest.artifacts.len(), 3);
    assert_eq!(summary.records, 1);
    assert_eq!(summary.success, 1);
    assert!(!issues.iter().any(|issue| issue.severity == Severity::Error));
    assert!(report.contains("Experiment Audit Report"));

    fs::remove_dir_all(root).unwrap();
}
