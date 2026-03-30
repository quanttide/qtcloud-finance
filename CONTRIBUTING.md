# 贡献指南

## 项目结构

多模块 monorepo，各模块独立版本管理：

```
src/
├── cli/          # CLI 应用 (qtcloud-finance-cli)
├── provider/     # 数据提供层 (计划中)
└── studio/       # 可视化界面 (计划中)
```

每个子模块有自己的 `pyproject.toml`、`tests/`、`ROADMAP.md`。

---

## 构建与依赖

使用 `uv` 管理依赖和运行：

```bash
# 进入子模块
cd src/cli

# 安装依赖
uv sync

# 运行应用
uv run qtcloud-finance

# 添加依赖
uv add <package>
uv add --dev <package>
```

---

## 测试

```bash
# 运行所有测试
cd src/cli && uv run pytest tests/

# 运行单个测试文件
uv run pytest tests/test_bookkeeper.py

# 运行单个测试函数
uv run pytest tests/test_bookkeeper.py::test_validate_valid

# 详细输出
uv run pytest tests/ -v
```

测试使用 `pytest` + `pytest-asyncio`，fixtures 定义在 `tests/conftest.py`。

---

## 代码风格

### Python 版本
- Python >= 3.12
- 使用新版语法：`list[str]` 而非 `List[str]`（简单类型）
- 复杂泛型仍从 `typing` 导入：`Optional`, `Tuple`

### 导入顺序
```python
# 1. 标准库
import re
from pathlib import Path
from typing import Optional

# 2. 第三方库
import requests
from pydantic import BaseModel

# 3. 本地模块（相对导入）
from .config import Settings
```

### 命名规范
- 模块/文件：`snake_case.py`
- 类：`PascalCase`
- 函数/变量：`snake_case`
- 常量：`UPPER_SNAKE_CASE`
- 私有：`_leading_underscore`

### 类型注解
```python
def get_accounts(path: Path) -> list[str]:
    ...

def process(data: str) -> tuple[bool, str]:
    ...

async def fetch(url: str, timeout: Optional[int] = None) -> dict:
    ...
```

### 文档字符串
模块和公共函数使用 docstring：
```python
"""模块说明"""


def func(arg: str) -> bool:
    """函数说明

    Args:
        arg: 参数说明

    Returns:
        返回值说明
    """
```

### 错误处理
```python
# 返回 (success, result) 元组
def validate(text: str) -> tuple[bool, str]:
    if not text.strip():
        return False, "空内容"
    try:
        ...
        return True, text
    except Exception as e:
        return False, str(e)
```

### Pydantic 模型
```python
from pydantic import BaseModel
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """配置使用 BaseSettings，支持环境变量"""
    host: str = "http://localhost:8080"


class Data(BaseModel):
    """数据模型使用 BaseModel"""
    name: str
    value: float
```

---

## 发布步骤

以发布 `cli/v0.0.1-alpha.1` 为例：

### 1. 更新版本号

修改 `src/cli/pyproject.toml`：

```toml
version = "0.0.1-alpha.1"
```

### 2. 写 CHANGELOG

创建/更新 `src/cli/CHANGELOG.md`：

```markdown
# Changelog

## [0.0.1-alpha.1] - 2026-03-30

### Added
- 功能列表...
```

### 3. 提交代码

```bash
git add src/cli/CHANGELOG.md src/cli/pyproject.toml
git commit -m "release: cli v0.0.1-alpha.1"
git push
```

### 4. 创建 Tag

```bash
git tag -a cli/v0.0.1-alpha.1 -m "Release cli/v0.0.1-alpha.1"
git push origin cli/v0.0.1-alpha.1
```

### 5. 创建 GitHub Release

```bash
gh release create cli/v0.0.1-alpha.1 --title "cli/v0.0.1-alpha.1" --notes "..."
```

### 版本号规范

格式：`{模块}/v{版本}`

示例：
- `cli/v0.0.1-alpha.1`
- `cli/v0.1.0`
- `api/v1.0.0`
