# 用 MCP 替代 Tool 的最简集成方案

在 [09-minimal-agent-design](../09-minimal-agent-design.md) 的扩展路径中，S4/S5 涉及「工具」：ReAct 调工具、多工具协作。本文给出**用 MCP（Model Context Protocol）作为工具来源、代替自建 Tool 生态**的最简集成方案：Agent 不直接依赖 `Tool` trait 与 `ToolRegistry`，改为通过 MCP 协议从 MCP Server 获取工具列表并执行调用。

**本目录**：方案概述（本文）+ [implementation.md](implementation.md)（实现说明与任务表）+ [context-optimization.md](context-optimization.md)（多工具上下文优化方案）。

---

## 1. 目标与对应关系

| 要素 | 自建 Tool（当前 05-tools） | MCP 集成（本方案） |
|------|---------------------------|---------------------|
| 工具发现 | `ToolRegistry::list()`、本地 `Tool` 实现 | MCP `tools/list`，由 MCP Server 提供 |
| 工具描述 | `name()`、`description()`、`schema()` | MCP 返回 `name`、`description`、`inputSchema` |
| 工具调用 | `ToolRegistry::invoke(ToolCall)` | MCP `tools/call`，参数 `name` + `arguments` |
| 扩展方式 | 每个工具实现 `Tool` trait、注册到 Registry | 任意 MCP Server 暴露工具，客户端仅做协议层 |

**设计目标**：保持 [09-minimal-agent-design](../09-minimal-agent-design.md) 与 [11-state-graph-design](../11-state-graph-design.md) 不变；在「需要工具」的节点（如 ReAct 的 Act 节点）上，**工具来源**抽象为「可列出、可调用」的接口，默认或推荐实现为 **MCP 客户端**，从而用 MCP 代替自建 Tool 生态，减少重复实现、对齐业界协议。

---

## 2. MCP 协议要点（工具部分）

MCP 使用 JSON-RPC 2.0。与工具相关的方法：

- **`tools/list`**：发现可用工具。  
  - 请求：`{ "method": "tools/list", "params": { "cursor": "optional" } }`  
  - 响应：`{ "result": { "tools": [ { "name", "description", "inputSchema" }, ... ], "nextCursor" } }`

- **`tools/call`**：执行一次工具调用。  
  - 请求：`{ "method": "tools/call", "params": { "name": "tool_name", "arguments": { ... } } }`  
  - 响应：`{ "result": { "content": [ { "type": "text", "text": "..." } ], "isError": false } }`

传输层可以是 stdio 或 HTTP/SSE，与具体 MCP Server 配置一致。最简集成时，先支持一种（如 stdio）即可。

---

## 3. 最简集成方案

### 3.1 抽象：工具来源（Tool Source）

不要求 ReAct/Agent 直接依赖 `ToolRegistry`，而是依赖一个「能列工具、能调工具」的抽象，便于同一套 Agent 逻辑既可接自建 Tool，也可接 MCP。

```text
概念层：
  ToolSource: list_tools() -> Vec<ToolSpec>
              call_tool(name, args) -> Result<Content, ToolSourceError>
```

- **ToolSpec**：与 MCP 对齐，至少包含 `name`、`description`、`inputSchema`（JSON Schema 或等价），供 LLM/ReAct 生成工具调用。
- **call_tool**：与 MCP `tools/call` 对齐，返回内容为文本或结构化（如 MCP 的 `content[].text`）。

实现体可以是：

- **ToolRegistryAdapter**：包装现有 `ToolRegistry`，实现 `ToolSource`（从现有 `Tool` 取 name/description/schema，invoke 时转成 call_tool）。
- **McpToolSource**：MCP 客户端，通过 stdio（或 SSE）连接 MCP Server，`list_tools()` 发 `tools/list`，`call_tool(name, args)` 发 `tools/call`。

这样，**用 MCP 代替 Tool** 体现为：ReAct/Agent 使用 `McpToolSource` 作为默认或可选的工具来源，不再必须实现并注册大量 `Tool`。

