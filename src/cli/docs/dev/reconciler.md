# 对账员

## 项目概述

一个基于 Textual 的 TUI（终端用户界面）应用，提供对话式 reconciliation 功能。用户通过自然语言描述对账需求，AI 自动分析差异并生成调账建议。

**设计理念**：
> 让大模型做分析员，让验证器做审核员，让对账流程可追溯

**演进原则**：
> 解析健壮性 → 匹配算法 → LLM 稳定性 → 结果可追溯

---

## 核心模块：reconciler.py

### 完整代码

```python
"""Reconciler - 智能对账 v2.0"""
import logging
from pathlib import Path
from typing import Tuple, Optional, Dict, List
from datetime import datetime, timedelta
import re
import json

import requests
from beancount import loader
from beancount.core import data
from pydantic import BaseModel
from pydantic_settings import BaseSettings

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class Settings(BaseSettings):
    """统一配置"""
    ollama_host: str = "http://localhost:11434"
    ollama_model: str = "qwen2.5-coder:3b"
    data_root: Path = Path(__file__).parent.parent / "data"
    match_date_window: int = 2  # 日期匹配窗口（天）
    match_amount_threshold: float = 1.0  # 金额匹配阈值


class Transaction(BaseModel):
    """交易记录"""
    date: str
    description: str
    amount: float
    account: str
    source: str = "bank"  # "bank" or "ledger"
    matched: bool = False
    match_reason: Optional[str] = None


class MatchPair(BaseModel):
    """匹配对"""
    bank_idx: int
    ledger_idx: int
    reason: str
    auto: bool = False


class ReconciliationResult(BaseModel):
    """对账结果"""
    total_bank: int
    total_ledger: int
    auto_matched: int
    llm_matched: int
    auto_pairs: List[MatchPair] = []
    llm_pairs: List[MatchPair] = []
    suggestions: List[str] = []
    unresolved_bank: List[Transaction] = []
    unresolved_ledger: List[Transaction] = []


class ReconciliationReport(BaseModel):
    """完整对账报告"""
    success: bool
    message: str
    bank_file: str
    ledger_file: str
    timestamp: str = datetime.now().isoformat()
    result: Optional[ReconciliationResult] = None


# ============ Beancount 解析（使用原生 loader）============


def load_ledger(ledger_path: Path) -> List[Transaction]:
    """使用 beancount.loader 加载账本交易"""
    transactions = []
    if not ledger_path.exists():
        logger.warning("账本文件不存在: %s", ledger_path)
        return transactions
    
    try:
        entries, _, _ = loader.load_file(str(ledger_path))
        logger.info("加载账本 %s: %d 条目", ledger_path, len(entries))
        
        for entry in entries:
            if isinstance(entry, data.Transaction):
                date = entry.date.isoformat()
                narration = entry.narration or ""
                
                for post in entry.postings:
                    if post.account.startswith("Assets:") or post.account.startswith("Liabilities:"):
                        amount = float(post.units.number) if post.units.number else 0.0
                        transactions.append(Transaction(
                            date=date,
                            description=narration,
                            amount=amount,
                            account=post.account,
                            source="ledger",
                            matched=False
                        ))
    except Exception as e:
        logger.error("加载账本失败: %s", e)
    
    return transactions


# ============ 银行 CSV 解析 ============


def load_bank_statement(bank_file: Path) -> List[Transaction]:
    """从银行 CSV 加载交易记录"""
    transactions = []
    if not bank_file.exists():
        logger.warning("银行流水文件不存在: %s", bank_file)
        return transactions
    
    try:
        content = bank_file.read_text(encoding="utf-8-sig")
        lines = content.strip().split("\n")[1:]  # 跳过表头
        logger.info("加载银行流水 %s: %d 条", bank_file, len(lines))
        
        for line in lines:
            parts = line.split(",")
            if len(parts) >= 4:
                transactions.append(Transaction(
                    date=parts[0].strip('"'),
                    description=parts[1].strip('"'),
                    amount=float(parts[2].strip('"').replace(",", "")),
                    account=parts[3].strip('"'),
                    source="bank",
                    matched=False
                ))
    except Exception as e:
        logger.error("加载银行流水失败: %s", e)
    
    return transactions


# ============ 智能匹配算法 ============


def parse_date(date_str: str) -> Optional[datetime]:
    """解析日期字符串"""
    for fmt in ["%Y-%m-%d", "%Y/%m/%d", "%m/%d/%Y"]:
        try:
            return datetime.strptime(date_str, fmt)
        except ValueError:
            continue
    return None


def is_candidate(bank: Transaction, ledger: Transaction, settings: Settings) -> bool:
    """判断是否候选匹配"""
    bank_date = parse_date(bank.date)
    ledger_date = parse_date(ledger.date)
    
    if not bank_date or not ledger_date:
        return False
    
    date_diff = abs((bank_date - ledger_date).days)
    if date_diff > settings.match_date_window:
        return False
    
    amount_diff = abs(bank.amount - ledger.amount)
    if amount_diff > settings.match_amount_threshold:
        return False
    
    return True


def score_match(bank: Transaction, ledger: Transaction) -> float:
    """计算匹配分数"""
    bank_date = parse_date(bank.date)
    ledger_date = parse_date(ledger.date)
    
    date_diff = abs((bank_date - ledger_date).days) if bank_date and ledger_date else 999
    amount_diff = abs(bank.amount - ledger.amount)
    
    date_score = max(0, 1 - date_diff / 3)
    amount_score = max(0, 1 - amount_diff / max(abs(bank.amount), 1))
    
    return date_score + amount_score


def match_transactions(
    bank: List[Transaction], 
    ledger: List[Transaction],
    settings: Settings
) -> Tuple[List[MatchPair], List[Transaction], List[Transaction]]:
    """智能匹配：日期窗口 + 金额容差 + 打分排序"""
    pairs = []
    unmatched_bank = []
    unmatched_ledger = []
    
    bank_copy = [t.model_copy() for t in bank]
    ledger_copy = [t.model_copy() for t in ledger]
    
    for b_idx, b in enumerate(bank_copy):
        candidates = []
        for l_idx, l in enumerate(ledger_copy):
            if l.matched:
                continue
            if is_candidate(b, l, settings):
                score = score_match(b, l)
                candidates.append((l_idx, score))
        
        if candidates:
            candidates.sort(key=lambda x: x[1], reverse=True)
            best_idx, best_score = candidates[0]
            
            if best_score > 0.5:
                pairs.append(MatchPair(
                    bank_idx=b_idx,
                    ledger_idx=best_idx,
                    reason=f"自动匹配: 日期差异={abs((parse_date(b.date) - parse_date(ledger[best_idx].date)).days if parse_date(b.date) and parse_date(ledger[best_idx].date) else 0)}天, 金额差异={abs(b.amount - ledger[best_idx].amount):.2f}",
                    auto=True
                ))
                b.matched = True
                ledger_copy[best_idx].matched = True
    
    unmatched_bank = [b for b in bank_copy if not b.matched]
    unmatched_ledger = [l for l in ledger_copy if not l.matched]
    
    return pairs, unmatched_bank, unmatched_ledger


# ============ LLM 分析 ============


def build_prompt(bank_unmatched: List[Transaction], ledger_unmatched: List[Transaction]) -> Tuple[str, str]:
    """构建提示词（system + user 分离）"""
    system = """你是一个专业的银行对账助手。

规则：
- 分析金额、日期、描述的相似度
- 考虑可能的时差、手续费、汇率差异
- 输出 JSON，包含 matched_pairs / suggestions / unresolved
- 如果无法确定，说明原因
- 每个 matched_pair 格式: ["bank_idx", "ledger_idx", "reason"]
"""
    bank_str = "\n".join([
        f"[{i}] {t.date} | {t.description[:30]} | {t.amount:.2f}"
        for i, t in enumerate(bank_unmatched[:10])
    ])
    ledger_str = "\n".join([
        f"[{i}] {t.date} | {t.description[:30]} | {t.amount:.2f} | {t.account}"
        for i, t in enumerate(ledger_unmatched[:10])
    ])
    
    user = f"""未匹配银行流水：
{bank_str}

未匹配账本记录：
{ledger_str}

请分析以上差异并给出调账建议（JSON 格式）。"""

    return system, user


def call_llm(system: str, user: str, settings: Settings) -> str:
    """调用 Ollama（JSON 模式）"""
    resp = requests.post(
        f"{settings.ollama_host}/api/generate",
        json={
            "model": settings.ollama_model,
            "system": system,
            "prompt": user,
            "stream": False,
            "format": "json",  # JSON 模式
            "options": {"temperature": 0.1, "num_predict": 2000},
        },
        timeout=30,
    )
    resp.raise_for_status()
    return resp.json().get("response", "")


def parse_llm_response(text: str) -> dict:
    """解析 LLM 返回的 JSON"""
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        logger.warning("JSON 解析失败，尝试正则提取")
        if m := re.search(r"\{.*\}", text, re.DOTALL):
            try:
                return json.loads(m.group())
            except json.JSONDecodeError:
                pass
        return {"matched_pairs": [], "suggestions": [], "unresolved": []}


# ============ 核心入口 ============


def reconcile(bank_file: str, ledger_file: Optional[str] = None) -> ReconciliationReport:
    """
    核心接口：对账分析
    
    Args:
        bank_file: 银行流水文件路径（CSV）
        ledger_file: 账本路径（可选，默认使用 settings.data_root / main.beancount）
    
    Returns:
        ReconciliationReport: 结构化对账报告
    """
    settings = Settings()
    bank_path = Path(bank_file)
    ledger_path = Path(ledger_file) if ledger_file else settings.data_root / "main.beancount"
    
    logger.info("开始对账: bank=%s, ledger=%s", bank_path, ledger_path)
    
    # 加载数据
    bank_transactions = load_bank_statement(bank_path)
    ledger_transactions = load_ledger(ledger_path)
    
    if not bank_transactions:
        return ReconciliationReport(
            success=False,
            message="无法加载银行流水",
            bank_file=str(bank_path),
            ledger_file=str(ledger_path)
        )
    
    if not ledger_transactions:
        return ReconciliationReport(
            success=False,
            message="账本无交易记录",
            bank_file=str(bank_path),
            ledger_file=str(ledger_path)
        )
    
    # 自动匹配
    auto_pairs, unmatched_bank, unmatched_ledger = match_transactions(
        bank_transactions, ledger_transactions, settings
    )
    logger.info("自动匹配完成: %d 对, 未匹配: bank=%d, ledger=%d", 
                len(auto_pairs), len(unmatched_bank), len(unmatched_ledger))
    
    # LLM 分析未匹配项
    llm_pairs = []
    suggestions = []
    if unmatched_bank and unmatched_ledger:
        system, user = build_prompt(unmatched_bank, unmatched_ledger)
        try:
            llm_output = call_llm(system, user, settings)
            analysis = parse_llm_response(llm_output)
            
            for pair in analysis.get("matched_pairs", []):
                if len(pair) >= 3:
                    llm_pairs.append(MatchPair(
                        bank_idx=pair[0],
                        ledger_idx=pair[1],
                        reason=pair[2],
                        auto=False
                    ))
            suggestions = analysis.get("suggestions", [])
        except Exception as e:
            logger.error("LLM 调用失败: %s", e)
    
    # 构建结果
    result = ReconciliationResult(
        total_bank=len(bank_transactions),
        total_ledger=len(ledger_transactions),
        auto_matched=len(auto_pairs),
        llm_matched=len(llm_pairs),
        auto_pairs=auto_pairs,
        llm_pairs=llm_pairs,
        suggestions=suggestions,
        unresolved_bank=unmatched_bank,
        unresolved_ledger=unmatched_ledger
    )
    
    return ReconciliationReport(
        success=True,
        message=f"对账完成: 自动匹配 {result.auto_matched} 对, LLM 匹配 {result.llm_matched} 对, 未解决 {len(unmatched_bank)} 银行 + {len(unmatched_ledger)} 账本",
        bank_file=str(bank_path),
        ledger_file=str(ledger_path),
        result=result
    )


async def reconcile_async(bank_file: str, ledger_file: Optional[str] = None) -> ReconciliationReport:
    """异步版本，用于 TUI"""
    import asyncio
    return await asyncio.to_thread(reconcile, bank_file, ledger_file)
```

