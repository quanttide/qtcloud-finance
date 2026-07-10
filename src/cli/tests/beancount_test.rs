mod common;

/// 集成测试：从真实文件解析 Beancount 账本
#[test]
fn test_parse_full_ledger() {
    let content = r#"; 账本示例
option "operating_currency" "CNY"

2026-01-05 * "Freelance" "网站项目"
  Income:Freelance:Web    -35000.00 CNY
  Assets:Cash:Checking     35000.00 CNY

2026-01-08 * "房租"
  Expenses:Housing:Rent    12000.00 CNY
  Assets:Cash:Checking    -12000.00 CNY

2026-01-10 * "超市"
  Expenses:Food:Grocery     1280.50 CNY
  Assets:Cash:Checking     -1280.50 CNY
"#;
    let path = common::write_beancount(content);
    let txs = qtcloud_finance_cli::beancount::parse(&path).unwrap();
    common::cleanup(&path);

    assert_eq!(txs.len(), 3, "应解析出 3 笔交易");

    // 第一笔：收入
    assert_eq!(txs[0].date.to_string(), "2026-01-05");
    assert_eq!(txs[0].postings.len(), 2);

    // 第二笔：房租支出
    assert_eq!(txs[1].description, "房租");
    assert_eq!(txs[1].postings[0].account, "Expenses:Housing:Rent");
    assert_eq!(txs[1].postings[0].amount, 12000.0);
    assert_eq!(txs[1].postings[1].account, "Assets:Cash:Checking");
    assert_eq!(txs[1].postings[1].amount, -12000.0);

    // 第三笔：超市
    assert_eq!(txs[2].description, "超市");
    assert_eq!(txs[2].postings[1].amount, -1280.50);
}

#[test]
fn test_parse_large_ledger() {
    // 模拟 100 笔交易
    let mut lines = String::from("option \"operating_currency\" \"CNY\"\n");
    for i in 1..=100 {
        let day = (i % 28) + 1;
        lines.push_str(&format!(
            "2026-{:02}-{:02} * \"交易{}\"\n",
            (i % 12) + 1,
            day,
            i
        ));
        lines.push_str(&format!("  Income:Test    -{}.00 CNY\n", i * 100));
        lines.push_str(&format!("  Assets:Cash:Checking  {}.00 CNY\n", i * 100));
    }

    let path = common::write_beancount(&lines);
    let txs = qtcloud_finance_cli::beancount::parse(&path).unwrap();
    common::cleanup(&path);

    assert_eq!(txs.len(), 100);
    assert_eq!(txs[0].postings.len(), 2);
    assert_eq!(txs[99].description, "交易100");
}

#[test]
fn test_parse_file_not_found() {
    let result = qtcloud_finance_cli::beancount::parse("/tmp/__nonexistent_beancount__");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("无法读取文件"));
}

#[test]
fn test_parse_empty_file() {
    let path = common::write_beancount("");
    let txs = qtcloud_finance_cli::beancount::parse(&path).unwrap();
    common::cleanup(&path);
    assert!(txs.is_empty());
}

#[test]
fn test_parse_complex_postings() {
    let content = r#"
2026-03-10 * "房贷"
  Expenses:Housing:Mortgage  5800.00 CNY
  Assets:Cash:Checking      -5800.00 CNY
  Liabilities:Loan:Mortgage   5800.00 CNY

2026-03-12 * "分红"
  Income:Investment:Dividend  -1500.00 CNY
  Assets:Cash:Savings          1500.00 CNY
"#;
    let path = common::write_beancount(content);
    let txs = qtcloud_finance_cli::beancount::parse(&path).unwrap();
    common::cleanup(&path);

    assert_eq!(txs.len(), 2);

    // 房贷：三分账
    assert_eq!(txs[0].postings.len(), 3);
    assert_eq!(txs[0].postings[2].account, "Liabilities:Loan:Mortgage");

    // 分红
    assert_eq!(txs[1].postings[1].account, "Assets:Cash:Savings");
    assert_eq!(txs[1].postings[1].amount, 1500.0);
}
