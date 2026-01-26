# 插件开发

插件通过钩子扩展 OpenCode 功能，可以监听事件、添加工具、修改行为。

## 插件结构

插件是 JavaScript/TypeScript 模块，导出接收上下文并返回钩子的函数。

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const MyPlugin: Plugin = async (ctx) => {
  const { project, client, $, directory, worktree } = ctx

  return {
    // 钩子实现
  }
}
```

### 上下文参数

| 参数 | 描述 |
|------|------|
| `project` | 当前项目信息 |
| `client` | SDK 客户端实例 |
| `$` | Bun shell API |
| `directory` | 工作目录 |
| `worktree` | Git worktree 路径 |

## 插件位置

- **项目级**: `.opencode/plugins/`
- **全局**: `~/.config/opencode/plugins/`

## 加载方式

### 本地文件

直接放入 plugins 目录：

```
.opencode/
  plugins/
    my-plugin.ts
    another-plugin.js
```

### npm 包

在 `opencode.json` 中指定：

```json
{
  "plugin": [
    "opencode-helicone-session",
    "@my-org/custom-plugin"
  ]
}
```

### 依赖管理

在 `.opencode/package.json` 中声明依赖：

```json
{
  "dependencies": {
    "lodash": "^4.17.21"
  }
}
```

## 可用事件

### 命令事件

| 事件 | 描述 |
|------|------|
| `command.executed` | 命令执行后 |

### 文件事件

| 事件 | 描述 |
|------|------|
| `file.edited` | 文件被编辑 |
| `file.watcher.updated` | 文件监控更新 |

### 安装事件

| 事件 | 描述 |
|------|------|
| `installation.updated` | 安装状态更新 |

### LSP 事件

| 事件 | 描述 |
|------|------|
| `lsp.client.diagnostics` | 诊断信息更新 |
| `lsp.updated` | LSP 状态更新 |

### 消息事件

| 事件 | 描述 |
|------|------|
| `message.part.removed` | 消息部分被移除 |
| `message.part.updated` | 消息部分更新 |
| `message.removed` | 消息被删除 |
| `message.updated` | 消息更新 |

### 权限事件

| 事件 | 描述 |
|------|------|
| `permission.asked` | 请求权限 |
| `permission.replied` | 权限响应 |
| `permission.updated` | 权限更新 |

### 服务器事件

| 事件 | 描述 |
|------|------|
| `server.connected` | 服务器连接 |

### 会话事件

| 事件 | 描述 |
|------|------|
| `session.created` | 会话创建 |
| `session.updated` | 会话更新 |
| `session.deleted` | 会话删除 |
| `session.compacted` | 会话压缩 |
| `session.diff` | 会话差异 |
| `session.error` | 会话错误 |
| `session.idle` | 会话空闲 |
| `session.status` | 会话状态变化 |

### 任务事件

| 事件 | 描述 |
|------|------|
| `todo.updated` | 任务更新 |

### 工具事件

| 事件 | 描述 |
|------|------|
| `tool.execute.before` | 工具执行前 |
| `tool.execute.after` | 工具执行后 |

### TUI 事件

| 事件 | 描述 |
|------|------|
| `tui.prompt.append` | 追加提示 |
| `tui.command.execute` | 执行命令 |
| `tui.toast.show` | 显示通知 |

## 示例插件

### 环境保护

阻止读取敏感文件：

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const EnvProtection: Plugin = async ({ project, client }) => {
  return {
    "tool.execute.before": async (input, output) => {
      if (input.tool === "read") {
        const path = output.args.filePath as string
        if (path.includes(".env") || path.includes("credentials")) {
          throw new Error("禁止读取敏感配置文件")
        }
      }
    }
  }
}
```

### 自动日志

记录所有工具调用：

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const AutoLogger: Plugin = async ({ client }) => {
  return {
    "tool.execute.before": async (input) => {
      await client.app.log({
        level: "info",
        message: `执行工具: ${input.tool}`
      })
    },
    "tool.execute.after": async (input, output) => {
      await client.app.log({
        level: "debug",
        message: `工具完成: ${input.tool}, 结果: ${JSON.stringify(output).slice(0, 100)}`
      })
    }
  }
}
```

### 自定义工具

添加项目特定工具：

```typescript
import { type Plugin, tool } from "@opencode-ai/plugin"

