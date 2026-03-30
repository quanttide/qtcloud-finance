# 量潮财务云命令行工具

## 项目结构

```
cli/
├── src/
│   ├── __init__.py
│   ├── tui.py          # TUI 应用入口（见 tui.md）
│   └── bookkeeper.py        # 核心逻辑（见下文）
├── data/
│   └── main.beancount       # 账本文件
└── docs/
    ├── dev/                   # 开发者文档
    │   ├── bookkeeper.md      # 本文档（总览）
    │   ├── config.md          # 配置说明
    │   └── tui.md             # TUI 界面说明
    ├── qa/                    # 质量保证文档
    │   ├── README.md          # QA 索引
    │   └── bookkeeper.md  
            tui.md
    └── README.md              # 项目说明
```
