"""Bookkeeper - Beancount 智能记账"""

import re
from pathlib import Path
from typing import Optional

import requests
from beancount import loader
from beancount.parser import parser

from .config import Settings


def get_open_accounts(ledger_path: Path) -> list[str]:
    """提取账本中的 open 账户"""
    try:
        entries, _, _ = loader.load_file(str(ledger_path))
        return sorted({e.account for e in entries if hasattr(e, "account")})
    except Exception:
        return []


def build_prompt(accounts: list[str], user_input: str) -> tuple[str, str]:
    """构建系统提示词和用户提示词"""
    account_list = "\n".join(f"- {a}" for a in accounts)
    system = f"""你是一个严谨的 Beancount 记账助手。

可用账户：
{account_list}

规则：
- 日期格式：YYYY-MM-DD，默认 2026-03-30
- 金额两位小数，CNY 货币
- 借贷平衡（正负号相反）
- 只输出 Beancount 代码块

示例：
2026-03-30 * "早餐"
  Expenses:Food:Restaurant  10.00 CNY
  Assets:Digital:WeChatPay  -10.00 CNY
"""
    return system, f'请将 "{user_input}" 转为 Beancount，只输出代码块：'


def call_llm(system: str, user: str, settings: Settings) -> str:
    """调用 Ollama"""
    resp = requests.post(
        f"{settings.ollama_host}/api/generate",
        json={
            "model": settings.ollama_model,
            "system": system,
            "prompt": user,
            "stream": False,
            "options": {
                "temperature": settings.temperature,
                "num_predict": settings.max_tokens,
            },
        },
        timeout=30,
    )
    resp.raise_for_status()
    return resp.json().get("response", "")


def extract_beancount(text: str) -> str:
    """提取 Beancount 代码块"""
    if m := re.search(r"```(?:beancount)?\n(.*?)```", text, re.DOTALL):
        return m.group(1).strip()
    if re.match(r"^\d{4}-\d{2}-\d{2}", text.strip()):
        return text.strip()
    return ""


def validate(text: str) -> tuple[bool, str]:
    """验证 Beancount 语法"""
    if not text.strip():
        return False, "空内容"
    try:
        entries, errors, _ = parser.parse_string(text)
        if errors and not entries:
            return False, "\n".join(f"Line {e.line}: {e.message}" for e in errors)
        if not entries:
            return False, "无交易记录"
        return True, text
    except Exception as e:
        return False, str(e)


def append_to_ledger(ledger: Path, content: str) -> bool:
    """追加内容到账本文件"""
    try:
        with open(ledger, "a", encoding="utf-8") as f:
            f.write("\n" + content + "\n")
        return True
    except Exception:
        return False


def journalize(user_input: str, ledger_path: Optional[Path] = None) -> tuple[bool, str]:
    """
    核心接口：将自然语言转为 Beancount 并写入账本

    Args:
        user_input: 用户自然语言描述，如"早餐 10 元 微信"
        ledger_path: 账本路径（可选，默认使用 settings.data_root / main.beancount）

    Returns:
        (success, message): success 为 True 表示成功，message 包含结果或错误信息
    """
    settings = Settings()
    ledger = ledger_path or settings.data_root / "main.beancount"
    ledger.parent.mkdir(parents=True, exist_ok=True)

    # 初始化账本（如果不存在）
    if not ledger.exists():
        ledger.write_text("""2026-01-01 open Assets:Digital:WeChatPay
2026-01-01 open Assets:Bank:CMB
2026-01-01 open Expenses:Food:Restaurant
2026-01-01 open Equity:Opening-Balances
""")

    accounts = get_open_accounts(ledger)
    if not accounts:
        return False, "无法读取账户"

    system, user = build_prompt(accounts, user_input)
    llm_output = call_llm(system, user, settings)
    beancount = extract_beancount(llm_output)

    if not beancount:
        return False, "无法生成账单"

    ok, result = validate(beancount)
    if not ok:
        return False, f"验证失败：{result}"

    if append_to_ledger(ledger, result):
        return True, f"已记录:\n{result}"
    return False, "写入账本失败"


async def journalize_async(
    user_input: str, ledger_path: Optional[Path] = None
) -> tuple[bool, str]:
    """异步版本，用于 TUI"""
    import asyncio

    return await asyncio.to_thread(journalize, user_input, ledger_path)
