# Experiment Audit Report

- Generated unix time: `1781538870`
- Experiment root: `/mnt/c/Users/acszd/Documents/Codex/2026-06-15/a-python-c-c-rust-java/outputs/exp-audit-rs/examples/vlm_safety_run_a`
- Artifact count: `4`
- Total size: `794` bytes

## Artifact Inventory

| Kind | Count |
| --- | ---: |
| config | 1 |
| json_log | 1 |
| text_log | 1 |
| result_json | 1 |

## Experiment Summary

- Parsed records: `4`
- Success: `2`
- Failed: `1`
- Interrupted/unknown: `1`
- Malformed log lines: `1`
- Average latency: `1783.17` ms

## Method Distribution

| Method | Records |
| --- | ---: |
| `bap` | 4 |

## Validation Issues

| Severity | Message |
| --- | --- |
| warning | 1 malformed jsonl lines were skipped |
| warning | 1 records have unknown/interrupted status |
| warning | only one or zero methods detected: cross-run comparison may be limited |

## Representative Artifacts

| Path | Kind | Size |
| --- | --- | ---: |
| `configs/bap.yaml` | config | 170 |
| `logs/events.jsonl` | json_log | 366 |
| `logs/run.log` | text_log | 154 |
| `outputs/result.json` | result_json | 104 |
