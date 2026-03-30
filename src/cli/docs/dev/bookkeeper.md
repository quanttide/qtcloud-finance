# 记账员

## 项目概述

一个基于 Textual 的 TUI（终端用户界面）应用，提供对话式 journalize 功能。用户通过自然语言描述消费，AI 自动生成 Beancount 复式记账格式并写入账本。

**设计理念**：
> 让大模型做打字员，让解析器做审核员，让代码保持简单


## 核心模块：bookkeeper.py

### 完整代码

```python
"""Bookkeeper - Beancount 智能记账"""
from pathlib import Path
from typing import Tuple, Optional
import re

import requests
from beancount import parser, loader
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """统一配置"""
    ollama_host: str = "http://localhost:11434"
    ollama_model: str = "qwen2.5-coder:3b"
    data_root: Path = Path(__file__).parent.parent / "data"


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
            "options": {"temperature": 0.1, "num_predict": 1000},
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
    
    ledger.write_text(ledger.read_text() + "\n" + result + "\n")
    return True, f"已记录:\n{result}"


async def journalize_async(user_input: str, ledger_path: Optional[Path] = None) -> tuple[bool, str]:
    """异步版本，用于 TUI"""
    import asyncio
    return await asyncio.to_thread(journalize, user_input, ledger_path)
```

### 核心函数说明

| 函数 | 说明 |
|------|------|
| `get_open_accounts()` | 从账本提取 open 账户列表 |
| `build_prompt()` | 构建 LLM 提示词（系统 + 用户） |
| `call_llm()` | 调用 Ollama API |
| `extract_beancount()` | 从 LLM 输出提取 Beancount 代码 |
| `validate()` | 使用 beancount.parser 验证语法 |
| `journalize()` | **核心入口**：完整流程（生成 + 验证 + 写入） |
| `journalize_async()` | 异步版本，供 TUI 使用 |

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
用户输入
    │
    ▼
on_input_submitted()
    │
    ▼
journalize_async() ──────┐
    │                    │ (后台线程)
    ▼                    ▼
build_prompt()       call_llm()
    │                    │
    ▼                    ▼
get_open_accounts()  Ollama API
    │                    │
    ▼                    ▼
    └──────────────→ extract_beancount()
                           │
                           ▼
                       validate() ──×── 失败 → 返回错误
                           │
                           ✓
                           ▼
                       写入账本
                           │
                           ▼
                       返回结果 → TUI 更新预览
```

---

## 快速开始

### 步骤 1：创建项目结构

```bash
mkdir -p src data
touch src/__init__.py
touch src/bookkeeper.py
touch src/__main__.py
```

### 步骤 2：写入 bookkeeper.py

复制上方"核心模块：bookkeeper.py"完整代码

### 步骤 3：写入 __main__.py

参考 [tui.md](tui.md) 中的完整代码

### 步骤 4：安装依赖

```bash
pip install textual pydantic-settings beancount requests
```

或使用 uv：
```bash
uv add textual pydantic-settings beancount requests
```

### 步骤 5：启动 Ollama

```bash
# 确保 Ollama 服务运行
ollama serve

# 拉取模型（如果未安装）
ollama pull qwen2.5-coder:3b
```

### 步骤 6：运行 TUI

```bash
python -m src
```

---

## 常见问题

### Q: LLM 无法连接
**A**:
1. 确保 Ollama 服务运行：`ollama serve`
2. 检查模型已安装：`ollama list | grep qwen`
3. 测试 API：`curl http://localhost:11434/api/tags`

### Q: 账户列表为空
**A**: 首次运行时会自动创建默认账本。如需自定义账户，编辑 `data/main.beancount`：
```beancount
2026-01-01 open Assets:Digital:WeChatPay
2026-01-01 open Assets:Bank:CMB
2026-01-01 open Expenses:Food:Restaurant
```

### Q: 验证失败但看起来正确
**A**: `validate()` 使用 beancount 原生解析器，严格检查语法。常见错误：
- 缩进不是两个空格
- 借贷不平衡
- 账户名不存在

### Q: 如何更换模型
**A**: 设置环境变量：
```bash
export OLLAMA_MODEL=qwen2.5:7b
python -m src
```

---

## 扩展建议

1. **多账本支持**：添加账本选择器，支持 `--ledger` 参数
2. **对话上下文**：将 `chat_history` 传给 LLM，支持多轮对话修改
3. **分类推荐**：记录历史交易，自动推荐常用分类
4. **批量导入**：支持 CSV/Excel 导入，批量 journalize
5. **报表集成**：集成 `bean-report` 生成收支报表
