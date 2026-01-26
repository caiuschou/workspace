# 命令参考

OpenCode 支持内置命令和自定义命令，通过 `/` 前缀调用。

## 内置命令

| 命令 | 描述 |
|------|------|
| `/init` | 初始化项目配置 |
| `/connect` | 连接 AI 提供商 |
| `/models` | 查看可用模型 |
| `/tools` | 查看可用工具 |
| `/agents` | 查看可用代理 |
| `/undo` | 撤销上一步操作 |
| `/redo` | 重做操作 |
| `/share` | 分享会话 |
| `/help` | 显示帮助 |

## 自定义命令

### 创建命令

命令文件存放在：
- `.opencode/commands/` - 项目级
- `~/.config/opencode/commands/` - 全局

### 文件格式

支持 Markdown (.md)、YAML (.yaml/.yml)、JSON (.json) 格式。

### Markdown 格式

```markdown
<!-- .opencode/commands/test.md -->
---
description: 运行测试套件
agent: build
---

运行完整的测试套件并生成覆盖率报告。如果有失败，分析原因并修复。
```

### YAML 格式

```yaml
# .opencode/commands/test.yaml
description: 运行测试套件
agent: build
template: |
  运行完整的测试套件并生成覆盖率报告。
  如果有失败，分析原因并修复。
```

### JSON 格式

```json
{
  "description": "运行测试套件",
  "agent": "build",
  "template": "运行完整的测试套件并生成覆盖率报告。"
}
```

## 命令参数

| 参数 | 类型 | 描述 | 必需 |
|------|------|------|------|
| `template` | string | 发送给 LLM 的提示 | 是 |
| `description` | string | TUI 中显示的描述 | 否 |
| `agent` | string | 执行命令的代理 | 否 |
| `model` | string | 覆盖默认模型 | 否 |
| `subtask` | boolean | 强制使用子代理 | 否 |

## 占位符

### 参数占位符

```markdown
---
description: 创建 React 组件
---

创建一个名为 $1 的 React 组件，包含以下功能：
$ARGUMENTS
```

| 占位符 | 描述 |
|--------|------|
| `$ARGUMENTS` | 所有传入的参数 |
| `$1, $2, $3...` | 位置参数 |

### 使用

```
/component Button "带有 hover 效果"
```

### 文件引用

```markdown
---
description: 审查组件代码
---

审查 @src/components/Button.tsx 的代码质量。
```

使用 `@filename` 引用文件内容。

### Shell 命令输出

```markdown
---
description: 检查 Git 状态
---

当前 Git 状态：
!`git status`!

请分析并给出建议。
```

使用 `` !`command`! `` 注入命令输出。

## 示例命令

### 测试命令

```markdown
<!-- .opencode/commands/test.md -->
---
description: 运行测试并修复失败
agent: build
---

运行项目测试：
!`npm test`!

如果有失败的测试，分析原因并修复。
```

### 代码审查

```markdown
<!-- .opencode/commands/review.md -->
---
description: 审查代码变更
agent: reviewer
---

审查当前的代码变更：
!`git diff`!

检查：
1. 代码质量
2. 潜在 bug
3. 性能问题
4. 安全隐患

给出具体的改进建议。
```

### 组件生成

```markdown
<!-- .opencode/commands/component.md -->
---
description: 创建 React 组件
---

在 src/components/ 下创建一个新的 React 组件：

组件名：$1
功能要求：$ARGUMENTS

要求：
- 使用 TypeScript
- 包含 Props 类型定义
- 添加基础样式
- 导出组件
```

使用：
```
/component UserProfile "显示用户头像和名称"
```

### 文档生成

```markdown
<!-- .opencode/commands/docs.md -->
---
description: 生成 API 文档
agent: docs
---

为 @$1 生成 API 文档：

1. 提取所有导出的函数和类
2. 描述每个函数的用途
3. 记录参数和返回值
4. 提供使用示例
```

使用：
```
/docs src/utils/api.ts
```

### 提交助手

```markdown
<!-- .opencode/commands/commit.md -->
---
description: 智能提交
---

查看当前变更：
!`git diff --staged`!

根据变更内容生成 commit message：
- 使用 conventional commit 格式
- 简洁描述变更内容
- 中文描述

然后执行 git commit。
```

### 调试助手

```markdown
<!-- .opencode/commands/debug.md -->
---
description: 调试错误
agent: debugger
---

分析以下错误：
$ARGUMENTS

1. 解释错误原因
2. 定位问题代码
3. 提供修复方案
4. 应用修复
```

使用：
```
/debug "TypeError: Cannot read property 'map' of undefined"
```

### 重构命令

```markdown
<!-- .opencode/commands/refactor.md -->
---
description: 重构代码
---

重构 @$1：

目标：$ARGUMENTS

要求：
- 保持功能不变
- 提高代码可读性
- 遵循项目规范
- 添加必要的类型注解
```

使用：
```
/refactor src/utils/helpers.ts "提取重复逻辑为独立函数"
```

## 配置文件方式

也可以在 `opencode.json` 中定义命令：

```json
{
  "command": {
    "lint": {
      "description": "运行 lint 检查",
      "template": "运行 npm run lint 并修复所有问题"
    },
    "build": {
      "description": "构建项目",
      "template": "运行 npm run build，如果失败则分析并修复"
    }
  }
}
```

## TUI 中使用

1. 输入 `/` 查看命令列表
2. 选择或输入命令名
3. 添加参数（如果需要）
4. 回车执行

## CLI 中使用

```bash
# 执行命令
opencode --command test

# 带参数
opencode --command component Button
```

## SDK 中使用

```typescript
await client.session.command(sessionId, {
  command: "/test"
})
```

## 最佳实践

1. **描述清晰**: 让用户知道命令做什么
2. **模板完整**: 提供足够的上下文给 LLM
3. **适当代理**: 选择合适的代理执行
4. **参数灵活**: 使用占位符支持不同场景
5. **文档化**: 在项目 README 中说明自定义命令

## 下一步

- [代理配置](agents.md) - 配置命令使用的代理
- [工具系统](tools.md) - 了解命令可用的工具
- [SDK 参考](sdk.md) - 程序化执行命令
