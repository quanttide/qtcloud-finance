# TUI模块

## 概述

基于 Textual 框架的三列布局终端应用，提供对话式记账界面。

## 完整代码

```python
"""Tally TUI - 对话式记账"""
from textual.app import App, ComposeResult
from textual.containers import Container, ScrollableContainer
from textual.widgets import Button, Footer, Header, Input, Static, TextArea
from textual.binding import Binding

from .bookkeeper import journalize_async, get_open_accounts, Settings


class ChatMessage(Static):
    """单条聊天消息组件"""

    def __init__(self, content: str, is_user: bool = False, is_error: bool = False):
        super().__init__()
        self.content = content
        self.is_user = is_user
        self.is_error = is_error

    def compose(self) -> ComposeResult:
        prefix = "👤 你" if self.is_user else "🤖 助手"
        yield Static(prefix, classes="message-prefix")
        content = Static(self.content, classes="message-content")
        if self.is_error:
            content.add_class("error")
        yield content


class TallyApp(App):
    """Tally TUI 应用"""

    CSS = """
    #main { layout: horizontal; height: 1fr; }
    #sidebar { width: 20; background: $primary-background; border-right: solid $primary; }
    #chat {
        width: 40%;
        border-right: solid $primary;
        layout: vertical;
        align-vertical: bottom;
    }
    #preview-panel {
        width: 1fr;
        layout: vertical;
        align-vertical: bottom;
    }
    #preview { height: 1fr; width: 1fr; }
    #history { height: 1fr; overflow-y: auto; }
    #input-area { height: 5; border-top: solid $primary; padding: 1; }
    #buttons { height: auto; border-top: solid $primary; padding: 1; width: 1fr; }
    .message-prefix { color: $primary; text-style: bold; margin-bottom: 0; }
    .message-content { margin-left: 2; margin-bottom: 1; }
    .error { color: $error; }
    Input { width: 100%; }
    Button { margin-right: 1; }
    """

    BINDINGS = [
        Binding("q", "quit", "退出"),
        Binding("y", "confirm", "确认", priority=True),
        Binding("n", "reject", "重写", priority=True),
    ]

    def __init__(self):
        super().__init__()
        self.settings = Settings()
        self.current = None  # 当前待确认账单

    def compose(self) -> ComposeResult:
        yield Header()
        with Container(id="main"):
            # 左侧：账户列表
            with Container(id="sidebar"):
                yield Static("账户", classes="title")
                yield Static(id="accounts")

            # 中间：对话面板
            with Container(id="chat"):
                yield Static("对话", classes="title")
                yield ScrollableContainer(id="history")
                with Container(id="input-area"):
                    yield Input(placeholder="输入描述... (Enter 发送)", id="input")

            # 右侧：预览面板
            with Container(id="preview-panel"):
                yield TextArea(id="preview", read_only=True)
                with Container(id="buttons"):
                    yield Button("❌ 重写 (N)", id="reject", variant="error")
                    yield Button("✅ 确认 (Y)", id="confirm", variant="success", disabled=True)
        yield Footer()

    def on_mount(self) -> None:
        """启动时初始化"""
        accounts = get_open_accounts(self.settings.data_root / "main.beancount")
        self.query_one("#accounts", Static).update(
            "\n".join(accounts) if accounts else "无账户"
        )
        self.query_one("#history", ScrollableContainer).mount(
            ChatMessage(
                "你好！我是你的记账助手。\n\n"
                "请输入消费描述，例如：\n"
                "• 早餐 10 元 微信\n"
                "• 地铁上班 5 元\n"
                "• 昨天超市买菜 125 元 支付宝",
                is_user=False
            )
        )

    def add_msg(self, content: str, is_user: bool = False, is_error: bool = False):
        """添加聊天消息"""
        history = self.query_one("#history", ScrollableContainer)
        history.mount(ChatMessage(content, is_user=is_user, is_error=is_error))
        self.call_later(lambda: history.scroll_end(animate=False))

    async def on_input_submitted(self, event: Input.Submitted) -> None:
        """处理用户输入"""
        input_w = self.query_one("#input", Input)
        text = input_w.value.strip()
        if not text:
            return
        
        self.add_msg(text, is_user=True)
        input_w.value = ""
        self.add_msg("生成中...")
        
        # 调用异步 journalize
        ok, result = await journalize_async(
            text, 
            self.settings.data_root / "main.beancount"
        )
        
        # 移除"生成中"消息
        history = self.query_one("#history", ScrollableContainer)
        history.query(".message-content").last().remove()
        
        if ok:
            self.current = result
            self.query_one("#preview", TextArea).text = result
            self.query_one("#confirm", Button).disabled = False
            self.add_msg("✅ 已生成账单，按 Y 确认写入，或按 N 重新生成")
        else:
            self.add_msg(f"❌ {result}", is_error=True)

    def action_confirm(self) -> None:
        """确认写入（Y 键）"""
        if self.current:
            self.add_msg(f"✅ 已写入账本:\n{self.current}")
            self.query_one("#preview", TextArea).text = ""
            self.query_one("#confirm", Button).disabled = True
            self.current = None

    def action_reject(self) -> None:
        """拒绝重写（N 键）"""
        if self.current:
            self.query_one("#preview", TextArea).text = ""
            self.query_one("#confirm", Button).disabled = True
            self.add_msg("好的，请重新描述或告诉我如何修改")
            self.current = None

    def on_button_pressed(self, event: Button.Pressed) -> None:
        """按钮点击事件"""
        if event.button.id == "confirm":
            self.action_confirm()
        elif event.button.id == "reject":
            self.action_reject()


def main():
    """启动 TUI"""
    TallyApp().run()


if __name__ == "__main__":
    main()
```

