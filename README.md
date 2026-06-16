# exp-audit-rs

`exp-audit-rs` is a Rust command-line tool for auditing research experiment folders. It scans configs, logs, result files, reports, and output artifacts, then produces structured summaries and reproducibility-oriented Markdown reports.

The first target scenario is small VLM safety reproduction work, where experiment configs, JSONL logs, result JSON files, and generated reports are often scattered across directories.

## Features

- Scan an experiment directory and classify artifacts.
- Validate whether key experiment materials are missing.
- Stream-parse JSONL records and tolerate malformed lines.
- Summarize success/failure counts, method distribution, errors, and latency.
- Generate a Markdown audit report.
- Compare two experiment folders.

## Build

This repository pins the Rust toolchain in `rust-toolchain.toml`.

```bash
cargo build
```

On this machine, Windows PowerShell may not expose `cargo`, but WSL has the toolchain:

```bash
wsl.exe bash -lc 'cd /mnt/c/Users/acszd/Documents/Codex/2026-06-15/a-python-c-c-rust-java/outputs/exp-audit-rs && cargo build'
```

## Usage

```bash
cargo run -- scan examples/vlm_safety_run_a
cargo run -- summarize examples/vlm_safety_run_a
cargo run -- validate examples/vlm_safety_run_a
cargo run -- report examples/vlm_safety_run_a --output audit-report.md
cargo run -- diff examples/vlm_safety_run_a examples/vlm_safety_run_b
```

Machine-readable output is available for `scan`, `summarize`, `validate`, and `diff`:

```bash
cargo run -- scan examples/vlm_safety_run_a --json
```

## Example Output

```text
records: 4
success: 2
failed: 1
interrupted_or_unknown: 1
malformed_lines: 1
average_latency_ms: 1426.54
method_counts:
  bap: 4
error_counts:
  policy_refusal: 1
```

## Engineering Notes

The main engineering problem is not model inference itself. The tool addresses the reproducibility problem around experiments: configs, logs, outputs, and reports are easy to lose or mix when experiments are run repeatedly. `exp-audit-rs` creates a deterministic inventory and summary so that an experiment folder can be checked and handed off.

The code is organized as a normal Rust crate:

- `src/scanner.rs`: recursive artifact scanning and classification.
- `src/summary.rs`: streaming JSONL parsing and metric aggregation.
- `src/validate.rs`: completeness and quality checks.
- `src/report.rs`: Markdown report rendering.
- `src/diff.rs`: cross-run comparison.
- `src/main.rs`: CLI interface.

## Project Positioning

This project is suitable to describe as an independently developed Rust utility for research workflow support. It is not a course assignment and does not claim to implement a new VLM attack algorithm. Its contribution is engineering support for experiment auditability and reproducibility.
