# 贡献指南

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

## 版本号规范

格式：`{模块}/v{版本}`

示例：
- `cli/v0.0.1-alpha.1`
- `cli/v0.1.0`
- `api/v1.0.0`
