# MCP 集成实现说明

本文档给出 [README](README.md) 方案在 rust-langgraph 中的**实现规划**：类型定义、模块划分、任务表与验收。开发时按表格推进，完成后标记。

---

## 1. 类型与 API

### 1.1 ToolSource 抽象

```rust
/// Tool specification, aligned with MCP tools/list result item.
/// Used by ReAct/Think to build tool descriptions for the LLM.
pub struct ToolSpec {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,  // JSON Schema
}

/// Result of a single tool call; aligns with MCP tools/call content.
pub struct ToolCallContent {
    pub text: String,
    // optional: other content types (image, etc.) later
}

/// Errors from listing or calling tools (ToolSource or MCP).
#[derive(Debug, thiserror::Error)]
pub enum ToolSourceError {
    #[error("tool not found: {0}")]
    NotFound(String),
    #[error("invalid arguments: {0}")]
    InvalidInput(String),
    #[error("MCP/transport error: {0}")]
    Transport(String),
    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),
}

/// Tool source: list tools and call a tool. ReAct/Agent depends on this instead of ToolRegistry.
#[async_trait]
pub trait ToolSource: Send + Sync {
    /// List available tools (e.g. MCP tools/list).
    async fn list_tools(&self) -> Result<Vec<ToolSpec>, ToolSourceError>;

    /// Call a tool by name with JSON arguments (e.g. MCP tools/call).
    async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<ToolCallContent, ToolSourceError>;
}
```

### 1.2 ToolRegistryAdapter

包装现有 `ToolRegistry`，实现 `ToolSource`，使现有 ReAct 可无缝切换为「工具来源」驱动：

- `list_tools()`：遍历 registry，对每个 `Tool` 取 `name()`、`description()`、`schema()` 构造 `ToolSpec`。
- `call_tool(name, args)`：构造 `ToolCall { name, args, id }`，调用 `registry.invoke(&call)`，将 `serde_json::Value` 结果转为 `ToolCallContent { text: value.to_string() }` 或按约定取文本字段。

### 1.3 McpToolSource（MCP 客户端）

- **传输**：最简先支持 stdio（子进程 stdin/stdout，按行或按长度读 JSON-RPC 消息）。
- **协议**：JSON-RPC 2.0，请求带 `id`、`method`、`params`；响应解析 `result` 或 `error`。
- **list_tools()**：发 `tools/list`，解析 `result.tools` 为 `Vec<ToolSpec>`（`name`、`description`、`inputSchema`）。
- **call_tool(name, arguments)**：发 `tools/call`，解析 `result.content`，取 `type: "text"` 的 `text` 填入 `ToolCallContent`。

可选：同一 crate 内 `mcp/` 子模块，或独立 `langgraph-mcp` crate，视依赖与复用范围定。

---

## 2. 模块与文件布局

建议在 `rust-langgraph/crates/langgraph/src/` 下：

| 路径 | 职责 |
|------|------|
| `tool_source/mod.rs` | `ToolSource` trait、`ToolSpec`、`ToolCallContent`、`ToolSourceError` |
| `tool_source/registry_adapter.rs` | `ToolRegistryAdapter` 实现 `ToolSource`，依赖现有 `tool/` |
| `tool_source/mcp/mod.rs` | MCP 客户端入口、`McpToolSource` 实现 `ToolSource` |
| `tool_source/mcp/stdio.rs` | stdio 传输：子进程 spawn、读写 JSON-RPC 行 |
| `tool_source/mcp/jsonrpc.rs` | JSON-RPC 2.0 请求/响应构建与解析 |

若 MCP 依赖较多或希望独立发布，可改为：

- `rust-langgraph/crates/langgraph-mcp/`：`ToolSource` trait 由 langgraph 定义并 re-export；`langgraph-mcp` 实现 `McpToolSource`，依赖 `langgraph` 的 `ToolSpec`/`ToolCallContent`/`ToolSourceError`。

ReAct 包（如 `langgraph-react`）仅依赖 `ToolSource`，不直接依赖 MCP 实现，便于测试时注入 mock。

---

## 3. ReAct 侧改动

- **Think 节点**：入参增加 `&dyn ToolSource`（或 `Arc<dyn ToolSource>`）；`build_prompt` 时调用 `tool_source.list_tools().await?`，用返回的 `Vec<ToolSpec>` 生成工具描述（与当前从 `ToolRegistry` 生成方式一致）。
- **Act 节点**：入参改为 `&dyn ToolSource`；对每个 tool call 调用 `tool_source.call_tool(name, arguments).await?`，将 `ToolCallContent::text` 填入观察结果。
- **示例与测试**：可配置 `ToolRegistryAdapter(registry)` 或 `McpToolSource`；至少一个示例仅用 MCP（或 mock MCP Server）跑通。

---

## 4. 任务表

| 项 | 状态 | 说明 |
|----|------|------|
| ToolSpec / ToolCallContent / ToolSourceError | 待实现 | 定义于 `tool_source/mod.rs` |
| ToolSource trait | 待实现 | `list_tools()`、`call_tool(name, args)` |
| ToolRegistryAdapter | 待实现 | 包装 ToolRegistry，实现 ToolSource |
| MCP JSON-RPC 层 | 待实现 | 请求/响应序列化与解析，`jsonrpc.rs` |
| MCP stdio 传输 | 待实现 | 子进程 + 按行读写，`stdio.rs` |
| McpToolSource | 待实现 | 实现 ToolSource，调用 tools/list、tools/call |
| ReAct 接 ToolSource | 待实现 | Think/Act 使用 ToolSource 替代直接 ToolRegistry |
| 示例：ReAct + MCP | 待实现 | 一个 MCP Server（或 mock）+ ReAct 仅用 MCP 工具跑通 |

每项完成后在「状态」列改为「已完成」，并在本仓库对应代码处补充注释引用本方案（如 `mcp-integration/implementation.md`）。

---

## 5. 验收

- 单元测试：`ToolRegistryAdapter` 对现有 `ToolRegistry` 做 `list_tools`/`call_tool`，结果与直接 `registry.list`/`registry.invoke` 一致。
- 集成：ReAct 示例可配置为仅使用 `McpToolSource`（对接真实或 mock MCP Server），问一句需调工具的问题，能返回工具结果。
- 与 09 一致：不改变 Agent/StateGraph 的 state 形态；仅工具来源从 Registry 抽象为 ToolSource，MCP 为一种实现。