### 核心函数说明

| 函数 | 说明 |
|------|------|
| `load_ledger()` | 使用 beancount.loader 加载（支持完整语法） |
| `load_bank_statement()` | 从银行 CSV 加载交易记录 |
| `match_transactions()` | 智能匹配：日期窗口 + 金额容差 + 打分 |
| `is_candidate()` | 候选匹配判断 |
| `score_match()` | 匹配打分算法 |
| `build_prompt()` | 构建提示词（system/user 分离） |
| `call_llm()` | 调用 Ollama（JSON 模式） |
| `parse_llm_response()` | 解析 LLM 返回的 JSON |
| `reconcile()` | **核心入口**：返回 ReconciliationReport |
| `reconcile_async()` | 异步版本，供 TUI 使用 |

### 数据模型

```python
# 单笔交易
Transaction:
  - date, description, amount, account
  - source: "bank" | "ledger"
  - matched, match_reason

# 匹配对
MatchPair:
  - bank_idx, ledger_idx
  - reason
  - auto: 是否自动匹配

# 对账结果
ReconciliationResult:
  - total_bank, total_ledger
  - auto_matched, llm_matched
  - auto_pairs, llm_pairs  # 区分两类匹配
  - suggestions
  - unresolved_bank, unresolved_ledger

# 完整报告
ReconciliationReport:
  - success, message
  - bank_file, ledger_file, timestamp
  - result: Optional[ReconciliationResult]
```

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
load_ledger() ─ beancount.loader
    │
    ▼
