"""Tally TUI - 对话式记账"""

from textual.app import App, ComposeResult
from textual.containers import Container, ScrollableContainer
from textual.widgets import Button, Footer, Header, Input, Static, TextArea
from textual.binding import Binding

from .bookkeeper import journalize_async, get_open_accounts, append_to_ledger
from .config import Settings


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
                    yield Button(
                        "✅ 确认 (Y)", id="confirm", variant="success", disabled=True
                    )
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
                is_user=False,
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
            text, self.settings.data_root / "main.beancount"
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
            # 从结果中提取 beancount 内容
            content = self.current.replace("已记录:\n", "")
            ledger_path = self.settings.data_root / "main.beancount"
            if append_to_ledger(ledger_path, content):
                self.add_msg(f"✅ 已写入账本:\n{content}")
            else:
                self.add_msg("❌ 写入账本失败", is_error=True)
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
