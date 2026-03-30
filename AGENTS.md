# AGENTS.md

## 文档索引

| 文档 | 用途 |
|------|------|
| [README.md](README.md) | 项目简介、运行方式 |
| [ROADMAP.md](ROADMAP.md) | 版本规划、模块状态 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 开发指南、代码风格、发布流程 |

## 使用条件

### 开发新功能
1. 阅读 `ROADMAP.md` 确认当前版本目标
2. 参考 `CONTRIBUTING.md` 的代码风格
3. 在对应子模块 `src/{module}/` 下开发

### 修复 Bug
1. 参考 `CONTRIBUTING.md` 的测试命令验证修复
2. 遵循代码风格

### 发布版本
1. 参考 `CONTRIBUTING.md` 的发布步骤
2. 更新 `ROADMAP.md` 状态

### 运行测试
```bash
cd src/cli && uv run pytest tests/ -v
```

---

## 自我更新条件

当发生以下情况时，应更新本文档：

1. **新增子模块**：更新文档索引表
2. **文档结构变更**：修正链接路径
3. **新增重要文档**：添加到索引表
