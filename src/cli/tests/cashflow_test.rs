mod common;

use chrono::NaiveDate;
use qtcloud_finance_cli::beancount;
use qtcloud_finance_cli::cashflow;

fn demo_ledger() -> String {
    common::write_beancount(
        r#"
2026-01-05 * "工资"
  Income:Salary    -30000.00 CNY
  Assets:Cash:Checking  30000.00 CNY

2026-01-08 * "房租"
  Expenses:Rent    12000.00 CNY
  Assets:Cash:Checking -12000.00 CNY

2026-01-22 * "信用卡还款"
  Liabilities:CreditCard  3200.00 CNY
  Assets:Cash:Checking   -3200.00 CNY

2026-02-05 * "工资"
  Income:Salary    -30000.00 CNY
  Assets:Cash:Checking  30000.00 CNY

2026-02-08 * "房租"
  Expenses:Rent    12000.00 CNY
  Assets:Cash:Checking -12000.00 CNY

2026-02-20 * "信用卡还款"
  Liabilities:CreditCard  2800.00 CNY
  Assets:Cash:Checking   -2800.00 CNY

2026-03-05 * "工资"
  Income:Salary    -30000.00 CNY
  Assets:Cash:Checking  30000.00 CNY

2026-03-08 * "房租"
  Expenses:Rent    12000.00 CNY
  Assets:Cash:Checking -12000.00 CNY

2026-03-22 * "信用卡还款"
  Liabilities:CreditCard  3200.00 CNY
  Assets:Cash:Checking   -3200.00 CNY
"#,
    )
}

#[test]
fn test_cashflow_quarterly() {
    let path = demo_ledger();
    let txs = beancount::parse(&path).unwrap();
    common::cleanup(&path);

    let report = cashflow::compute_cashflow(
        &txs,
        cashflow::Period::Quarter,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
    );

    // 两个资产/负债账户
    assert_eq!(report.accounts.len(), 2);

    let checking = report
        .accounts
        .iter()
        .find(|a| a.account == "Assets:Cash:Checking")
        .unwrap();
    let credit = report
        .accounts
        .iter()
        .find(|a| a.account == "Liabilities:CreditCard")
        .unwrap();

    // Checking: 流入 90000(3×工资), 流出 45200(3×12000+3200+2800+3200)
    assert_eq!(checking.periods[0].inflow as i32, 90000);
    assert_eq!(checking.periods[0].outflow as i32, 45200);
    assert_eq!(checking.periods[0].net as i32, 44800);

    // CreditCard: 还债 9200(3200+2800+3200) = 流出
    assert_eq!(credit.periods[0].outflow as i32, 9200);
    assert_eq!(credit.periods[0].inflow as i32, 0);

    // 净现金流: 44800 - 9200 = 35600
    assert_eq!(report.net_totals[0] as i32, 35600);
}

#[test]
fn test_cashflow_monthly_in_two_periods() {
    let path = demo_ledger();
    let txs = beancount::parse(&path).unwrap();
    common::cleanup(&path);

    // 自定义整个季度，验证 generate_periods 只产生一个 custom 区间
    let report = cashflow::compute_cashflow(
        &txs,
        cashflow::Period::Month,
        // 只看 2 月
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
    );

    let checking = report
        .accounts
        .iter()
        .find(|a| a.account == "Assets:Cash:Checking")
        .unwrap();
    assert_eq!(checking.periods[0].inflow as i32, 30000);
    assert_eq!(checking.periods[0].outflow as i32, 14800);
    assert_eq!(checking.periods[0].net as i32, 15200);
}

#[test]
fn test_forecast_with_recurring_income() {
    let path = demo_ledger();
    let txs = beancount::parse(&path).unwrap();
    common::cleanup(&path);

    let report = cashflow::compute_forecast(
        &txs,
        3,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
    );

    // 预期: Checking(工资) + CreditCard(还款) 两个规律账户
    let checking = report
        .rows
        .iter()
        .find(|r| r.account == "Assets:Cash:Checking");
    // 预期: CreditCard 还款是规律性最强的账户（每月约 3000）
    let credit = report
        .rows
        .iter()
        .find(|r| r.account == "Liabilities:CreditCard");
    assert!(credit.is_some(), "CreditCard 应有预测");

    // Checking 有混合的流入(30000)和流出(12000)，标准差大，不会被检测为规律
    let checking = report
        .rows
        .iter()
        .find(|r| r.account == "Assets:Cash:Checking");
    assert!(checking.is_none(), "Checking 混合收支不应被检测为规律");

    assert_eq!(credit.as_ref().unwrap().periods.len(), 3, "应预测 3 个月");
}

#[test]
fn test_cashflow_json_format() {
    let path = demo_ledger();
    let txs = beancount::parse(&path).unwrap();
    common::cleanup(&path);

    let report = cashflow::compute_cashflow(
        &txs,
        cashflow::Period::Month,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
    );

    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("Assets:Cash:Checking"));
    assert!(json.contains("inflow"));
    assert!(json.contains("outflow"));
    assert!(json.contains("net"));
}

#[test]
fn test_cashflow_no_matching_accounts() {
    let content = r#"
2026-01-05 * "咨询"
  Income:Consulting  -5000.00 CNY
  Expenses:Office      500.00 CNY
  Expenses:Equipment  4500.00 CNY
"#;
    let path = common::write_beancount(content);
    let txs = beancount::parse(&path).unwrap();
    common::cleanup(&path);

    let report = cashflow::compute_cashflow(
        &txs,
        cashflow::Period::Month,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
    );

    // 只有收入/费用，没有资产/负债，所以没有 cashflow 账户
    assert_eq!(report.accounts.len(), 0);
    assert_eq!(report.net_totals, vec![0.0]);
}
