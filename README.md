# exp-audit-rs

`exp-audit-rs` 是一个用 Rust 编写的命令行工具，用于审计科研实验目录。它可以扫描配置文件、日志、结果文件、报告和输出 artifact，并生成结构化统计和面向复盘的 Markdown 报告。

项目最初面向小规模 VLM safety 复现实验场景：实验配置、JSONL 日志、结果 JSON 和阶段报告经常分散在不同目录里，后续检查和交接都比较麻烦。

## 功能

- 扫描实验目录，并按类型识别 artifact。
- 检查关键实验材料是否缺失。
- 按行解析 JSONL 记录，并容忍少量损坏日志行。
- 统计成功/失败数量、方法分布、错误类型和耗时。
- 生成 Markdown 审计报告。
- 对比两个实验目录的统计差异。

## 构建

仓库通过 `rust-toolchain.toml` 固定 Rust 工具链版本。

```bash
cargo build
```

在 WSL 中可以使用 Rust 工具链：

```bash
wsl.exe bash -lc 'cd /mnt/c/Users/acszd/Documents/Codex/2026-06-15/a-python-c-c-rust-java/outputs/exp-audit-rs && cargo build'
```

## 使用

```bash
cargo run -- scan examples/vlm_safety_run_a
cargo run -- summarize examples/vlm_safety_run_a
cargo run -- validate examples/vlm_safety_run_a
cargo run -- report examples/vlm_safety_run_a --output audit-report.md
cargo run -- diff examples/vlm_safety_run_a examples/vlm_safety_run_b
```

`scan`、`summarize`、`validate` 和 `diff` 支持机器可读的 JSON 输出：

```bash
cargo run -- scan examples/vlm_safety_run_a --json
```

## 示例输出

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

## 工程说明

这个项目解决的重点是实验资料的可复盘问题。重复跑实验时，配置、日志、输出和报告很容易散落或混在一起。`exp-audit-rs` 会为实验目录生成稳定的文件清单和统计摘要，让一次实验更容易检查、复盘和交接。

代码按普通 Rust crate 组织：

- `src/scanner.rs`：递归扫描实验 artifact 并分类。
- `src/summary.rs`：按行解析 JSONL 日志并聚合指标。
- `src/validate.rs`：检查实验材料完整性和日志质量。
- `src/report.rs`：生成 Markdown 审计报告。
- `src/diff.rs`：对比两次实验目录。
- `src/main.rs`：命令行入口。