---

## 页面设计

### 三列布局

```
┌─────────────────────────────────────────────────────────────────┐
│ Header: 量潮财务云                                                   │
├──────────┬──────────────────────────┬───────────────────────────┤
│          │ 对话                     │ 预览                      │
│ 账户     │ ┌──────────────────────┐ │ ┌───────────────────────┐ │
│          │ │ 🤖 助手：你好！...   │ │ │ 2026-03-30 * "早餐"   │ │
│ Assets:  │ │ 👤 你：早餐 10 元微信 │ │ │   Expenses:Food...    │ │
│ - WeChat │ │ 🤖 助手：生成中...   │ │ │   Assets:Digital...   │ │
│ - Bank   │ │                      │ │ │                       │ │
│          │ │                      │ │ │                       │ │
│          │ └──────────────────────┘ │ ├───────────────────────┤ │
│          │ [输入框：输入描述...  ]  │ │ [❌ 重写] [✅ 确认]   │ │
│          │                          │ └───────────────────────┘ │
├──────────┴──────────────────────────┴───────────────────────────┤
│ Footer: q 退出  y 确认  n 重写                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 布局说明

| 区域 | 宽度 | 说明 |
|------|------|------|
| 左侧（#sidebar） | 20 字符 | 显示账户列表 |
| 中间（#chat） | 40% | 对话历史 + 输入框 |
| 右侧（#preview-panel） | 剩余空间 | Beancount 预览 + 确认/重写按钮 |

**注意**: 
- `#preview-panel` 是右侧容器 ID，使用垂直布局
- `#preview` 是 TextArea ID，占据预览区域主要空间
- `#buttons` 固定在预览面板底部，包含确认/重写按钮

---

## 组件 ID 映射

| ID | 组件类型 | 说明 |
|----|----------|------|
| `#main` | Container | 主容器（三列布局） |
| `#sidebar` | Container | 左侧账户列表容器 |
| `#accounts` | Static | 账户列表显示 |
| `#chat` | Container | 中间对话面板容器 |
| `#history` | ScrollableContainer | 聊天历史（可滚动） |
| `#input` | Input | 用户输入框 |
| `#input-area` | Container | 输入框容器 |
| `#preview-panel` | Container | 右侧预览面板**容器**（垂直布局） |
| `#preview` | TextArea | Beancount 预览（只读） |
| `#buttons` | Container | 按钮容器（固定在底部） |
| `#confirm` | Button | 确认按钮（Y 键） |
| `#reject` | Button | 重写按钮（N 键） |

**重要**: 所有 ID 必须唯一，不能重复使用。`query_one()` 会返回第一个匹配的节点。

---

## CSS 关键样式

