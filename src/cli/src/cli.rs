use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qtcloud-finance", version, about = "量潮财务云 CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 现金流相关操作
    Cashflow {
        #[command(subcommand)]
        sub: CashflowCommand,
    },
}

#[derive(Subcommand)]
pub enum CashflowCommand {
    /// 预留：现金流汇总
    Summary {
        /// Beancount 文件路径，默认 $QTCLOUD_BEANCOUNT_FILE 或 main.beancount
        #[arg(default_value = "main.beancount")]
        path: String,
        /// 输出格式: table, csv, json
        #[arg(long, short, default_value = "table")]
        format: String,
    },
    /// 指定期间的现金流状况
    Status {
        /// Beancount 文件路径
        #[arg(default_value = "main.beancount")]
        path: String,
        /// 期间: week, month, quarter, year, 或 custom:YYYY-MM-DD..YYYY-MM-DD
        #[arg(long, short = 'p', default_value = "month")]
        period: String,
        /// 起始日期 (YYYY-MM-DD)
        #[arg(long, short)]
        start: Option<String>,
        /// 截止日期 (YYYY-MM-DD)
        #[arg(long, short)]
        end: Option<String>,
        /// 输出格式: table, csv, json
        #[arg(long, short, default_value = "table")]
        format: String,
    },
    /// 假设场景推演
    Simulate {
        /// Beancount 文件路径
        #[arg(default_value = "main.beancount")]
        path: String,
        /// 调整项: ACCOUNT@AMOUNT, 如 Income:Salary@-20%, Expenses:Rent@1000
        #[arg(long, short = 'a', name = "ADJUST")]
        adjust: Vec<String>,
        /// 一次性事件: DATE@AMOUNT, 金额直接表示现金影响（正=流入，负=流出）
        #[arg(long, short = 'o', name = "ONE_TIME")]
        one_time: Vec<String>,
        /// 起始日期 (YYYY-MM-DD)
        #[arg(long, short)]
        start: Option<String>,
        /// 截止日期 (YYYY-MM-DD)
        #[arg(long, short)]
        end: Option<String>,
        /// 输出格式: table, json
        #[arg(long, short, default_value = "table")]
        format: String,
    },
    /// 现金流预测
    Forecast {
        /// Beancount 文件路径
        #[arg(default_value = "main.beancount")]
        path: String,
        /// 预测期数
        #[arg(long, short = 'n', default_value = "3")]
        periods: u32,
        /// 预测方法: recurring, trend
        #[arg(long, short, default_value = "recurring")]
        method: String,
        /// 输出格式: table, csv, json
        #[arg(long, short, default_value = "table")]
        format: String,
    },
}
