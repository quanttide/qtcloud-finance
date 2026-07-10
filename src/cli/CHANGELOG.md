# Changelog

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