match_transactions()
    │  ├─ is_candidate()  # 日期窗口 + 金额阈值
    │  └─ score_match()   # 打分排序
    │
    ├─ 自动匹配 ──────────→ auto_pairs
    │
    └─ 未匹配项 ──────────→ LLM 分析
                               │
                               ▼
                         call_llm(format="json")
                               │
                               ▼
                         parse_llm_response()
                               │
                               ▼
                         llm_pairs + suggestions
                               │
                               ▼
                         ReconciliationReport
                               │
                               ▼
                         TUI 显示 / 日志记录
```

---

## 与记账员的协作

### 输入输出契约

| 角色 | 输入 | 输出 | 消费者 |
|------|------|------|--------|
| **记账员** | 自然语言 | Beancount | 对账员 |
| **对账员** | 银行 CSV + Beancount | ReconciliationReport | 会计师 / 用户 |

### 工作流

```
记账员 (bookkeeper)
    │
    ▼
Beancount 账本
    │
    ▼
对账员 (reconciler)
    ├─ 自动匹配 → 人工确认
    └─ LLM 建议 → 人工确认
    │
    ▼
调账（Beancount 片段）
    │
    ▼
再对账 → 循环
```

---

## 演进计划

### 短期（可用性）
- [x] Beancount 解析改用 loader
- [x] 匹配逻辑增加日期窗口 + 金额容差
- [x] LLM 启用 JSON 模式

### 中期（稳定性）
- [ ] 区分自动匹配 + LLM 匹配
- [x] ReconciliationReport 结构化
- [x] 日志记录

### 长期（产品化）
- [ ] 支持多银行、多账本
- [ ] 手动确认/修改匹配结果
- [ ] HTTP/gRPC 服务化
