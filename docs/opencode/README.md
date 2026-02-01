# OpenCode SDK 文档

> OpenCode 是一个强大的开源 AI 编程助手，专为终端环境构建。

## 概述

OpenCode 是由 SST 团队开发的开源 AI 编程代理，它将 AI 能力带入终端环境。与专有解决方案不同，OpenCode 支持 **75+ LLM 提供商**，包括本地模型，让开发者可以灵活选择 AI 后端。

### 核心特性

- **多提供商支持**: 支持 Anthropic、OpenAI、Google、Azure、本地模型等 75+ 提供商
- **终端原生**: 专为命令行环境设计的 TUI 界面
- **强大的工具系统**: 内置文件操作、Shell 执行、LSP 集成等工具
- **可扩展架构**: 通过插件和 MCP 服务器扩展功能
- **客户端-服务器架构**: 支持多客户端连接和程序化交互
- **类型安全 SDK**: 完整的 TypeScript/JavaScript SDK

## 架构

OpenCode 采用客户端-服务器架构：

```
┌─────────────────────────────────────────────────────────┐
│                    OpenCode Server                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │   REST API  │  │  SSE Events │  │   OpenAPI Spec  │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────┘
         │                  │                   │
    ┌────┴────┐        ┌────┴────┐        ┌────┴────┐
    │   CLI   │        │   TUI   │        │   SDK   │
    └─────────┘        └─────────┘        └─────────┘
```

- **Server**: 基于 Hono 的 HTTP 服务器，提供 REST 和 SSE 端点
- **TUI**: 终端用户界面，作为客户端与服务器通信
- **SDK**: 类型安全的 JS/TS 客户端，用于程序化控制

## 文档目录

| 文档 | 描述 |
|------|------|
| [Serve API 接口文档](../opencode-serve-api/README.md) | 按模块拆分的 REST 接口详细文档（便于查找） |
| [安装指南](installation.md) | 安装和配置 OpenCode |
| [SDK 参考](sdk.md) | TypeScript/JavaScript SDK 完整指南 |
| [提供商配置](providers.md) | 配置各种 AI 提供商 |
| [工具系统](tools.md) | 内置工具和自定义工具 |
| [代理配置](agents.md) | 配置和自定义代理 |
| [Agent 与 Planner 实现讲解](agent-planner-implementation.md) | Agent（ReAct）与 Planner（Plan-and-Execute）的实现要点与自实现清单 |
| [插件开发](plugins.md) | 创建和使用插件 |
| [MCP 集成](mcp.md) | Model Context Protocol 服务器集成 |
| [命令参考](commands.md) | 内置和自定义命令 |

## 快速开始

### 安装

```bash
# 使用 Homebrew (macOS/Linux)
brew install opencode-ai/tap/opencode

# 或使用 npm
npm install -g opencode

# 或使用 curl
curl -fsSL https://opencode.ai/install | bash
```

### 基础使用

```bash
# 在项目目录中启动 OpenCode
cd your-project
opencode

# 指定模型启动
opencode --model anthropic/claude-sonnet-4

# 启动独立服务器
opencode serve --port 4096
```

### SDK 快速示例

```typescript
import { createOpencode } from "@opencode-ai/sdk"

// 创建客户端并自动启动服务器
const { client } = await createOpencode()

// 创建会话
const session = await client.session.create({})

// 发送消息
await client.session.chat(session.id, {
  content: "解释这个代码库的结构"
})
```

## 核心概念

### 会话 (Sessions)

会话是与 AI 的对话上下文，包含消息历史和状态。每个会话独立维护上下文。

### 代理 (Agents)

代理是专门配置的 AI 助手，具有特定的工具和权限：
- **Build**: 默认代理，拥有完整工具访问权限
- **Plan**: 只读代理，用于代码探索和分析

### 工具 (Tools)

工具是 LLM 可以调用的功能：
- 文件操作: `read`, `write`, `edit`, `glob`, `grep`
- Shell 执行: `bash`
- 代码智能: `lsp`
- 任务管理: `todoread`, `todowrite`

### 插件 (Plugins)

插件通过钩子扩展 OpenCode 功能，可以监听事件、添加工具、修改行为。

## 配置文件

OpenCode 使用 `opencode.json` 或 `opencode.jsonc` 进行配置：

```json
{
  "model": "anthropic/claude-sonnet-4",
  "provider": {
    "anthropic": {
      "options": {
        "baseURL": "https://api.anthropic.com/v1"
      }
    }
  },
  "agent": {
    "build": {
      "tools": {
        "bash": true,
        "edit": true
      }
    }
  },
  "permission": {
    "bash": "ask",
    "edit": "allow"
  }
}
```

## 相关资源

- [OpenCode 官方网站](https://opencode.ai)
- [GitHub 仓库](https://github.com/opencode-ai/opencode)
- [SDK 文档](https://opencode.ai/docs/sdk/)
- [Discord 社区](https://discord.gg/opencode)

## 许可证

OpenCode 采用 MIT 许可证开源。
