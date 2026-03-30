"""Pytest 配置"""

import pytest
from pathlib import Path
import tempfile


@pytest.fixture
def tmp_data_dir():
    """创建临时数据目录"""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def sample_ledger(tmp_data_dir):
    """创建示例账本文件"""
    ledger_path = tmp_data_dir / "main.beancount"
    ledger_path.write_text("""2026-01-01 open Assets:Digital:WeChatPay
2026-01-01 open Assets:Bank:CMB
2026-01-01 open Expenses:Food:Restaurant
2026-01-01 open Equity:Opening-Balances

2026-03-30 * "早餐" "煎饼果子"
  Expenses:Food:Restaurant  10.00 CNY
  Assets:Digital:WeChatPay  -10.00 CNY
""")
    return ledger_path


@pytest.fixture
def sample_bank_csv(tmp_data_dir):
    """创建示例银行流水 CSV"""
    csv_path = tmp_data_dir / "bank.csv"
    csv_path.write_text("""日期,摘要,金额,账户
2026-03-30,煎饼果子,10.00,WeChatPay
2026-03-29,地铁,5.00,WeChatPay
""")
    return csv_path