```css
/* 三列横向布局 */
#main { layout: horizontal; height: 1fr; }

/* 左侧固定宽度，深色背景 */
#sidebar {
    width: 20;
    background: $primary-background;
    border-right: solid $primary;
}

/* 中间占 40%，垂直布局，内容对齐底部 */
#chat { 
    width: 40%; 
    border-right: solid $primary; 
    layout: vertical;
    align-vertical: bottom;
}

/* 右侧占剩余空间，垂直布局，内容对齐底部 */
#preview-panel { 
    width: 1fr; 
    layout: vertical; 
    align-vertical: bottom;
}

/* 预览 TextArea 占剩余空间 */
#preview { height: 1fr; width: 1fr; }

/* 聊天历史自动滚动 */
#history { height: 1fr; overflow-y: auto; }

/* 输入区域和按钮区域固定高度 */
#input-area { height: 5; border-top: solid $primary; padding: 1; }
#buttons { height: auto; border-top: solid $primary; padding: 1; width: 1fr; }

/* 消息样式 */
.message-prefix { color: $primary; text-style: bold; margin-bottom: 0; }
.message-content { margin-left: 2; margin-bottom: 1; }
.error { color: $error; }

/* 输入框占满宽度 */
Input { width: 100%; }

/* 按钮间距 */
Button { margin-right: 1; }
```

---

## 快捷键

| 键 | 功能 | 优先级 | 说明 |
|----|------|--------|------|
| `q` | 退出应用 | 普通 | 任意焦点下可用 |
| `y` | 确认写入 | **高** | 即使焦点在输入框也响应 |
| `n` | 拒绝重写 | **高** | 即使焦点在输入框也响应 |
| `Enter` | 发送输入 | 普通 | 焦点在输入框时 |

---

## 核心方法

### ChatMessage 组件

```python
class ChatMessage(Static):
    """单条聊天消息组件"""
    
    def __init__(self, content: str, is_user: bool = False, is_error: bool = False):
        super().__init__()
        self.content = content
        self.is_user = is_user
        self.is_error = is_error

    def compose(self) -> ComposeResult:
        prefix = "👤 你" if self.is_user else "🤖 助手"
        yield Static(prefix, classes="message-prefix")
        content = Static(self.content, classes="message-content")
        if self.is_error:
            content.add_class("error")
        yield content
```

### TallyApp 方法

| 方法 | 说明 |
|------|------|
| `__init__()` | 初始化配置和状态（`current`, `is_processing`, `_loading_msg`） |
| `compose()` | 构建界面布局 |
| `on_mount()` | 启动时初始化账户列表和欢迎消息 |
| `add_msg()` | 添加聊天消息到历史，**返回组件引用** |
| `on_input_submitted()` | 处理用户输入（异步调用 journalize，**防重复提交**） |
| `action_confirm()` | Y 键确认写入，**调用 `append_to_ledger` 实际写入** |
| `action_reject()` | N 键拒绝重写 |
| `on_button_pressed()` | 处理按钮点击 |

---

## 事件流程

```
用户输入文本 → 按 Enter
    │
    ▼
on_input_submitted()
    │
    ├── 检查 is_processing（防重复提交）
    ├── 禁用输入框
    │
    ├── 添加用户消息到聊天历史
    │
    ├── 显示"生成中..."（保存引用）
    │
    ├── await journalize_async()
    │       │
    │       ├── 获取账户列表
    │       ├── 构建提示词
    │       ├── 调用 Ollama
    │       ├── 提取 Beancount
    │       └── 验证语法        ← 到此为止，不写入账本
    │
    ├── 精准移除"生成中"消息（使用引用）
    │
    ├── 启用输入框
    │
    ├── 成功？
    │   ├── 更新预览面板
    │   ├── 启用确认按钮
    │   └── 提示按 Y/N
    │
    └── 失败？
        └── 显示错误消息

───────────────────────────────────────────────

用户按 Y 键
    │
    ▼
action_confirm()
    │
    ├── append_to_ledger()      ← 实际写入账本
    ├── 添加成功消息
    ├── 清空预览面板
    └── 禁用确认按钮
```

**关键设计决策**:
1. `journalize_async` 只生成草稿，不自动写入
2. 用户按 Y 确认后才调用 `append_to_ledger` 实际写入
3. 使用 `is_processing` 防止重复提交
4. 使用消息引用精准删除"生成中"提示

---

## 运行 TUI

```bash
# 确保 Ollama 运行
ollama serve

# 启动 TUI（两种方式）
python -m src
# 或使用 uv
uv run tally
```

## 依赖

```bash
pip install textual pydantic-settings beancount requests
```

## 核心模块依赖

| 模块 | 函数 | 说明 |
|------|------|------|
| `bookkeeper` | `journalize_async` | 生成 Beancount 草稿（不写入） |
| `bookkeeper` | `append_to_ledger` | 将草稿追加到账本 |
| `bookkeeper` | `get_open_accounts` | 提取账本中的 open 账户 |
| `config` | `Settings` | 配置管理（Ollama 主机、模型等） |
