# 对账员

## 项目概述

一个基于 Textual 的 TUI（终端用户界面）应用，提供对话式 reconciliation 功能。用户通过自然语言描述对账需求，AI 自动分析差异并生成调账建议。

**设计理念**：
> 让大模型做分析员，让验证器做审核员，让对账流程可追溯

## 核心模块：reconciler.py

### 完整代码

```python
"""Reconciler - 智能对账"""
from pathlib import Path
from typing import Tuple, Optional, Dict, List
from datetime import datetime
import re
import json

import requests
from pydantic import BaseModel
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """统一配置"""
    ollama_host: str = "http://localhost:11434"
    ollama_model: str = "qwen2.5-coder:3b"
    data_root: Path = Path(__file__).parent.parent / "data"


class Transaction(BaseModel):
    """交易记录"""
    date: str
    description: str
    amount: float
    account: str
    matched: bool = False


class ReconciliationResult(BaseModel):
    """对账结果"""
    total_records: int
    matched: int
    unmatched: List[Dict]
    suggestions: List[str]


def load_bank_statement(bank_file: Path) -> List[Transaction]:
    """从银行 CSV 加载交易记录"""
    transactions = []
    if not bank_file.exists():
        return transactions
    
    content = bank_file.read_text(encoding="utf-8-sig")
    lines = content.strip().split("\n")[1:]  # 跳过表头
    
    for line in lines:
        parts = line.split(",")
        if len(parts) >= 4:
            transactions.append(Transaction(
                date=parts[0].strip('"'),
                description=parts[1].strip('"'),
                amount=float(parts[2].strip('"').replace(",", "")),
                account=parts[3].strip('"')
            ))
    return transactions


def load_ledger(ledger_path: Path) -> List[Transaction]:
    """从 Beancount 账本加载交易记录"""
    transactions = []
    if not ledger_path.exists():
        return transactions
    
    content = ledger_path.read_text()
    pattern = r"(\d{4}-\d{2}-\d{2})\s+\*\s+\"([^\"]+)\"\s*\n\s+([^\s:]+)\s+([\d.]+)\s+CNY"
    
    for match in re.finditer(pattern, content, re.MULTILINE):
        date, desc, account, amount = match.groups()
        transactions.append(Transaction(
            date=date,
            description=desc,
            amount=float(amount),
            account=account,
            matched=False
        ))
    return transactions


def match_transactions(bank: List[Transaction], ledger: List[Transaction]) -> Tuple[List[Transaction], List[Transaction]]:
    """匹配银行和账本交易"""
    unmatched_bank = []
    unmatched_ledger = []
    
    for b in bank:
        b.matched = False
        for l in ledger:
            if (abs(b.amount - l.amount) < 0.01 and 
                b.date == l.date and 
                not l.matched):
                b.matched = True
                l.matched = True
                break
        if not b.matched:
            unmatched_bank.append(b)
    
    unmatched_ledger = [l for l in ledger if not l.matched]
    return unmatched_bank, unmatched_ledger


def build_prompt(bank_unmatched: List[Transaction], ledger_unmatched: List[Transaction]) -> tuple[str, str]:
    """构建对账分析提示词"""
    bank_str = "\n".join([
        f"- {t.date} | {t.description} | {t.amount} | {t.account}"
        for t in bank_unmatched[:10]
    ])
    ledger_str = "\n".join([
        f"- {t.date} | {t.description} | {t.amount} | {t.account}"
        for t in ledger_unmatched[:10]
    ])
    
    system = f"""你是一个专业的财务对账助手。

未匹配银行流水：
{bank_str}

未匹配账本记录：
{ledger_str}

规则：
- 分析金额、日期、描述的相似度
- 考虑可能的时差、手续费、汇率差异
- 输出调账建议（Beancount 格式）
- 如果无法确定，说明原因

输出格式：
```json
{{
  "matched_pairs": [["银行序号", "账本序号", "原因"], ...],
  "suggestions": ["调账建议1", "调账建议2"],
  "unresolved": ["未解决项1", "未解决项2"]
}}
```"""
    return system, "请分析以上差异并给出调账建议："


def call_llm(system: str, user: str, settings: Settings) -> str:
    """调用 Ollama"""
    resp = requests.post(
        f"{settings.ollama_host}/api/generate",
        json={
            "model": settings.ollama_model,
            "system": system,
            "prompt": user,
            "stream": False,
            "options": {"temperature": 0.2, "num_predict": 2000},
        },
        timeout=30,
    )
    resp.raise_for_status()
    return resp.json().get("response", "")


def parse_llm_response(text: str) -> dict:
    """解析 LLM 返回的 JSON"""
    if m := re.search(r"```json\n(.*?)```", text, re.DOTALL):
        return json.loads(m.group(1))
    return {"matched_pairs": [], "suggestions": [], "unresolved": []}


def reconcile(bank_file: str, ledger_file: Optional[str] = None) -> Tuple[bool, str]:
    """
    核心接口：对账分析
    
    Args:
        bank_file: 银行流水文件路径（CSV）
        ledger_file: 账本路径（可选，默认使用 settings.data_root / main.beancount）
    
    Returns:
        (success, message): success 为 True 表示成功，message 包含结果
    """
    settings = Settings()
    bank_path = Path(bank_file)
    ledger_path = Path(ledger_file) if ledger_file else settings.data_root / "main.beancount"
    
    bank_transactions = load_bank_statement(bank_path)
    ledger_transactions = load_ledger(ledger_path)
    
    if not bank_transactions:
        return False, "无法加载银行流水"
    if not ledger_transactions:
        return False, "账本无交易记录"
    
    unmatched_bank, unmatched_ledger = match_transactions(bank_transactions, ledger_transactions)
    
    system, user = build_prompt(unmatched_bank, unmatched_ledger)
    llm_output = call_llm(system, user, settings)
    analysis = parse_llm_response(llm_output)
    
    result = ReconciliationResult(
        total_records=len(bank_transactions) + len(ledger_transactions),
        matched=len(bank_transactions) - len(unmatched_bank),
        unmatched=[
            {"bank": b.model_dump(), "ledger": l.model_dump()}
            for b, l in zip(unmatched_bank[:5], unmatched_ledger[:5])
        ],
        suggestions=analysis.get("suggestions", [])
    )
    
    return True, f"""对账结果：
- 总记录数：{result.total_records}
- 已匹配：{result.matched}
- 未匹配：{len(unmatched_bank)} 银行 + {len(unmatched_ledger)} 账本

调账建议：
{chr(10).join(result.suggestions) if result.suggestions else "无"}
"""


async def reconcile_async(bank_file: str, ledger_file: Optional[str] = None) -> Tuple[bool, str]:
    """异步版本，用于 TUI"""
    import asyncio
    return await asyncio.to_thread(reconcile, bank_file, ledger_file)
```

