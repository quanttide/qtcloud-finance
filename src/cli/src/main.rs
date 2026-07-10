mod beancount;
mod cashflow;
mod cli;

use cashflow::*;
use chrono::{Datelike, NaiveDate};
use clap::Parser;
use cli::{CashflowCommand, Commands};

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
