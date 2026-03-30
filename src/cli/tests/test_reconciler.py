"""对账模块测试"""

import pytest
from pathlib import Path

from qtcloud_finance_cli.reconciler import (
    Transaction,
    MatchPair,
    ReconciliationResult,
    ReconciliationReport,
    load_ledger,
    load_bank_statement,
    parse_date,
    is_candidate,
    score_match,
    match_transactions,
    parse_llm_response,
    Settings,
)


def test_transaction_model():
    """测试交易模型"""
    t = Transaction(
        date="2026-03-30",
        description="早餐",
        amount=10.0,
        account="Expenses:Food:Restaurant",
        source="ledger",
    )

    assert t.date == "2026-03-30"
    assert t.description == "早餐"
    assert t.amount == 10.0
    assert t.source == "ledger"
    assert t.matched is False


def test_match_pair_model():
    """测试匹配对模型"""
    pair = MatchPair(bank_idx=0, ledger_idx=1, reason="自动匹配", auto=True)

    assert pair.bank_idx == 0
    assert pair.ledger_idx == 1
    assert pair.auto is True


def test_reconciliation_result_model():
    """测试对账结果模型"""
    result = ReconciliationResult(
        total_bank=10, total_ledger=8, auto_matched=5, llm_matched=2
    )

    assert result.total_bank == 10
    assert result.total_ledger == 8
    assert result.auto_matched == 5
    assert result.llm_matched == 2


def test_reconciliation_report_model():
    """测试对账报告模型"""
    report = ReconciliationReport(
        success=True,
        message="测试完成",
        bank_file="bank.csv",
        ledger_file="main.beancount",
    )

    assert report.success is True
    assert report.message == "测试完成"


def test_load_ledger(sample_ledger):
    """测试加载账本"""
    transactions = load_ledger(sample_ledger)

    assert len(transactions) > 0
    assert any(t.description == "煎饼果子" for t in transactions)


def test_load_ledger_nonexistent(tmp_data_dir):
    """测试加载不存在的账本"""
    transactions = load_ledger(tmp_data_dir / "nonexistent.beancount")

    assert transactions == []


def test_load_bank_statement(sample_bank_csv):
    """测试加载银行流水"""
    transactions = load_bank_statement(sample_bank_csv)

    assert len(transactions) == 2
    assert transactions[0].description == "煎饼果子"
    assert transactions[0].amount == 10.0


def test_load_bank_statement_nonexistent(tmp_data_dir):
    """测试加载不存在的银行流水"""
    transactions = load_bank_statement(tmp_data_dir / "nonexistent.csv")

    assert transactions == []


def test_parse_date():
    """测试日期解析"""
    assert parse_date("2026-03-30").year == 2026
    assert parse_date("2026/03/30").month == 3
    assert parse_date("03/30/2026").day == 30
    assert parse_date("invalid") is None


def test_is_candidate():
    """测试候选匹配判断"""
    settings = Settings(match_date_window=2, match_amount_threshold=1.0)

    bank = Transaction(
        date="2026-03-30", description="早餐", amount=10.0, account="WeChatPay"
    )
    ledger = Transaction(
        date="2026-03-30",
        description="煎饼果子",
        amount=10.0,
        account="Expenses:Food:Restaurant",
    )

    assert is_candidate(bank, ledger, settings) is True


def test_is_candidate_date_diff():
    """测试日期差异过大的情况"""
    settings = Settings(match_date_window=2, match_amount_threshold=1.0)

    bank = Transaction(
        date="2026-03-30", description="早餐", amount=10.0, account="WeChatPay"
    )
    ledger = Transaction(
        date="2026-03-20",
        description="煎饼果子",
        amount=10.0,
        account="Expenses:Food:Restaurant",
    )

    assert is_candidate(bank, ledger, settings) is False


def test_is_candidate_amount_diff():
    """测试金额差异过大的情况"""
    settings = Settings(match_date_window=2, match_amount_threshold=1.0)

    bank = Transaction(
        date="2026-03-30", description="早餐", amount=10.0, account="WeChatPay"
    )
    ledger = Transaction(
        date="2026-03-30",
        description="煎饼果子",
        amount=20.0,
        account="Expenses:Food:Restaurant",
    )

    assert is_candidate(bank, ledger, settings) is False


def test_score_match():
    """测试匹配打分"""
    bank = Transaction(
        date="2026-03-30", description="早餐", amount=10.0, account="WeChatPay"
    )
    ledger = Transaction(
        date="2026-03-30",
        description="煎饼果子",
        amount=10.0,
        account="Expenses:Food:Restaurant",
    )

    score = score_match(bank, ledger)

    assert score > 1.5  # 日期和金额都完全匹配


def test_match_transactions():
    """测试自动匹配"""
    settings = Settings(match_date_window=2, match_amount_threshold=1.0)

    bank = [
        Transaction(
            date="2026-03-30", description="早餐", amount=10.0, account="WeChatPay"
        ),
        Transaction(
            date="2026-03-29", description="地铁", amount=5.0, account="WeChatPay"
        ),
    ]

    ledger = [
        Transaction(
            date="2026-03-30",
            description="煎饼果子",
            amount=10.0,
            account="Expenses:Food:Restaurant",
        ),
        Transaction(
            date="2026-03-29",
            description="地铁",
            amount=5.0,
            account="Expenses:Transport",
        ),
    ]

    pairs, unmatched_bank, unmatched_ledger = match_transactions(bank, ledger, settings)

    assert len(pairs) == 2
    assert len(unmatched_bank) == 0
    assert len(unmatched_ledger) == 0


def test_parse_llm_response_valid():
    """测试解析有效的 LLM 响应"""
    text = (
        '{"matched_pairs": [[0, 1, "原因"]], "suggestions": ["建议"], "unresolved": []}'
    )

    result = parse_llm_response(text)

    assert len(result["matched_pairs"]) == 1
    assert result["matched_pairs"][0] == [0, 1, "原因"]


def test_parse_llm_response_invalid():
    """测试解析无效的 LLM 响应"""
    text = "Hello World"

    result = parse_llm_response(text)

    assert result["matched_pairs"] == []
    assert result["suggestions"] == []
