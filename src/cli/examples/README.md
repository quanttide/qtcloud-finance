# 演示案例

使用 `demo.beancount`（2026 上半年自由职业者收支）展示 `cashflow` 功能。

## 前提

在项目根目录运行所有命令：

```bash
cd qtcloud-finance/apps/qtcloud-finance/src/cli
```

或指定 `--manifest-path`：

```bash
cargo run --manifest-path apps/qtcloud-finance/src/cli/Cargo.toml -- cashflow ...
```

## 现金流状况

### 自定义期间汇总（推荐）

查看半年整体情况：

```bash
cargo run -- cashflow status examples/demo.beancount -p custom:2026-01-01..2026-06-30
```

输出：

```
  --- 2026-01-01~2026-06-30 ---
+----------------------------+------------+------------+------------+
| 账户                       | 流入       | 流出       | 净额       |
+----------------------------+------------+------------+------------+
| Assets:Cash:Checking       |     181000 |     149570 |     +31430 |
+----------------------------+------------+------------+------------+
| Assets:Cash:Savings        |       6500 |          0 |      +6500 |
+----------------------------+------------+------------+------------+
| Liabilities:CreditCard:BOC |          0 |      18700 |     -18700 |
+----------------------------+------------+------------+------------+
  净现金流: +19230
```

### JSON 输出

```bash
cargo run -- cashflow status examples/demo.beancount -p custom:2026-01-01..2026-06-30 -f json
```

### CSV 导出

```bash
cargo run -- cashflow status examples/demo.beancount -p custom:2026-01-01..2026-06-30 -f csv
```

## 现金流预测

基于过去 12 个月的历史规律，预测未来 3 个月：

```bash
cargo run -- cashflow forecast examples/demo.beancount
```

预测未来 6 个月：

```bash
cargo run -- cashflow forecast examples/demo.beancount -n 6
```

JSON 格式：

```bash
cargo run -- cashflow forecast examples/demo.beancount -f json
```
