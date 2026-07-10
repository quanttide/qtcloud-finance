use crate::beancount::Transaction;
use chrono::{Datelike, NaiveDate};
use serde::Serialize;
use std::collections::BTreeMap;
use tabled::{Table, Tabled};

#[derive(Debug, Serialize, Tabled)]
pub struct PeriodRow {
    pub account: String,
    #[tabled(rename = "净额")]
    pub net: String,
    #[tabled(skip)]
    pub periods: Vec<PeriodValue>,
}

#[derive(Debug, Serialize)]
pub struct PeriodValue {
    pub label: String,
    pub inflow: f64,
    pub outflow: f64,
    pub net: f64,
}

#[derive(Debug, Serialize)]
pub struct CashflowReport {
    pub period_label: String,
    pub start: String,
    pub end: String,
    pub accounts: Vec<PeriodRow>,
    pub net_totals: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct ForecastRow {
    pub account: String,
    pub pattern: String,
    pub confidence: String,
    pub periods: Vec<PeriodValue>,
}

#[derive(Debug, Serialize)]
pub struct ForecastReport {
    pub period_label: String,
    pub start: String,
    pub end: String,
    pub rows: Vec<ForecastRow>,
    pub net_totals: Vec<f64>,
}

// ─── 期间工具 ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum Period {
    Week,
    Month,
    Quarter,
    Year,
    Custom(NaiveDate, NaiveDate),
}

