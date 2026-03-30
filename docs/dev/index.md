# qtcloud-finance 开发文档

## Agent 体系

qtcloud-finance 是一套财务 AI Agent 系统，包含以下角色：

| Agent | 职责 | 输入 | 输出 |
|-------|------|------|------|
| **记账员 (Bookkeeper)** | 记录交易 | 自然语言 | 复式记账格式 (Beancount) |
| **对账员 (Reconciler)** | 核对差异 | 银行流水 + 账本 | 调账建议 |

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

## 文档索引

- [bookkeeper.py](../src/cli/dev/bookkeeper.md) - 记账员实现
- [reconciler.py](../src/cli/dev/reconciler.md) - 对账员实现
- [tui.md](../src/cli/dev/tui.md) - TUI 实现
- [config.md](../src/cli/dev/config.md) - 配置说明

---

## 核心实现

- 使用 **Beancount** 格式
- 存储：本地文件 / OSS / S3