### 核心函数说明

| 函数 | 说明 |
|------|------|
| `load_bank_statement()` | 从银行 CSV 加载交易记录 |
| `load_ledger()` | 从 Beancount 账本提取交易 |
| `match_transactions()` | 自动匹配银行与账本（金额+日期） |
| `build_prompt()` | 构建 LLM 分析提示词 |
| `call_llm()` | 调用 Ollama API |
| `parse_llm_response()` | 解析 LLM 返回的 JSON |
| `reconcile()` | **核心入口**：完整对账流程 |
| `reconcile_async()` | 异步版本，供 TUI 使用 |

### 配置项

```python
class Settings(BaseSettings):
    ollama_host: str = "http://localhost:11434"   # Ollama 地址
    ollama_model: str = "qwen2.5-coder:3b"        # 模型名称
    data_root: Path = Path(__file__).parent.parent / "data"  # 数据目录
```

详细配置说明见 [config.md](config.md)。

---

## 数据流

```
银行 CSV 文件
    │
    ▼
load_bank_statement()
    │
    ▼
账本文件 (Beancount)
    │
    ▼
load_ledger()
    │
    ▼
match_transactions() ──────┐
    │                      │
    ▼                      ▼
未匹配银行流水        未匹配账本记录
    │                      │
    └──────────┬───────────┘
               ▼
        build_prompt()
               │
               ▼
           call_llm()
               │
               ▼
           Ollama API
               │
               ▼
        parse_llm_response()
               │
               ▼
        生成调账建议
               │
               ▼
        返回对账结果 → TUI 显示
```

---

## 快速开始

### 步骤 1：准备银行流水 CSV

```csv
日期,交易描述,金额,对方账户
2026-03-28,工资,15000.00,公司账户
2026-03-29,午餐,-15.00,餐饮
2026-03-30,购物,-299.00,淘宝
```

### 步骤 2：运行对账

```python
from reconciler import reconcile

ok, result = reconcile("data/bank_202603.csv")
print(result)
```

### 步骤 3：TUI 使用

```bash
python -m src --mode reconcile --bank data/bank_202603.csv
```

---

## 常见问题

### Q: 银行 CSV 格式不标准
**A**: 目前支持标准格式 `日期,交易描述,金额,对方账户`，可扩展 `load_bank_statement()` 支持更多格式。

### Q: 金额不匹配
**A**: 
- 检查时差（银行次日到账）
- 考虑手续费差异
- LLM 会分析可能的匹配项

### Q: 无法自动匹配
**A**: LLM 会给出建议，人工确认后手动调账。

---

## 与记账员的关系

| 角色 | 职责 | 输入 | 输出 |
|------|------|------|------|
| **记账员** | 记录交易 | 自然语言 | Beancount |
| **对账员** | 核对差异 | 银行流水 + 账本 | 调账建议 |

工作流：`记账 → 对账 → 调账 → 再对账`