export const CustomTools: Plugin = async (ctx) => {
  return {
    tool: {
      deploy: tool({
        description: "部署到生产环境",
        args: {
          environment: tool.schema.enum(["staging", "production"]).describe("目标环境")
        },
        async execute({ environment }) {
          const result = await ctx.$`./deploy.sh ${environment}`
          return result.stdout.toString()
        }
      }),

      lint: tool({
        description: "运行代码检查",
        args: {},
        async execute() {
          const result = await ctx.$`npm run lint`
          return result.stdout.toString()
        }
      })
    }
  }
}
```

### 会话监控

监控会话活动：

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const SessionMonitor: Plugin = async ({ client }) => {
  return {
    "session.created": async (session) => {
      console.log(`新会话: ${session.id}`)
    },
    "session.error": async (error) => {
      await client.app.log({
        level: "error",
        message: `会话错误: ${error.message}`
      })
    }
  }
}
```

### Git 集成

自动提交：

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const GitAutoCommit: Plugin = async ({ $, worktree }) => {
  let editCount = 0

  return {
    "file.edited": async (file) => {
      editCount++

      if (editCount >= 5) {
        await $`cd ${worktree} && git add -A && git commit -m "Auto-commit: ${editCount} files changed"`
        editCount = 0
      }
    }
  }
}
```

### 会话压缩自定义

自定义会话摘要：

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const CustomCompaction: Plugin = async () => {
  return {
    "experimental.session.compacting": async (input, output) => {
      // 添加额外上下文
      output.context.push({
        type: "text",
        text: "重要: 用户偏好使用 TypeScript 和函数式编程风格"
      })

      // 或完全替换提示
      // output.prompt = "自定义压缩提示..."
    }
  }
}
```

## 日志记录

使用 `client.app.log()` 记录日志：

```typescript
await client.app.log({
  level: "debug",  // debug, info, warn, error
  message: "调试信息"
})
```

## TypeScript 支持

安装类型定义：

```bash
npm install -D @opencode-ai/plugin
```

导入类型：

```typescript
import type { Plugin, PluginContext } from "@opencode-ai/plugin"
import { tool } from "@opencode-ai/plugin"
```

## 加载顺序

1. 全局配置
2. 项目配置
3. 全局插件
4. 项目插件

后加载的可以覆盖先加载的。

## 调试插件

### 启用调试日志

```typescript
export const DebugPlugin: Plugin = async ({ client }) => {
  console.log("插件已加载")

  return {
    "*": async (event, data) => {
      console.log(`事件: ${event}`, data)
    }
  }
}
```

### 错误处理

```typescript
export const SafePlugin: Plugin = async ({ client }) => {
  return {
    "tool.execute.before": async (input, output) => {
      try {
        // 插件逻辑
      } catch (error) {
        await client.app.log({
          level: "error",
          message: `插件错误: ${error.message}`
        })
        // 不抛出错误，让工具继续执行
      }
    }
  }
}
```

## 最佳实践

1. **错误处理**: 捕获异常，避免中断主流程
2. **异步安全**: 正确处理 async/await
3. **性能考虑**: 避免在频繁事件中执行耗时操作
4. **日志适度**: 生产环境减少日志输出
5. **类型安全**: 使用 TypeScript 获得更好的开发体验

## 发布插件

### 作为 npm 包

```json
{
  "name": "opencode-my-plugin",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "peerDependencies": {
    "@opencode-ai/plugin": "^1.0.0"
  }
}
```

### 使用

```json
{
  "plugin": ["opencode-my-plugin"]
}
```

## 下一步

- [工具系统](tools.md) - 创建自定义工具
- [SDK 参考](sdk.md) - 使用客户端 API
- [MCP 集成](mcp.md) - 集成外部服务
