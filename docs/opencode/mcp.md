# MCP 集成

Model Context Protocol (MCP) 是连接 AI 模型与外部工具和数据源的开放标准。OpenCode 原生支持 MCP 服务器集成。

## 什么是 MCP

MCP 可以理解为 "AI 应用的 USB-C" —— 一个统一的标准，让 AI 可以连接 GitHub、数据库、Slack 等外部服务，无需每个 AI 编辑器都构建自己的集成。

### MCP 架构

```
┌─────────────────┐     ┌─────────────────┐
│   OpenCode      │────▶│   MCP Server    │
│   (Client)      │◀────│   (Tools)       │
└─────────────────┘     └─────────────────┘
         │                      │
         │              ┌───────┴───────┐
         │              │               │
         ▼              ▼               ▼
    [AI Model]     [GitHub]      [Database]
```

## 配置 MCP 服务器

在 `opencode.json` 的 `mcp` 部分配置：

```json
{
  "mcp": {
    "server-name": {
      "type": "local",
      "command": ["command", "args"],
      "enabled": true
    }
  }
}
```

## 本地服务器

本地服务器作为子进程运行，使用 stdio 通信。

### 基础配置

```json
{
  "mcp": {
    "everything": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-everything"]
    }
  }
}
```

### 带环境变量

```json
{
  "mcp": {
    "github": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "environment": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

### 常用本地 MCP 服务器

#### 文件系统

```json
{
  "mcp": {
    "filesystem": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "/path/to/dir"]
    }
  }
}
```

#### GitHub

```json
{
  "mcp": {
    "github": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "environment": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

#### PostgreSQL

```json
{
  "mcp": {
    "postgres": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-postgres"],
      "environment": {
        "DATABASE_URL": "${DATABASE_URL}"
      }
    }
  }
}
```

#### Puppeteer (浏览器自动化)

```json
{
  "mcp": {
    "puppeteer": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-puppeteer"]
    }
  }
}
```

## 远程服务器

远程服务器通过 HTTP/SSE 通信，支持 OAuth 认证。

### 基础配置

```json
{
  "mcp": {
    "remote-service": {
      "type": "remote",
      "url": "https://api.example.com/mcp"
    }
  }
}
```

### 带 API Key

```json
{
  "mcp": {
    "context7": {
      "type": "remote",
      "url": "https://mcp.context7.com",
      "headers": {
        "Authorization": "Bearer ${CONTEXT7_API_KEY}"
      },
      "oauth": false
    }
  }
}
```

### OAuth 认证

OpenCode 自动处理 OAuth 流程：

```json
{
  "mcp": {
    "sentry": {
      "type": "remote",
      "url": "https://mcp.sentry.io"
    }
  }
}
```

#### 预注册凭据

```json
{
  "mcp": {
    "my-service": {
      "type": "remote",
      "url": "https://api.example.com/mcp",
      "oauth": {
        "clientId": "your-client-id",
        "clientSecret": "your-client-secret",
        "scope": "read write"
      }
    }
  }
}
```

#### 手动认证

```bash
opencode mcp auth sentry
```

#### Token 存储

OAuth token 存储在 `~/.local/share/opencode/mcp-auth.json`。

## 服务器管理

### 启用/禁用

```json
{
  "mcp": {
    "github": {
      "type": "local",
      "command": ["npx", "@modelcontextprotocol/server-github"],
      "enabled": false
    }
  }
}
```

### 超时配置

```json
{
  "mcp": {
    "slow-server": {
      "type": "local",
      "command": ["./slow-server"],
      "timeout": 30000
    }
  }
}
```

默认超时 5000ms。

### 全局禁用

```json
{
  "tools": {
    "github_*": false,
    "my-mcp_*": false
  }
}
```

### 按代理配置

全局禁用但为特定代理启用：

```json
{
  "tools": {
    "github_*": false
  },
  "agent": {
    "github-expert": {
      "tools": {
        "github_*": true
      }
    }
  }
}
```

## 使用 MCP 工具

### 在提示中

```
使用 github 创建一个新的 issue
```

### 明确指定

```
使用 mcp:github 的 create_issue 工具
```

### 查看可用工具

```
/tools
```

## 示例配置

### 完整开发环境

```json
{
  "mcp": {
    "github": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "environment": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}"
      }
    },
    "postgres": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-postgres"],
      "environment": {
        "DATABASE_URL": "postgresql://localhost:5432/mydb"
      }
    },
    "sentry": {
      "type": "remote",
      "url": "https://mcp.sentry.io"
    },
    "context7": {
      "type": "remote",
      "url": "https://mcp.context7.com",
      "headers": {
        "Authorization": "Bearer ${CONTEXT7_API_KEY}"
      },
      "oauth": false
    }
  }
}
```

### 本地开发

```json
{
  "mcp": {
    "filesystem": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "./data"]
    },
    "sqlite": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-sqlite", "./db.sqlite"]
    }
  }
}
```

## 创建 MCP 服务器

### 使用官方 SDK

```bash
npm install @modelcontextprotocol/sdk
```

### 基础服务器

```typescript
import { Server } from "@modelcontextprotocol/sdk/server"
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio"

const server = new Server({
  name: "my-server",
  version: "1.0.0"
})

// 注册工具
server.setRequestHandler("tools/list", async () => {
  return {
    tools: [{
      name: "hello",
      description: "Say hello",
      inputSchema: {
        type: "object",
        properties: {
          name: { type: "string" }
        }
      }
    }]
  }
})

server.setRequestHandler("tools/call", async (request) => {
  if (request.params.name === "hello") {
    return {
      content: [{
        type: "text",
        text: `Hello, ${request.params.arguments.name}!`
      }]
    }
  }
})

// 启动服务器
const transport = new StdioServerTransport()
await server.connect(transport)
```

## 注意事项

### 上下文限制

> MCP 服务器会增加上下文，谨慎启用，过多工具会快速超出上下文限制。

### 安全考虑

- 只启用必要的 MCP 服务器
- 使用环境变量存储敏感凭据
- 定期轮换 OAuth token
- 审查 MCP 服务器的权限范围

### 性能

- 本地服务器启动需要时间
- 设置合理的超时时间
- 考虑使用远程服务器提高性能

## 故障排除

### 服务器无法启动

```bash
# 手动测试命令
npx -y @modelcontextprotocol/server-github

# 检查环境变量
echo $GITHUB_TOKEN
```

### OAuth 失败

```bash
# 清除 token 重新认证
rm ~/.local/share/opencode/mcp-auth.json
opencode mcp auth server-name
```

### 工具未显示

1. 检查服务器是否 `enabled: true`
2. 检查全局 `tools` 配置没有禁用
3. 检查代理的工具配置

## 资源

- [MCP 官方规范](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP 服务器目录](https://github.com/modelcontextprotocol/servers)
- [MCP SDK](https://github.com/modelcontextprotocol/sdk)

## 下一步

- [工具系统](tools.md) - 了解内置工具
- [插件开发](plugins.md) - 使用插件扩展
- [代理配置](agents.md) - 配置代理工具访问
