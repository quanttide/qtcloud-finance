use chrono::NaiveDate;
use std::fs;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub date: NaiveDate,
    pub description: String,
    pub postings: Vec<Posting>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Posting {
    pub account: String,
    pub amount: f64,
}

/// 从文件路径解析 Beancount 账本。
pub fn parse(path: &str) -> Result<Vec<Transaction>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("无法读取文件 '{}': {}", path, e))?;
    parse_str(&content)
}

/// 从字符串解析 Beancount 账本（便于测试和内联数据）。
pub fn parse_str(content: &str) -> Result<Vec<Transaction>, String> {
    let mut transactions = Vec::new();
    let mut current_tx: Option<Transaction> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('*') {
            continue;
        }

        if trimmed.starts_with("option ")
            || trimmed.starts_with("plugin ")
            || trimmed.starts_with("include ")
        {
            continue;
        }

        if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
            let date_str = &trimmed[..10.min(trimmed.len())];
            if date_str.len() == 10 && date_str.chars().filter(|&c| c == '-').count() == 2 {
                let rest = trimmed[10..].trim();
                if rest.starts_with('*') || rest.starts_with('!') {
                    if let Some(tx) = current_tx.take() {
                        if !tx.postings.is_empty() {
                            transactions.push(tx);
                        }
                    }

                    // 提取引号内的描述: 支持 "Payee" "Narration" 或仅 "Narration"
                    let desc = rest[1..].trim();
                    let desc = desc
                        .split('"')
                        .filter(|s| !s.is_empty() && *s != " ")
                        .collect::<Vec<_>>()
                        .join(" ");
                    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        .map_err(|e| format!("日期解析失败 '{}': {}", date_str, e))?;
                    current_tx = Some(Transaction {
                        date,
                        description: desc,
                        postings: Vec::new(),
                    });
                }
            }
        } else if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(tx) = &mut current_tx {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let account = parts[0].to_string();
                    if let Some(amount_str) = parts.get(1) {
                        if let Ok(amount) = amount_str.parse::<f64>() {
                            tx.postings.push(Posting { account, amount });
                        }
                    }
                }
            }
        }
    }

    if let Some(tx) = current_tx {
        if !tx.postings.is_empty() {
            transactions.push(tx);
        }
    }

    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(parse_str("").unwrap(), vec![]);
    }

    #[test]
    fn test_comments_and_options() {
        let input = r#";
; 注释行
* 也是注释
option "operating_currency" "CNY"
plugin "beancount.plugins.implicit_prices"
"#;
        assert_eq!(parse_str(input).unwrap(), vec![]);
    }

    #[test]
    fn test_single_transaction() {
        let input = r#"2026-01-05 * "Freelance" "项目款"
  Income:Freelance:Web  -35000.00 CNY
  Assets:Cash:Checking   35000.00 CNY
"#;
        let txs = parse_str(input).unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].date.to_string(), "2026-01-05");
        assert_eq!(txs[0].description, "Freelance 项目款");
        assert_eq!(txs[0].postings.len(), 2);
        assert_eq!(txs[0].postings[0].account, "Income:Freelance:Web");
        assert_eq!(txs[0].postings[0].amount, -35000.0);
        assert_eq!(txs[0].postings[1].account, "Assets:Cash:Checking");
        assert_eq!(txs[0].postings[1].amount, 35000.0);
    }

    #[test]
    fn test_multiple_transactions() {
        let input = r#"2026-01-08 * "房租"
  Expenses:Housing:Rent  12000.00 CNY
  Assets:Cash:Checking  -12000.00 CNY

2026-01-10 * "超市"
  Expenses:Food:Grocery  1280.50 CNY
  Assets:Cash:Checking  -1280.50 CNY
"#;
        let txs = parse_str(input).unwrap();
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[1].description, "超市");
    }

    #[test]
    fn test_tab_indented_postings() {
        let input = "2026-01-05 * \"测试\"
\tAssets:Cash:Checking  100.00 CNY
\tIncome:Test  -100.00 CNY";
        let txs = parse_str(input).unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].postings.len(), 2);
    }

    #[test]
    fn test_skip_posting_without_amount() {
        let input = r#"2026-01-05 * "测试"
  Assets:Cash:Checking  100.00 CNY
  Equity:Opening-Balances
"#;
        let txs = parse_str(input).unwrap();
        assert_eq!(txs.len(), 1);
        // 只有一条有金额的分账被解析
        assert_eq!(txs[0].postings.len(), 1);
    }

    #[test]
    fn test_malformed_date() {
        let input = "not-a-date * \"test\"\n  A: B  1.00";
        let result = parse_str(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_txn_with_flag_bang() {
        let input = r#"2026-01-05 ! "已核对"
  Assets:Cash:Checking  100.00 CNY
  Income:Test  -100.00 CNY
"#;
        let txs = parse_str(input).unwrap();
        assert_eq!(txs.len(), 1);
    }
}