pub fn parse_period(
    period_str: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<(Period, NaiveDate, NaiveDate), String> {
    if let Some(custom) = period_str.strip_prefix("custom:") {
        let parts: Vec<&str> = custom.split("..").collect();
        if parts.len() != 2 {
            return Err("custom 格式: custom:YYYY-MM-DD..YYYY-MM-DD".into());
        }
        let s = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
            .map_err(|e| format!("起始日期: {}", e))?;
        let e = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d")
            .map_err(|e| format!("截止日期: {}", e))?;
        return Ok((Period::Custom(s, e), s, e));
    }

    let today = chrono::Local::now().date_naive();
    let start = start
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(today);
    let _end = end
        .and_then(|e| NaiveDate::parse_from_str(e, "%Y-%m-%d").ok())
        .unwrap_or(today);

    match period_str {
        "week" => {
            let weekday = start.weekday().num_days_from_monday();
            let week_start = start - chrono::Duration::days(weekday as i64);
            let week_end = week_start + chrono::Duration::days(6);
            Ok((Period::Week, week_start, week_end))
        }
        "month" => {
            let month_start = NaiveDate::from_ymd_opt(start.year(), start.month(), 1).unwrap();
            let month_end = if start.month() == 12 {
                NaiveDate::from_ymd_opt(start.year() + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(start.year(), start.month() + 1, 1).unwrap()
            } - chrono::Duration::days(1);
            Ok((Period::Month, month_start, month_end))
        }
        "quarter" => {
            let q_start_month = (start.month() - 1) / 3 * 3 + 1;
            let q_start = NaiveDate::from_ymd_opt(start.year(), q_start_month, 1).unwrap();
            let q_end_month = if q_start_month + 3 > 12 {
                12
            } else {
                q_start_month + 3
            };
            let q_end = if q_end_month == 12 {
                NaiveDate::from_ymd_opt(start.year() + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(start.year(), q_end_month, 1).unwrap()
            } - chrono::Duration::days(1);
            Ok((Period::Quarter, q_start, q_end))
        }
        "year" => {
            let y_start = NaiveDate::from_ymd_opt(start.year(), 1, 1).unwrap();
            let y_end = NaiveDate::from_ymd_opt(start.year() + 1, 1, 1).unwrap()
                - chrono::Duration::days(1);
            Ok((Period::Year, y_start, y_end))
        }
        _ => Err(format!(
            "不认识的期间: '{}'。可选: week, month, quarter, year, custom:start..end",
            period_str
        )),
    }
}

pub fn period_label(p: Period, idx: u32) -> String {
    match p {
        Period::Week => format!("第{}周", idx + 1),
        Period::Month => {
            let months = [
                "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月",
                "12月",
            ];
            months[(idx as usize) % 12].to_string()
        }
        Period::Quarter => format!("Q{}", idx % 4 + 1),
        Period::Year => format!("第{}年", idx + 1),
        Period::Custom(s, e) => format!("{}~{}", s, e),
    }
}

pub fn generate_periods(p: Period, start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut cur = start;
    loop {
        if cur > end {
            break;
        }
        dates.push(cur);
        cur = match p {
            Period::Week => cur + chrono::Duration::days(7),
            Period::Month => {
                if cur.month() == 12 {
                    NaiveDate::from_ymd_opt(cur.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(cur.year(), cur.month() + 1, 1).unwrap()
                }
            }
            Period::Quarter => {
                let m = cur.month() + 3;
                if m > 12 {
                    NaiveDate::from_ymd_opt(cur.year() + 1, m - 12, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(cur.year(), m, 1).unwrap()
                }
            }
            Period::Year => NaiveDate::from_ymd_opt(cur.year() + 1, 1, 1).unwrap(),
            Period::Custom(_, _) => end + chrono::Duration::days(1), // only one period
        };
    }
    dates
}

fn period_end(p: Period, start: NaiveDate) -> NaiveDate {
    match p {
        Period::Week => start + chrono::Duration::days(6),
        Period::Month => {
            let next = if start.month() == 12 {
                NaiveDate::from_ymd_opt(start.year() + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(start.year(), start.month() + 1, 1).unwrap()
            };
            next - chrono::Duration::days(1)
        }
        Period::Quarter => {
            let m = start.month() + 3;
            let (y, m) = if m > 12 {
                (start.year() + 1, m - 12)
            } else {
                (start.year(), m)
            };
            NaiveDate::from_ymd_opt(y, m, 1).unwrap() - chrono::Duration::days(1)
        }
        Period::Year => {
            NaiveDate::from_ymd_opt(start.year() + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        }
        Period::Custom(_, e) => e,
    }
}

// ─── 核心逻辑 ──────────────────────────────────────────────

fn is_cash_or_liability(account: &str) -> bool {
    account.starts_with("Assets:") || account.starts_with("Liabilities:")
}

/// 从交易中提取每个现金/负债账户在每个期间的汇总
pub fn compute_cashflow(
    transactions: &[Transaction],
    period: Period,
    start: NaiveDate,
    end: NaiveDate,
) -> CashflowReport {
    let period_starts = generate_periods(period, start, end);
    let _label_base = period_label(period, 0);

    // 账户 -> 期间索引 -> (inflow, outflow)
    let mut account_data: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();

    for tx in transactions {
        if tx.date < start || tx.date > end {
            continue;
        }

        // 找到所属期间
        let mut period_idx = None;
        for (i, &ps) in period_starts.iter().enumerate() {
            let pe = period_end(period, ps);
            if tx.date >= ps && tx.date <= pe {
                period_idx = Some(i);
                break;
            }
        }
        let Some(idx) = period_idx else { continue };

        for posting in &tx.postings {
            if !is_cash_or_liability(&posting.account) {
                continue;
            }

            let entry = account_data
                .entry(posting.account.clone())
                .or_insert_with(|| vec![(0.0, 0.0); period_starts.len()]);
            if entry.len() <= idx {
                entry.resize(period_starts.len(), (0.0, 0.0));
            }

            let is_liability = posting.account.starts_with("Liabilities:");
            if is_liability {
                // 负债：正值 = 还债(流出), 负值 = 举债(流入)
                if posting.amount >= 0.0 {
                    entry[idx].1 += posting.amount; // outflow
                } else {
                    entry[idx].0 += -posting.amount; // inflow
                }
            } else {
                // 资产：正值 = 流入, 负值 = 流出
                if posting.amount >= 0.0 {
                    entry[idx].0 += posting.amount; // inflow
                } else {
                    entry[idx].1 += -posting.amount; // outflow (positive)
                }
            }
        }
    }

    let mut accounts = Vec::new();
    let mut net_totals = vec![0.0_f64; period_starts.len()];

    for (account, vals) in &account_data {
        let mut periods = Vec::new();
        for (i, &(inflow, outflow)) in vals.iter().enumerate() {
            let net = inflow - outflow;
            periods.push(PeriodValue {
                label: if i < period_starts.len() {
                    period_label(period, i as u32)
                } else {
                    String::new()
                },
                inflow,
                outflow,
                net,
            });
            net_totals[i] += net;
        }
        let total_net: f64 = periods.iter().map(|p| p.net).sum();
        accounts.push(PeriodRow {
            account: account.clone(),
            net: format!("{:+.0}", total_net),
            periods,
        });
    }

    CashflowReport {
        period_label: format!("{:?} {}", period, start.format("%Y-%m")),
        start: start.to_string(),
        end: end.to_string(),
        accounts,
        net_totals,
    }
}

// ─── 预测 ──────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug)]
struct Pattern {
    account: String,
    label: String,
    avg_amount: f64,
    day_of_month: u32,
    std_dev: f64,
    count: u32,
}

fn detect_patterns(transactions: &[Transaction], start: NaiveDate, end: NaiveDate) -> Vec<Pattern> {
    // 按账户聚合分账
    let mut account_postings: BTreeMap<String, Vec<(NaiveDate, f64)>> = BTreeMap::new();

    for tx in transactions {
        if tx.date < start || tx.date > end {
            continue;
        }
        for p in &tx.postings {
            if !is_cash_or_liability(&p.account) {
                continue;
            }
            account_postings
                .entry(p.account.clone())
                .or_default()
                .push((tx.date, p.amount));
        }
    }

    let mut patterns = Vec::new();

    for (account, entries) in &account_postings {
        if entries.len() < 2 {
            continue;
        }

        // 按日期排序
        let mut sorted = entries.clone();
        sorted.sort_by_key(|(d, _)| *d);

        // 计算平均金额
        let amounts: Vec<f64> = sorted.iter().map(|(_, a)| a.abs()).collect();
        let avg: f64 = amounts.iter().sum::<f64>() / amounts.len() as f64;
        if avg < 0.01 {
            continue;
        }

        // 标准差
        let variance: f64 =
            amounts.iter().map(|a| (a - avg).powi(2)).sum::<f64>() / amounts.len() as f64;
        let std_dev = variance.sqrt();

        // 判断是否规律：标准差 < 平均值的 30%
        if std_dev > avg * 0.3 {
            continue;
        }

        // 常用的日期
        let days: Vec<u32> = sorted.iter().map(|(d, _)| d.day()).collect();
        let common_day = most_common(&days);

        // 判断流入流出
        let net_avg: f64 = sorted.iter().map(|(_, a)| a).sum::<f64>() / sorted.len() as f64;

        let label = if net_avg >= 0.0 {
            format!("每月{}日流入~{:.0}", common_day, avg)
        } else {
            format!("每月{}日流出~{:.0}", common_day, avg)
        };

        patterns.push(Pattern {
            account: account.clone(),
            label,
            avg_amount: avg,
            day_of_month: common_day,
            std_dev,
            count: entries.len() as u32,
        });
    }

    patterns
}

fn most_common(items: &[u32]) -> u32 {
    let mut counts: BTreeMap<u32, u32> = BTreeMap::new();
    for &item in items {
        *counts.entry(item).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .map(|(v, _)| v)
        .unwrap_or(1)
}

pub fn compute_forecast(
    transactions: &[Transaction],
    periods: u32,
    start: NaiveDate,
    end: NaiveDate,
) -> ForecastReport {
    let patterns = detect_patterns(transactions, start, end);

    // 预测的期间标签
    let month_labels = [
        "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
    ];
    let forecast_months: Vec<String> = (0..periods)
        .map(|i| {
            let m = (end.month() as u32 + i) % 12;
            let m = if m == 0 { 12 } else { m };
            format!("{}(预)", month_labels[(m - 1) as usize])
        })
        .collect();

    let mut rows = Vec::new();
    let mut net_totals = vec![0.0; periods as usize];

    for pat in &patterns {
        let mut period_values = Vec::new();
        for i in 0..periods {
            let net = if pat.avg_amount > 0.0 {
                pat.avg_amount
            } else {
                -pat.avg_amount
            };
            period_values.push(PeriodValue {
                label: if (i as usize) < forecast_months.len() {
                    forecast_months[i as usize].clone()
                } else {
                    String::new()
                },
                inflow: if net > 0.0 { net } else { 0.0 },
                outflow: if net < 0.0 { -net } else { 0.0 },
                net,
            });
            net_totals[i as usize] += net;
        }

        let confidence = if pat.std_dev < pat.avg_amount * 0.1 {
            "高".into()
        } else if pat.std_dev < pat.avg_amount * 0.2 {
            "中".into()
        } else {
            "低".into()
        };

        rows.push(ForecastRow {
            account: pat.account.clone(),
            pattern: pat.label.clone(),
            confidence,
            periods: period_values,
        });
    }

    // 按置信度排序
    rows.sort_by(|a, b| b.confidence.cmp(&a.confidence));

    ForecastReport {
        period_label: "预测".into(),
        start: start.to_string(),
        end: end.to_string(),
        rows,
        net_totals,
    }
}

// ─── 输出格式化 ────────────────────────────────────────────

pub fn print_status_table(report: &CashflowReport, format: &str) {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(report).unwrap());
        }
        "csv" => {
            println!("account,period,inflow,outflow,net");
            for row in &report.accounts {
                for p in &row.periods {
                    println!(
                        "{},{},{},{},{}",
                        row.account, p.label, p.inflow, p.outflow, p.net
                    );
                }
            }
        }
        _ => {
            // 构建 tabled 表格
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "账户")]
                account: String,
                #[tabled(rename = "流入")]
                inflow: String,
                #[tabled(rename = "流出")]
                outflow: String,
                #[tabled(rename = "净额")]
                net: String,
            }

            // 一期一表，简单清晰
            for (i, ps) in report
                .accounts
                .first()
                .map(|a| a.periods.iter().enumerate())
                .into_iter()
                .flatten()
            {
                let label = &ps.label;
                println!("\n  --- {} ---", label);

                let mut rows = Vec::new();
                for acc in &report.accounts {
                    if i < acc.periods.len() {
                        let p = &acc.periods[i];
                        if p.inflow == 0.0 && p.outflow == 0.0 {
                            continue;
                        }
                        rows.push(Row {
                            account: acc.account.clone(),
                            inflow: format!("{:>10.0}", p.inflow),
                            outflow: format!("{:>10.0}", p.outflow),
                            net: format!("{:>+10.0}", p.net),
                        });
                    }
                }

                if !rows.is_empty() {
                    println!("{}", Table::new(rows));
                }

                // 净现金流行
                println!("  净现金流: {:+.0}", report.net_totals[i]);
            }
            println!();
        }
    }
}

pub fn print_forecast_table(report: &ForecastReport, format: &str) {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(report).unwrap());
        }
        "csv" => {
            println!("account,pattern,confidence,period,net");
            for row in &report.rows {
                for p in &row.periods {
                    println!(
                        "{},{},{},{},{}",
                        row.account, row.pattern, row.confidence, p.label, p.net
                    );
                }
            }
        }
        _ => {
            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "账户")]
                account: String,
                #[tabled(rename = "规律")]
                pattern: String,
                #[tabled(rename = "可信度")]
                confidence: String,
                #[tabled(rename = "预测净额")]
                net: String,
            }

            for (i, ps) in report
                .rows
                .first()
                .map(|a| a.periods.iter().enumerate())
                .into_iter()
                .flatten()
            {
                println!("\n  --- {} ---", ps.label);

                let mut rows = Vec::new();
                for acc in &report.rows {
                    if i < acc.periods.len() {
                        let p = &acc.periods[i];
                        if p.net == 0.0 {
                            continue;
                        }
                        rows.push(Row {
                            account: acc.account.clone(),
                            pattern: acc.pattern.clone(),
                            confidence: acc.confidence.clone(),
                            net: format!("{:+.0}", p.net),
                        });
                    }
                }

                if !rows.is_empty() {
                    println!("{}", Table::new(rows));
                }

                println!("  预测净现金流: {:+.0}", report.net_totals[i]);
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beancount::Posting;

    // ── is_cash_or_liability ────────────────────────────

    #[test]
    fn test_cash_account() {
        assert!(is_cash_or_liability("Assets:Cash:Checking"));
    }

    #[test]
    fn test_liability_account() {
        assert!(is_cash_or_liability("Liabilities:CreditCard"));
    }

    #[test]
    fn test_income_account() {
        assert!(!is_cash_or_liability("Income:Freelance"));
    }

    #[test]
    fn test_expense_account() {
        assert!(!is_cash_or_liability("Expenses:Food"));
    }

    #[test]
    fn test_equity_account() {
        assert!(!is_cash_or_liability("Equity:Opening"));
    }

    // ── parse_period ────────────────────────────────────

    #[test]
    fn test_parse_period_month() {
        let (p, s, e) = parse_period("month", Some("2026-06-15"), None).unwrap();
        assert!(matches!(p, Period::Month));
        assert_eq!(s.to_string(), "2026-06-01");
        assert_eq!(e.to_string(), "2026-06-30");
    }

    #[test]
    fn test_parse_period_quarter() {
        let (p, s, e) = parse_period("quarter", Some("2026-05-01"), None).unwrap();
        assert!(matches!(p, Period::Quarter));
        assert_eq!(s.to_string(), "2026-04-01");
        assert_eq!(e.to_string(), "2026-06-30");
    }

    #[test]
    fn test_parse_period_year() {
        let (p, s, e) = parse_period("year", Some("2026-07-01"), None).unwrap();
        assert!(matches!(p, Period::Year));
        assert_eq!(s.to_string(), "2026-01-01");
        assert_eq!(e.to_string(), "2026-12-31");
    }

    #[test]
    fn test_parse_period_custom() {
        let (p, s, e) = parse_period("custom:2026-01-01..2026-06-30", None, None).unwrap();
        assert!(matches!(p, Period::Custom(_, _)));
        assert_eq!(s.to_string(), "2026-01-01");
        assert_eq!(e.to_string(), "2026-06-30");
    }

    #[test]
    fn test_parse_period_unknown() {
        let result = parse_period("decade", None, None);
        assert!(result.is_err());
    }

    // ── compute_cashflow ────────────────────────────────

    fn make_tx(date: &str, desc: &str, postings: Vec<(&str, f64)>) -> Transaction {
        Transaction {
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            description: desc.into(),
            postings: postings
                .into_iter()
                .map(|(a, amt)| Posting {
                    account: a.into(),
                    amount: amt,
                })
                .collect(),
        }
    }

    #[test]
    fn test_compute_cashflow_empty() {
        let report = compute_cashflow(
            &[],
            Period::Month,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );
        assert_eq!(report.accounts.len(), 0);
        assert_eq!(report.net_totals, vec![0.0]);
    }

    #[test]
    fn test_compute_cashflow_single_period() {
        let txs = vec![make_tx(
            "2026-01-15",
            "工资",
            vec![
                ("Income:Salary", -30000.0),
                ("Assets:Cash:Checking", 30000.0),
            ],
        )];
        let report = compute_cashflow(
            &txs,
            Period::Month,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );
        assert_eq!(report.accounts.len(), 1);
        assert_eq!(report.accounts[0].account, "Assets:Cash:Checking");
        assert_eq!(report.accounts[0].periods[0].inflow as i32, 30000);
        assert_eq!(report.accounts[0].periods[0].outflow as i32, 0);
    }

    #[test]
    fn test_compute_cashflow_inflow_outflow() {
        let txs = vec![
            make_tx(
                "2026-01-05",
                "工资",
                vec![
                    ("Income:Salary", -30000.0),
                    ("Assets:Cash:Checking", 30000.0),
                ],
            ),
            make_tx(
                "2026-01-08",
                "房租",
                vec![
                    ("Expenses:Rent", 12000.0),
                    ("Assets:Cash:Checking", -12000.0),
                ],
            ),
        ];
        let report = compute_cashflow(
            &txs,
            Period::Month,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );
        assert_eq!(report.accounts.len(), 1);
        let acc = &report.accounts[0];
        assert_eq!(acc.periods[0].inflow as i32, 30000);
        assert_eq!(acc.periods[0].outflow as i32, 12000);
        assert_eq!(acc.periods[0].net as i32, 18000);
    }

    #[test]
    fn test_liability_outflow() {
        let txs = vec![make_tx(
            "2026-01-22",
            "信用卡还款",
            vec![
                ("Liabilities:CreditCard:BOC", 3200.0),
                ("Assets:Cash:Checking", -3200.0),
            ],
        )];
        let report = compute_cashflow(
            &txs,
            Period::Month,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );
        // 负债：正金额 = 还债 = 流出
        let liab = report
            .accounts
            .iter()
            .find(|a| a.account.starts_with("Liabilities:"))
            .unwrap();
        assert_eq!(liab.periods[0].outflow as i32, 3200);
        assert_eq!(liab.periods[0].inflow as i32, 0);
        // 资产：负金额 = 流出
        let asset = report
            .accounts
            .iter()
            .find(|a| a.account.starts_with("Assets:"))
            .unwrap();
        assert_eq!(asset.periods[0].outflow as i32, 3200);
    }

    #[test]
    fn test_txn_outside_date_range() {
        let txs = vec![make_tx(
            "2026-02-01",
            "房租",
            vec![
                ("Expenses:Rent", 12000.0),
                ("Assets:Cash:Checking", -12000.0),
            ],
        )];
        let report = compute_cashflow(
            &txs,
            Period::Month,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );
        assert_eq!(report.accounts.len(), 0);
    }

    // ── detect_patterns ─────────────────────────────────

    #[test]
    fn test_detect_recurring_monthly() {
        let mut txs = Vec::new();
        // 每月 5 号房租 12000，持续 6 个月
        for m in 1..=6 {
            let date_str = format!("2026-{:02}-05", m);
            txs.push(make_tx(
                &date_str,
                "房租",
                vec![
                    ("Expenses:Rent", 12000.0),
                    ("Assets:Cash:Checking", -12000.0),
                ],
            ));
        }
        let patterns = detect_patterns(
            &txs,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        );
        // Assets:Cash:Checking 应被检测为规律性账户
        let pat = patterns
            .iter()
            .find(|p| p.account == "Assets:Cash:Checking");
        assert!(pat.is_some(), "应为 Assets:Cash:Checking 检测到规律");
        let pat = pat.unwrap();
        assert_eq!(pat.avg_amount as i32, 12000);
        assert_eq!(pat.count, 6);
    }

    #[test]
    fn test_no_pattern_for_irregular() {
        let mut txs = Vec::new();
        // 金额差异很大，不应检测为规律
        let amounts = [100.0, 5000.0, 200.0, 8000.0, 150.0];
        for (i, &amt) in amounts.iter().enumerate() {
            let date_str = format!("2026-{:02}-05", i + 1);
            txs.push(make_tx(
                &date_str,
                "随机",
                vec![("Expenses:Random", amt), ("Assets:Cash:Checking", -amt)],
            ));
        }
        let patterns = detect_patterns(
            &txs,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
        );
        let pat = patterns
            .iter()
            .find(|p| p.account == "Assets:Cash:Checking");
        assert!(pat.is_none(), "不规则金额不应被检测为规律");
    }

    #[test]
    fn test_most_common() {
        assert_eq!(most_common(&[1, 2, 2, 3, 2]), 2);
        assert_eq!(most_common(&[15, 15, 20, 15]), 15);
        assert_eq!(most_common(&[1]), 1);
    }

    // ── compute_forecast ────────────────────────────────

    #[test]
    fn test_forecast_no_transactions() {
        let report = compute_forecast(
            &[],
            3,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        );
        assert_eq!(report.rows.len(), 0);
        assert_eq!(report.net_totals.len(), 3);
        assert_eq!(report.net_totals, vec![0.0, 0.0, 0.0]);
    }
}
