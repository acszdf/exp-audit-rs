# 非 Python 项目说明草稿

以下内容用于回答老师关于非 Python 软件开发经历的问题。请根据真实情况补充 GitHub 链接、项目起止时间和是否已经实际用于你的实验目录。

## A. 是否使用过其它编程语言开发软件

除 Python 外，我使用 Rust 独立开发过一个科研实验目录审计工具 `exp-audit-rs`。该项目不是学校课程作业，而是围绕我在 VLM safety / 多模态模型安全复现实验中遇到的实验资料管理问题开发的工程辅助工具。

## B. 项目列举

### 项目：exp-audit-rs

1. 项目背景和内容

   在做 VLM safety 相关复现和小规模实验时，实验配置、运行日志、结果 JSON、输出图片和阶段报告经常分散在不同目录中。手工检查一次实验是否完整、日志是否中断、结果是否可以复盘比较麻烦，也容易漏掉异常记录。

   因此我用 Rust 开发了 `exp-audit-rs`，它是一个命令行工具，可以扫描实验目录，识别配置文件、JSONL 日志、文本日志、结果 JSON、图片和报告文件；同时可以解析 JSONL 实验记录，统计成功/失败数量、异常行、方法分布、平均耗时，并生成 Markdown 审计报告。工具还支持比较两次实验目录，用于检查两次运行之间的记录数、成功数、失败数和 artifact 类型差异。

2. 本人的开发背景

   该项目来自个人科研复现和导师方向准备过程中的工程需求，主要服务于 VLM safety / 多模态模型安全实验的资料整理与可复现性检查，不属于学校课程作业或课程项目。

3. 本人的角色和参与人数

   项目由我独立开发，共 1 人参与。我负责需求拆分、命令行接口设计、目录扫描、日志解析、统计汇总、报告生成、示例数据和测试用例编写。

4. 代码行数、语言和贡献比例

   项目主要使用 Rust 编写。当前核心 Rust 源码包括：

   - `src/main.rs`：命令行入口和参数解析；
   - `src/scanner.rs`：实验目录递归扫描和 artifact 分类；
   - `src/summary.rs`：JSONL 日志解析和统计汇总；
   - `src/validate.rs`：实验完整性与日志质量校验；
   - `src/report.rs`：Markdown 报告生成；
   - `src/diff.rs`：两次实验目录对比；
   - `tests/audit_flow.rs`：端到端测试。

   项目由我独立完成，因此我对项目代码贡献比例为 100%。实际代码行数可以用以下命令统计：

   ```bash
   find src tests -name '*.rs' -print0 | xargs -0 wc -l
   ```

5. 开源 repo 链接和本人代码部分

   当前项目可以上传到 GitHub。上传后填写：

   - Repo: `https://github.com/<your-name>/exp-audit-rs`
   - 本人代码部分：`src/`、`tests/`、`examples/` 和 `README.md` 均由本人编写。

## C. 技术或工程难点

一个核心难点是：实验日志和结果文件不一定完全规范。真实实验经常会出现 JSONL 中夹杂损坏行、某些记录缺少状态字段、实验中断导致记录不完整、不同方法使用不同字段名记录结果等情况。如果简单地把日志一次性读入内存，或者遇到一行格式错误就终止，工具在实际实验目录中会很脆弱。

我在 `src/summary.rs` 中实现了面向 JSONL 的流式解析和容错汇总逻辑。核心代码约数百行 Rust，主要功能包括：

- 递归查找实验目录中的 `.jsonl` 文件；
- 按行读取日志，避免一次性加载大文件；
- 对每一行提取 `status/result/outcome`、`method/attack/algorithm/strategy`、`error/error_type/exception`、`latency_ms/duration_ms/elapsed_ms/time_ms` 等常见字段；
- 对损坏行只计入 `malformed_lines`，不中断整个审计流程；
- 对 unknown / interrupted 状态单独统计，避免把中断实验误写成成功或失败；
- 计算成功数、失败数、方法分布、错误类型分布和平均耗时。

解决思路是把“实验审计”拆成扫描、解析、校验、报告四个阶段：扫描阶段只建立 artifact inventory；解析阶段尽量容错提取结构化字段；校验阶段给出 error/warning；报告阶段把这些信息稳定输出为 Markdown。这样即使实验数据不完整，工具也能指出具体缺失和异常，而不是直接失败。

最终效果是：对一个实验目录运行

```bash
cargo run -- summarize examples/vlm_safety_run_a
cargo run -- validate examples/vlm_safety_run_a
cargo run -- report examples/vlm_safety_run_a --output generated/audit-run-a.md
```

可以得到记录数、成功/失败数量、异常日志行、未知状态记录、方法分布和可提交的 Markdown 审计报告。这解决的是科研实验资料可追踪、可复盘、可交接的问题。
