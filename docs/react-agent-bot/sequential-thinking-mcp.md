# Sequential Thinking MCP 说明

Sequential Thinking 是 **Model Context Protocol (MCP)** 官方仓库提供的 MCP Server，通过结构化思考过程提供**动态、可反思的多步问题分解与推理**能力。适合与 ReAct Agent 结合，用于复杂问题的分步规划与子任务分解。

---

## 1. 概述

| 项目 | 说明 |
|------|------|
| **类型** | 官方 MCP Server（[modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers)） |
| **包名** | `@modelcontextprotocol/server-sequential-thinking` |
| **能力** | 动态问题分解、多步推理、子任务规划、分支/修订思考链 |
| **协议** | MCP（stdio 或 Docker） |

---

## 2. 核心能力

- **分步分解**：将复杂问题拆成可管理的、带编号的连续思考步骤。
- **修订与精炼**：在理解加深后修订之前的思考（`isRevision` / `revisesThought`）。
- **分支推理**：从某一步分支到不同思路（`branchFromThought` / `branchId`），同时探索多种方案。
- **动态调整总步数**：可随时增加「还需要更多步」（`needsMoreThoughts`），不必一开始定死总步数。
- **假设生成与验证**：在推理链上生成并验证假设。
- **非线式推理**：支持回溯、质疑先前决策，而不只是单线顺序。

---

## 3. 工具：`sequential_thinking`

唯一对外工具，用于推进「当前一步」的思考并可选地标记修订/分支。

### 输入参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `thought` | string | 是 | 当前这一步的思考内容 |
| `nextThoughtNeeded` | boolean | 是 | 是否还需要下一步思考 |
| `thoughtNumber` | integer | 是 | 当前是第几步（从 1 开始） |
| `totalThoughts` | integer | 是 | 预估总共需要的步数 |
| `isRevision` | boolean | 否 | 是否为对先前某步的修订 |
| `revisesThought` | integer | 否 | 被修订的是第几步 |
| `branchFromThought` | integer | 否 | 从第几步开始分支 |
| `branchId` | string | 否 | 分支标识，用于区分多条推理路径 |
| `needsMoreThoughts` | boolean | 否 | 是否需要在原计划外增加更多步 |

### 使用方式

由 MCP 客户端（如 Claude Desktop、Cursor、自研 ReAct Agent）调用 `sequential_thinking`，按步传入 `thought` 及上述元数据；Server 维护思考链状态，支持修订与分支。

---

## 4. 适用场景

- 需要**把复杂问题拆成多步**并显式规划时。
- **规划与设计**且允许中途修订的方案时。
- **分析类任务**可能中途改向、纠偏时。
- 问题**范围一开始不清晰**，需要边想边调整总步数时。
- 需要**在多步之间保持上下文**的连续任务。
- 需要在大量信息中**过滤无关内容**、只保留与当前推理相关的部分时。

与 [ideas.md](ideas.md) 中「MCP 生态扩展」一致：可作为官方 MCP 之一，为 ReAct Agent 提供「动态问题分解与多步推理」的补充能力。

---

## 5. 安装与配置

### 5.1 使用 npx（推荐）

**Claude Desktop**（`claude_desktop_config.json`）：

```json
{
  "mcpServers": {
    "sequential-thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
    }
  }
}
```

**VS Code / Cursor**（用户或工作区 `mcp.json`）：

```json
{
  "servers": {
    "sequential-thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
    }
  }
}
```

### 5.2 使用 Docker

```json
{
  "mcpServers": {
    "sequential-thinking": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "mcp/sequentialthinking"]
    }
  }
}
```

### 5.3 环境变量

- **`DISABLE_THOUGHT_LOGGING=true`**：关闭思考内容的日志输出（若担心敏感或冗长信息落盘可启用）。

---

## 6. 与 ReAct Agent 的集成思路

- 在 **AgentConfig** 或「工具源」预设中增加 Sequential Thinking 作为可选 MCP Server。
- 与 Exa、Fetch、Memory、Filesystem 等 MCP 并列，由 **多 ToolSource 聚合** 统一 `list_tools` / `call_tool`。
- 当用户问题偏规划、分解、多步推理时，由 Agent 决定是否调用 `sequential_thinking` 做显式思考链，再基于思考结果调用其他工具（如搜索、读文件等）。

---

## 7. 参考链接

- [npm: @modelcontextprotocol/server-sequential-thinking](https://www.npmjs.com/package/@modelcontextprotocol/server-sequential-thinking)
- [GitHub: modelcontextprotocol/servers — sequentialthinking](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking)
- [Model Context Protocol 官网](https://modelcontextprotocol.io)
