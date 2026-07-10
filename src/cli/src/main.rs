mod beancount;
mod cashflow;
mod cli;

use cashflow::*;
use chrono::{Datelike, NaiveDate};
use clap::Parser;
use cli::{CashflowCommand, Commands};
use std::process;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Cashflow { sub } => match sub {
            CashflowCommand::Summary { path, format } => {
                match run_status(&path, "month", None, None, &format) {
                    Ok(()) => {}
                    Err(e) => eprintln!("错误: {}", e),
                }
            }
            CashflowCommand::Status {
                path,
                period,
                start,
                end,
                format,
            } => match run_status(&path, &period, start.as_deref(), end.as_deref(), &format) {
                Ok(()) => {}
                Err(e) => eprintln!("错误: {}", e),
            },
            CashflowCommand::Simulate {
                path,
                adjust,
                one_time,
                start,
                end,
                format,
            } => match run_simulate(
                &path,
                &adjust,
                &one_time,
                start.as_deref(),
                end.as_deref(),
                &format,
            ) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("错误: {}", e);
                    process::exit(1);
                }
            },
            CashflowCommand::Forecast {
                path,
                periods,
                method,
                format,
            } => match run_forecast(&path, periods, &method, &format) {
                Ok(()) => {}
                Err(e) => eprintln!("错误: {}", e),
            },
        },
    }
}

fn run_status(
    path: &str,
    period_str: &str,
    start: Option<&str>,
    end: Option<&str>,
    format: &str,
) -> Result<(), String> {
    let transactions = beancount::parse(path)?;
    let (period, start_date, end_date) = parse_period(period_str, start, end)?;
    let report = compute_cashflow(&transactions, period, start_date, end_date);
    print_status_table(&report, format);
    Ok(())
}

fn run_simulate(
    path: &str,
    adjusts: &[String],
    one_times: &[String],
    start: Option<&str>,
    end: Option<&str>,
    format: &str,
) -> Result<(), String> {
    let transactions = beancount::parse(path)?;

    let adjustments: Vec<Adjustment> = adjusts
        .iter()
        .map(|s| parse_adjustment(s))
        .collect::<Result<Vec<_>, _>>()?;

    // 一次性事件格式: DATE@AMOUNT（不含账户，金额即现金影响）
    let mut events: Vec<(NaiveDate, String, f64)> = Vec::new();
    for s in one_times {
        let parts: Vec<&str> = s.splitn(2, '@').collect();
        if parts.len() < 2 {
            return Err(format!("一次性事件格式错误: '{}'，应为 DATE@AMOUNT", s));
        }
        let date = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
            .map_err(|e| format!("日期解析失败 '{}': {}", parts[0], e))?;
        let amount: f64 = parts[1]
            .parse()
            .map_err(|_| format!("无效金额: '{}'", parts[1]))?;
        events.push((date, format!("事件 {:.0}", amount), amount));
    }

    // 有起止日期时按 custom 覆盖整个范围，否则按月
    let period_str = if start.is_some() || end.is_some() {
        format!(
            "custom:{}..{}",
            start.unwrap_or("2026-01-01"),
            end.unwrap_or("2026-12-31")
        )
    } else {
        "month".to_string()
    };
    let (period, start_date, end_date) = parse_period(&period_str, None, None)?;
    let report = compute_simulate(
        &transactions,
        &adjustments,
        &events,
        period,
        start_date,
        end_date,
    );
    print_simulate_table(&report, format);
    Ok(())
}

fn run_forecast(path: &str, periods: u32, _method: &str, format: &str) -> Result<(), String> {
    let transactions = beancount::parse(path)?;

    // 用过去 12 个月做预测基准
    let end = chrono::Local::now().date_naive();
    let start = end - chrono::Duration::days(365);
    let real_start = NaiveDate::from_ymd_opt(start.year(), start.month(), 1).unwrap();

    let report = compute_forecast(&transactions, periods, real_start, end);
    print_forecast_table(&report, format);
    Ok(())
}
