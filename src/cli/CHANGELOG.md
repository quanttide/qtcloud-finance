# Changelog

## [0.0.2] - 2026-07-11

### Added
- `cashflow simulate` 命令：假设场景推演（报表层算术，不虚构账户）
  - `-a, --adjust` 调整项：`ACCOUNT@AMOUNT`，支持百分比（`-20%`）和绝对值（`1000`）
  - `-o, --one-time` 一次性事件：`DATE@AMOUNT`，金额即现金影响
  - 输出基线/推演/变化三列对比表，支持 table/json 格式

### Changed
- `simulate` 从交易层重构为报表层，去掉虚拟账户和手动调平
- 参数简化：去掉未使用的 `pattern` 字段和 `account` 字段

## [0.0.1] - 2026-07-11

### Changed
- 项目从 Python 迁移到 Rust（edition 2024）
- 命令名从 `qtcloud-finance`（uv run）改为原生 Rust 二进制

### Added
- `cashflow status` 命令：Beancount 现金流状况查询（支持 week/month/quarter/year/custom 期间，table/csv/json 输出）
- `cashflow forecast` 命令：基于周期性检测的现金流预测（recurring/trend 方法）
- `cashflow summary` 命令：预留汇总入口
- beancount 模块：最小 Beancount 解析器
- 示例账本 `examples/demo.beancount`（6 个月自由职业者收支）
- 单元测试 27 个 + 集成测试 10 个

### Removed
- Python 源码（已归档到 data/archive）
- TUI 界面（Textual）
- LLM 智能记账/对账功能（后续以独立模块重新设计）
- `scripts/run-cli.sh`

## [0.0.1-alpha.1] - 2026-03-30

### Added
- config 模块：声明式配置（Pydantic Settings）
- bookkeeper 模块：Beancount 智能记账（LLM 生成 + 语法验证）
- reconciler 模块：智能对账（自动匹配 + LLM 分析）
- tui 模块：Textual TUI 界面（对话式记账）
- 单元测试（29 个测试用例）
