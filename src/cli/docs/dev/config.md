# 声明式配置

本文档说明 Tally Bookkeeper 的配置管理。

## 配置类

Tally 使用 `pydantic-settings` 进行配置管理，所有配置项集中在 `Settings` 类中。

### 完整代码

```python
from pydantic_settings import BaseSettings
from pathlib import Path


class Settings(BaseSettings):
    """统一配置"""
    
    # Ollama 配置
    ollama_host: str = "http://localhost:11434"
    ollama_model: str = "qwen2.5-coder:3b"
    
    # 数据目录
    data_root: Path = Path(__file__).parent.parent / "data"
    
    # LLM 参数（可选）
    temperature: float = 0.1
    max_tokens: int = 1000
```

## 配置项说明

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `ollama_host` | str | `http://localhost:11434` | Ollama API 地址 |
| `ollama_model` | str | `qwen2.5-coder:3b` | 使用的 LLM 模型名称 |
| `data_root` | Path | `src/../data` | 数据目录路径 |
| `temperature` | float | `0.1` | LLM 温度（越低越确定） |
| `max_tokens` | int | `1000` | 最大生成 token 数 |

## 使用环境变量

所有配置项可通过环境变量覆盖：

```bash
# 自定义 Ollama 地址
export OLLAMA_HOST=http://192.168.1.100:11434

# 更换模型
export OLLAMA_MODEL=qwen2.5:7b

# 自定义数据目录
export DATA_ROOT=/home/user/my-ledger

# 运行
python -m src
```

## 使用示例

### 在代码中获取配置

```python
from .bookkeeper import Settings

settings = Settings()
print(settings.ollama_host)
print(settings.data_root)
```

### 在 bookkeeper.py 中使用

```python
def journalize(user_input: str, ledger_path: Optional[Path] = None) -> tuple[bool, str]:
    settings = Settings()
    ledger = ledger_path or settings.data_root / "main.beancount"
    # ...
```

### 在 TUI 中使用

```python
from .bookkeeper import Settings

class TallyApp(App):
    def __init__(self):
        super().__init__()
        self.settings = Settings()
    
    def on_mount(self) -> None:
        ledger_path = self.settings.data_root / "main.beancount"
        # ...
```

## 配置文件（可选）

如需使用配置文件，创建 `.env` 文件在项目根目录：

```ini
# .env
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=qwen2.5-coder:3b
DATA_ROOT=./data
TEMPERATURE=0.1
MAX_TOKENS=1000
```

`pydantic-settings` 会自动读取 `.env` 文件。

## 配置优先级

配置加载优先级（从高到低）：

1. 环境变量（如 `OLLAMA_HOST`）
2. `.env` 文件
3. 代码中的默认值

## 验证配置

```python
settings = Settings()

# 验证 Ollama 连接
import requests
try:
    resp = requests.get(f"{settings.ollama_host}/api/tags", timeout=5)
    print(f"Ollama 已连接：{resp.json()}")
except Exception as e:
    print(f"Ollama 连接失败：{e}")

# 验证数据目录
print(f"数据目录：{settings.data_root}")
print(f"目录存在：{settings.data_root.exists()}")
```
