# rust-langgraph 整合 MCP 方案总览

本文档整合 rust-langgraph 与 MCP（Model Context Protocol）的完整集成方案，涵盖 **Tools** 与 **Resources** 两大能力。

---

## 1. 目标与对应关系

| MCP 能力 | 语义 | rust-langgraph 抽象 | 使用方式 |
|----------|------|---------------------|----------|
| **tools/list** | 发现可用工具 | `ToolSource::list_tools()` | Think 节点拼 prompt |
| **tools/call** | 执行工具调用 | `ToolSource::call_tool(name, args)` | Act 节点执行 |
| **resources/list** | 列出可读资源 | `ResourceSource::list_resources()` | 上下文注入前选择 |
| **resources/read** | 读取资源内容 | `ResourceSource::read_resource(uri)` | 注入到 messages |

**设计原则**：Agent 不直接依赖 MCP 实现，而是依赖抽象接口（ToolSource、ResourceSource），便于测试、mock 和切换实现。

---

## 2. 整体架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ReAct Graph                                  │
│                                                                      │
│  ┌────────────┐    ┌──────────────┐    ┌────────────┐    ┌─────────┐│
│  │  Context   │───▶│    Think     │───▶│    Act     │───▶│ Observe ││
│  │  (可选)    │    │              │    │            │    │         ││
│  └─────┬──────┘    └──────┬───────┘    └─────┬──────┘    └─────────┘│
│        │                  │                  │                       │
│        ▼                  ▼                  ▼                       │
│  ResourceSource     ToolSource::      ToolSource::                   │
│  ::list_resources   list_tools()      call_tool()                    │
│  ::read_resource    (拼 prompt)       (执行调用)                      │
└────────┼──────────────────┼──────────────────┼───────────────────────┘
         │                  │                  │
         ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    MCP Client (McpContext)                           │
│  - tools/list, tools/call                                            │
│  - resources/list, resources/read                                    │
│  - 传输: stdio / HTTP-SSE                                            │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. 第一部分：Tools 集成

### 3.1 现状

- **ToolSource** trait 已定义：`list_tools()`、`call_tool(name, args)`
- **ActNode** 已接 ToolSource，对每个 tool_call 调用 `call_tool`
- **ThinkNode** 未接 ToolSource，仍用固定 prompt（待补充 `list_tools()` 拼工具描述）
- **McpToolSource** 未实现，需对接 MCP `tools/list`、`tools/call`

### 3.2 MCP 协议要点（Tools）

| 方法 | 请求 | 响应 |
|------|------|------|
| tools/list | `{ "method": "tools/list" }` | `{ "result": { "tools": [ { "name", "description", "inputSchema" } ] } }` |
| tools/call | `{ "method": "tools/call", "params": { "name", "arguments" } }` | `{ "result": { "content": [ { "type": "text", "text": "..." } ] } }` |

### 3.3 实现任务

| 项 | 状态 | 说明 |
|----|------|------|
| McpToolSource | 待实现 | 发 tools/list、tools/call，实现 ToolSource |
| MCP JSON-RPC 层 | 待实现 | 请求/响应序列化与解析 |
| MCP stdio 传输 | 待实现 | 子进程 spawn，按行读写 JSON-RPC |
| ThinkNode 接 ToolSource | 待实现 | 从 list_tools() 取描述拼进 prompt |

详见 [implementation.md](implementation.md)。

---

## 4. 第二部分：Resources 集成

### 4.1 语义区分

- **Tool**：可调用能力，LLM 通过 tool_calls 主动发起
- **Resource**：只读数据，作为上下文注入到对话，不作为工具暴露

### 4.2 推荐方案：Resource 作为上下文注入

| 要素 | 设计 |
|------|------|
| 抽象 | 新增 **ResourceSource**：`list_resources()`、`read_resource(uri)` |
| 注入时机 | Think 之前：新增 **Context 节点**，或 Think 内先拉 resource 再调 LLM |
| 注入形式 | 将 `read_resource(uri)` 的文本作为 **System 或 User 消息** 追加到 `state.messages` |
| ToolSource | 不变，仅负责 tools |

**不推荐**：把 Resource 暴露为「只读工具」——会导致工具列表膨胀、轮次增加。

### 4.3 实现任务

| 项 | 状态 | 说明 |
|----|------|------|
| ResourceSource trait | 待实现 | list_resources()、read_resource(uri) |
| mcp-client read_resource | 待实现 | 发 resources/read，解析 ReadResourceResult |
| McpResourceSource | 待实现 | 依赖 mcp-client 的 list + read |
| Context 节点或 Think 内注入 | 待实现 | Think 前拉 resource 并拼进 messages |

详见 [resource-integration.md](resource-integration.md)。

---

## 5. 统一 MCP 入口（可选）

若希望**一个 MCP 连接**同时提供 tools 与 resources：

- 引入 **McpContext**：内部持有一个 MCP client
- 实现 `ToolSource`（tools/list、tools/call）和 `ResourceSource`（resources/list、resources/read）
- ReAct 持有同一 `McpContext` 作为 ToolSource 与 ResourceSource

```
McpContext
├── implements ToolSource    → Think/Act 使用
└── implements ResourceSource → Context 节点使用
```

---

## 6. 实施顺序建议

1. **Tools 先落地**：McpToolSource + Think 接 ToolSource，跑通 ReAct + MCP 工具
2. **Resources 后补**：ResourceSource + McpResourceSource + Context 节点
3. **可选统一**：McpContext 同时实现双 Source

---

## 7. 文档索引

| 文档 | 内容 |
|------|------|
| [README.md](README.md) | MCP 替代 Tool 的方案概述 |
| [implementation.md](implementation.md) | Tools 实现的类型、模块、任务表 |
| [resource-integration.md](resource-integration.md) | Resources 整合方案与任务表 |
| **overview.md**（本文） | 整合总览 |