### 3.2 Agent/ReAct 侧改动（最小）

- **ReAct 的 Think 节点**：从 `ToolSource::list_tools()` 取工具列表，拼进 prompt（与当前从 `ToolRegistry` 取描述一致）。
- **ReAct 的 Act 节点**：收到 LLM 产出的 tool_calls 后，对每条调用 `ToolSource::call_tool(name, args)`，得到结果再交给 Observe；若来源是 MCP，即走 `tools/call`。
- **State**：无需改状态结构；tool_calls 与 results 仍为当前 ReAct 已有形态（如 name + arguments + result 文本）。

即：仅把「从哪里拿工具列表、从哪里执行调用」从 `ToolRegistry` 换成 `ToolSource`，其中一种实现是 MCP 客户端。

### 3.3 MCP 客户端职责（最简）

- 建立与 MCP Server 的会话（如 stdio 子进程或 HTTP/SSE）。
- 实现 JSON-RPC 2.0 的请求/响应（id、method、params、result/error）。
- 实现 `ToolSource`：
  - `list_tools()`：发 `tools/list`，解析 `result.tools`，映射为 `Vec<ToolSpec>`。
  - `call_tool(name, arguments)`：发 `tools/call`，解析 `result.content`（如取 `type: "text"` 的 `text`），返回给 Act 节点。

不要求本阶段实现 resources、prompts、listChanged 等，仅 tools 即可。

---

## 4. 与 09 最简设计的关系

- **Agent / StateGraph**：不变；仍为 state 进、state 出，线性链或多节点。
- **扩展路径 S4/S5**：  
  - S4（ReAct + 工具）：工具来源可以是 `ToolRegistry`（现有）或 **MCP**（本方案）。  
  - S5（工具生态）：优先通过 **接入多个 MCP Server 或同一 Server 多工具** 扩展，不再强制为每个能力写一个 Rust `Tool` 实现。
- **记忆、流式、多 Agent**：不受影响；MCP 只替代「工具发现与调用」这一块。

即：用 MCP 代替 Tool 是**在最小 Agent 设计之上的可选/推荐扩展**，不改变 09 的核心抽象。

---

## 5. 实现与任务

详见 [implementation.md](implementation.md)：类型定义、模块划分、任务表与验收。开发计划与任务表见 [mcp-tool-devplan.md](mcp-tool-devplan.md)。

**示例**：

- **react_mcp**：McpToolSource 连接本地 stdio MCP（如 mcp-filesystem-server），`--features mcp`。
- **react_mcp_gitlab**：连接 GitLab MCP，需 `GITLAB_TOKEN`，`--features mcp`。
- **react_exa**：ReAct + Exa MCP（网络搜索），经 `npx mcp-remote https://mcp.exa.ai/mcp` 连接 Exa 托管 MCP；需 `ZHIPU_API_KEY`，`--features zhipu`（mcp 已默认开启）。见 [Exa MCP](https://github.com/exa-labs/exa-mcp-server)。

依赖：[09-minimal-agent-design](../09-minimal-agent-design.md)、[11-state-graph-design](../11-state-graph-design.md)。与 [05-tools](../05-tools.md) 的关系：05 保留为「自建工具」实现；本方案通过 ToolSource + ToolRegistryAdapter 复用 05，同时用 MCP 作为推荐的工具扩展方式。

---

## 6. 小结

- **用 MCP 代替 Tool**：不删掉现有 Tool/ToolRegistry，而是引入 **ToolSource** 抽象，由 **McpToolSource** 提供「工具列表 + 工具调用」，ReAct/Agent 只依赖 ToolSource。
- **最简集成**：MCP 客户端只实现 `tools/list` 与 `tools/call`，先支持 stdio 传输；ReAct 的 Think/Act 改为使用 ToolSource，即可用任意 MCP Server 作为工具生态，无需为每个能力写 Rust Tool。
- 与 09 一致：每一步保持「当前可运行的最简形态」；先接一个 MCP Server 或 mock 跑通，再扩展多 Server、resources 等。
