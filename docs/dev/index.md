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

### 设计思路

- [bookkeeper.md](./bookkeeper.md) - 记账员设计思路

### 实现文档

- [bookkeeper.py](../src/cli/dev/bookkeeper.md) - 记账员实现
- [reconciler.py](../src/cli/dev/reconciler.md) - 对账员实现
- [tui.md](../src/cli/dev/tui.md) - TUI 实现
- [config.md](../src/cli/dev/config.md) - 配置说明

---

## 核心架构

### 分层设计

```
┌─────────────────────────────────────┐
│           UI 层 (TUI)               │
├─────────────────────────────────────┤
│         Agent 层                     │  记账员 / 对账员
├─────────────────────────────────────┤
│         存储层                       │  本地 / OSS / S3
└─────────────────────────────────────┘
```

### 存储后端

| 后端 | 说明 |
|------|------|
| 本地文件 | 开发/单机 |
| OSS | 阿里云对象存储 |
| S3 | AWS S3 |
| Git | 版本控制 |

### 账本格式

使用 **Beancount**（复式记账格式）
