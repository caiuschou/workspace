# 代理配置

代理是专门配置的 AI 助手，具有特定的工具、权限和行为设置。

## 代理类型

### 主代理 (Primary Agents)

直接交互的主要助手，通过 Tab 键切换。

| 代理 | 用途 |
|------|------|
| **Build** | 默认代理，拥有完整工具访问权限，用于开发工作 |
| **Plan** | 只读代理，用于代码探索和分析，不修改文件 |

### 子代理 (Subagents)

自动调用或通过 `@` 提及的专用助手。

| 代理 | 用途 |
|------|------|
| **General** | 多步骤任务，拥有完整工具访问 |
| **Explore** | 只读代码库探索 |

## 基础配置

### JSON 配置

在 `opencode.json` 中配置：

```json
{
  "agent": {
    "build": {
      "mode": "primary",
      "model": "anthropic/claude-sonnet-4",
      "tools": {
        "write": true,
        "bash": true,
        "edit": true
      }
    },
    "plan": {
      "mode": "primary",
      "model": "anthropic/claude-sonnet-4",
      "tools": {
        "write": false,
        "bash": false,
        "edit": false,
        "read": true,
        "glob": true,
        "grep": true
      }
    }
  }
}
```

### Markdown 配置

在 `~/.config/opencode/agents/` (全局) 或 `.opencode/agents/` (项目) 创建 Markdown 文件：

```markdown
<!-- .opencode/agents/reviewer.md -->
---
mode: primary
model: anthropic/claude-sonnet-4
tools:
  edit: false
  write: false
  bash: false
---

# Code Reviewer

你是一个专业的代码审查员。你的任务是：

1. 分析代码质量
2. 检查潜在问题
3. 提供改进建议

你不能修改代码，只能提供建议。
```

## 配置选项

### 基础选项

| 选项 | 类型 | 描述 |
|------|------|------|
| `mode` | `"primary"` \| `"subagent"` | 代理类型 |
| `model` | `string` | 使用的模型 ID |
| `temperature` | `number` | 响应随机性 (0.0-1.0) |
| `maxSteps` | `number` | 最大迭代次数 |

### 工具配置

```json
{
  "agent": {
    "myagent": {
      "tools": {
        "bash": true,
        "edit": true,
        "write": false,
        "webfetch": true,
        "mymcp_*": true
      }
    }
  }
}
```

### 权限配置

```json
{
  "agent": {
    "myagent": {
      "permission": {
        "bash": "ask",
        "edit": "allow",
        "webfetch": "allow"
      }
    }
  }
}
```

### Bash 细粒度权限

```json
{
  "agent": {
    "build": {
      "permission": {
        "bash": {
          "git *": "allow",
          "npm *": "allow",
          "rm -rf *": "deny",
          "*": "ask"
        }
      }
    }
  }
}
```

## 创建自定义代理

### 使用命令行

```bash
opencode agent create
```

交互式向导会引导你完成：
- 代理名称和描述
- 工具选择
- 文件生成

### 手动创建

1. **JSON 方式**:

```json
{
  "agent": {
    "security": {
      "mode": "subagent",
      "model": "anthropic/claude-sonnet-4",
      "tools": {
        "read": true,
        "grep": true,
        "glob": true,
        "bash": false,
        "edit": false
      }
    }
  }
}
```

2. **Markdown 方式**:

```markdown
<!-- .opencode/agents/security.md -->
---
mode: subagent
model: anthropic/claude-sonnet-4
tools:
  read: true
  grep: true
  glob: true
  bash: false
  edit: false
---

# Security Auditor

你是一个安全审计专家。专注于：

- 检测常见安全漏洞 (OWASP Top 10)
- 代码注入风险
- 敏感数据泄露
- 认证和授权问题

只分析代码，不做任何修改。
```

## 代理调用

### Tab 切换

在 TUI 中按 Tab 键切换主代理。

### @ 提及

```
@security 检查这个文件的安全问题
@explore 解释这个函数的作用
```

### 命令指定

```bash
/review-code --agent security
```

### SDK 调用

```typescript
const session = await client.session.create({
  agent: "security"
})
```

## 预设代理配置

### 代码审查员

```json
{
  "agent": {
    "reviewer": {
      "mode": "primary",
      "model": "anthropic/claude-opus-4-5",
      "temperature": 0.3,
      "tools": {
        "read": true,
        "glob": true,
        "grep": true,
        "lsp": true,
        "edit": false,
        "write": false,
        "bash": false
      }
    }
  }
}
```

### 测试专家

```json
{
  "agent": {
    "tester": {
      "mode": "subagent",
      "model": "anthropic/claude-sonnet-4",
      "tools": {
        "read": true,
        "write": true,
        "edit": true,
        "bash": true,
        "glob": true
      },
      "permission": {
        "bash": {
          "npm test*": "allow",
          "jest *": "allow",
          "vitest *": "allow",
          "*": "ask"
        }
      }
    }
  }
}
```

### 文档编写者

```json
{
  "agent": {
    "docs": {
      "mode": "subagent",
      "model": "anthropic/claude-sonnet-4",
      "temperature": 0.7,
      "tools": {
        "read": true,
        "write": true,
        "edit": true,
        "glob": true,
        "webfetch": true,
        "bash": false
      }
    }
  }
}
```

### 调试专家

```json
{
  "agent": {
    "debugger": {
      "mode": "subagent",
      "model": "anthropic/claude-opus-4-5",
      "maxSteps": 20,
      "tools": {
        "read": true,
        "edit": true,
        "bash": true,
        "lsp": true,
        "glob": true,
        "grep": true
      },
      "permission": {
        "bash": {
          "node --inspect*": "allow",
          "npm run debug*": "allow",
          "*": "ask"
        }
      }
    }
  }
}
```

## 高级配置

### 模型变体

```json
{
  "agent": {
    "complex-task": {
      "model": "anthropic/claude-sonnet-4",
      "modelVariant": "high"
    }
  }
}
```

### 步骤限制

```json
{
  "agent": {
    "quick": {
      "maxSteps": 5
    },
    "thorough": {
      "maxSteps": 30
    }
  }
}
```

### 继承配置

代理会继承全局配置，可以覆盖特定选项：

```json
{
  "permission": {
    "bash": "ask"
  },
  "agent": {
    "build": {
      "permission": {
        "bash": {
          "git *": "allow"
        }
      }
    }
  }
}
```

## 调试代理

### 查看可用代理

```bash
opencode agent list
```

### 代理日志

使用 SDK 记录日志：

```typescript
await client.app.log({
  level: "debug",
  message: "Agent action"
})
```

## 最佳实践

1. **职责单一**: 每个代理专注于特定任务
2. **最小权限**: 只授予必要的工具和权限
3. **清晰指令**: 在 Markdown 中写明代理的职责
4. **适当温度**: 创意任务高温度，精确任务低温度
5. **步骤限制**: 设置合理的 maxSteps 防止无限循环

## 下一步

- [工具系统](tools.md) - 了解可用工具
- [插件开发](plugins.md) - 扩展代理功能
- [命令参考](commands.md) - 创建代理专用命令
