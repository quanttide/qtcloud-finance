"""记账模块测试"""

import pytest
from pathlib import Path

from qtcloud_finance_cli.bookkeeper import (
    get_open_accounts,
    build_prompt,
    extract_beancount,
    validate,
    append_to_ledger,
)


def test_get_open_accounts(sample_ledger):
    """测试提取 open 账户"""
    accounts = get_open_accounts(sample_ledger)

    assert len(accounts) == 4
    assert "Assets:Bank:CMB" in accounts
    assert "Assets:Digital:WeChatPay" in accounts
    assert "Expenses:Food:Restaurant" in accounts
    assert "Equity:Opening-Balances" in accounts


def test_get_open_accounts_nonexistent(tmp_data_dir):
    """测试不存在的账本文件"""
    accounts = get_open_accounts(tmp_data_dir / "nonexistent.beancount")
    assert accounts == []


def test_build_prompt():
    """测试构建提示词"""
    accounts = ["Assets:Digital:WeChatPay", "Expenses:Food:Restaurant"]
    user_input = "早餐 10 元 微信"

    system, user = build_prompt(accounts, user_input)

    assert "Assets:Digital:WeChatPay" in system
    assert "Expenses:Food:Restaurant" in system
    assert user_input in user


def test_extract_beancount_code_block():
    """测试从代码块中提取"""
    text = """```beancount
2026-03-30 * "早餐"
  Expenses:Food:Restaurant  10.00 CNY
  Assets:Digital:WeChatPay  -10.00 CNY
```"""

    result = extract_beancount(text)

    assert "2026-03-30" in result
    assert "Expenses:Food:Restaurant" in result


def test_extract_beancount_plain():
    """测试从纯文本中提取"""
    text = """2026-03-30 * "早餐"
  Expenses:Food:Restaurant  10.00 CNY
  Assets:Digital:WeChatPay  -10.00 CNY"""

    result = extract_beancount(text)

    assert "2026-03-30" in result
    assert "Expenses:Food:Restaurant" in result


def test_extract_beancount_invalid():
    """测试无效输入"""
    result = extract_beancount("Hello World")
    assert result == ""


def test_validate_valid():
    """测试验证有效 beancount"""
    text = """2026-03-30 * "早餐"
  Expenses:Food:Restaurant  10.00 CNY
  Assets:Digital:WeChatPay  -10.00 CNY"""

    ok, result = validate(text)

    assert ok is True
    assert result == text


def test_validate_empty():
    """测试验证空内容"""
    ok, result = validate("")

    assert ok is False
    assert "空内容" in result


def test_validate_invalid_syntax():
    """测试验证无效语法"""
    text = """invalid syntax here"""

    ok, result = validate(text)

    assert ok is False


def test_append_to_ledger(tmp_data_dir):
    """测试追加到账本"""
    ledger_path = tmp_data_dir / "test.beancount"
    ledger_path.write_text("""2026-01-01 open Assets:Digital:WeChatPay
""")

    content = """2026-03-30 * "早餐"
  Expenses:Food:Restaurant  10.00 CNY
  Assets:Digital:WeChatPay  -10.00 CNY"""

    result = append_to_ledger(ledger_path, content)

    assert result is True
    assert "早餐" in ledger_path.read_text()


def test_append_to_ledger_nonexistent_dir():
    """测试追加到不存在的目录"""
    ledger_path = Path("/nonexistent/dir/test.beancount")
    content = "test"

    result = append_to_ledger(ledger_path, content)

    assert result is False
