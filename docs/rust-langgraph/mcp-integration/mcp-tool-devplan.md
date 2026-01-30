# MCP Tool 集成开发计划

仅集成 MCP **tools**（`tools/list`、`tools/call`），不涉及 Resources。

**复用根目录已有实现**：`crates/mcp-client`、`crates/mcp-core`（workspace 成员），不再自建 JSON-RPC 与 stdio 传输。

---

## 1. 现状

| 项 | 状态 |
|----|------|
| ToolSource trait、ToolSpec、ToolCallContent、ToolSourceError | ✅ 已实现 |
| ActNode 使用 ToolSource::call_tool | ✅ 已实现 |
| react_zhipu 示例：list_tools → with_tools，Act 用 call_tool | ✅ 已跑通 |
| mcp_client、mcp_core（workspace） | ✅ 已有 |
| McpToolSource（包装 mcp_client） | ❌ 待实现 |

当前示例流程：`tool_source.list_tools()` 在构建时调用 → 结果传给 `ChatZhipu::with_tools` → ActNode 用同一 `tool_source` 执行 `call_tool`。ThinkNode 无需改动。

---

## 2. 已有 MCP 实现参考

| 路径 | 说明 |
|------|------|
| `crates/mcp-client` | StdioClientTransport、stdio 传输、JSON-RPC 已实现 |
| `crates/mcp-core` | Tool、CallToolResult、ContentBlock 等类型 |
| `examples/mcp-filesystem-client` | 低阶用法：initialize → tools/list → tools/call |
| `mcp-impls/gitlab-mcp/crates/mcp-client` | McpServerClient：`list_tools()`、`call_tool(name, args)` 同步封装 |

McpToolSource 参考 gitlab 模式：`StdioClientTransport` + `mpsc` 收消息 + `send_request` + `wait_for_result`。

---

## 3. 开发任务（树形拆解，细致表格）

开发按阶段推进，每项完成后将「状态」改为「已完成」，并在对应代码处补充注释引用本文（如 `mcp-integration/mcp-tool-devplan.md`）。

### 3.1 阶段一：依赖与 workspace 配置

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 1.1 | langgraph 添加 mcp feature | `[features] mcp = ["dep:mcp_client", "dep:mcp_core"]`；`[dependencies]` 下 optional 依赖 | 已完成 | 依赖 workspace 已有 mcp_client、mcp_core |
| 1.2 | 构建验证 | `cargo build -p langgraph --features mcp` 通过 | 已完成 | 依赖 1.1 |

### 3.2 阶段二：McpSession 底层封装

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 2.1 | McpSession 结构 | `struct McpSession { transport, receiver }`；`new(command, args) -> Result<Self>` | 已完成 | 依赖 1.2；`tool_source/mcp/session.rs`；持 StdioClientTransport + mpsc::Receiver |
| 2.2 | transport 启动与 on_message | `StdioClientTransport::new(params).on_message(...).start()`；消息推入 channel | 已完成 | 依赖 2.1；参考 mcp-filesystem-client、gitlab McpServerClient |
| 2.3 | initialize 握手 | `initialize()`：发 `initialize` 请求 → 收 result → 发 `notifications/initialized`；协议版本 `2025-11-25` | 已完成 | 依赖 2.2；`new()` 内调用，失败则返回 Err |
| 2.4 | send_request | `send_request(id, method, params)`：构造 RequestMessage，transport.send(JsonRpcMessage::Request) | 已完成 | 依赖 2.2 |
| 2.5 | wait_for_result | `wait_for_result(id, timeout) -> Result<Option<ResultMessage>>`：从 receiver 循环 recv 直到匹配 id 或超时；处理 roots/list 等 server 请求 | 已完成 | 依赖 2.4；参考 mcp-filesystem-client wait_for_result |
| 2.6 | close | `close()`：transport.close()，回收资源 | 已完成 | 依赖 2.1 |
| 2.7 | McpSession 单元测试 | `tests/mcp_session.rs`：spawn mcp-filesystem-server，执行 initialize → tools/list → tools/call，断言返回结构 | 已完成 | 依赖 2.5, 2.6；`#[ignore]`，run with `--ignored` |

### 3.3 阶段三：McpToolSource 实现

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 3.1 | McpToolSource 结构 | `struct McpToolSource { session: McpSession }`；`new(command, args) -> Result<Self>` | 已完成 | 依赖 2.7；`tool_source/mcp/mod.rs`；session 用 Mutex 包装 |
| 3.2 | Tool → ToolSpec 映射 | 解析 tools/list 的 `result.tools`；每个 `{ name, description?, inputSchema }` → `ToolSpec { name, description, input_schema }` | 已完成 | 依赖 3.1；`list_tools` 内部 |
| 3.3 | list_tools 实现 | 发 tools/list，wait_for_result，解析 result.tools，映射为 `Vec<ToolSpec>`；错误映射为 ToolSourceError | 已完成 | 依赖 3.2 |
| 3.4 | CallToolResult → ToolCallContent 映射 | 解析 tools/call 的 result；取 `content[]` 中 `type == "text"` 的 `text`，或 `structuredContent` 序列化 → `ToolCallContent { text }` | 已完成 | 依赖 3.1；`call_tool` 内部 |
| 3.5 | call_tool 实现 | 发 tools/call `{ name, arguments }`，wait_for_result，按 3.4 映射；错误映射为 ToolSourceError | 已完成 | 依赖 3.4 |
| 3.6 | 异步包装 | `async fn list_tools` / `call_tool` 内用 `tokio::task::block_in_place` 包装同步 MCP 调用，避免阻塞 runtime | 已完成 | 依赖 3.3, 3.5 |
| 3.7 | 实现 ToolSource trait | `#[async_trait] impl ToolSource for McpToolSource`；`list_tools`、`call_tool` 符合 trait 签名 | 已完成 | 依赖 3.6 |
| 3.8 | McpToolSource 集成测试 | `tests/mcp_tool_source.rs`：用 McpToolSource 连接 mcp-filesystem-server，`list_tools().await`、`call_tool("list_directory", args).await`，断言 ToolSpec 与 ToolCallContent | 已完成 | 依赖 3.7；`#[ignore]`，需 `#[tokio::test(flavor = "multi_thread")]` |

### 3.4 阶段四：导出与文档

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 4.1 | 条件导出 | `tool_source/mod.rs`：`#[cfg(feature = "mcp")] mod mcp;`；`#[cfg(feature = "mcp")] pub use mcp::McpToolSource;` | 已完成 | 依赖 3.7 |
| 4.2 | lib 导出 | `lib.rs`：`#[cfg(feature = "mcp")] pub use tool_source::McpToolSource;` | 已完成 | 依赖 4.1 |
| 4.3 | 注释与文档 | McpSession、McpToolSource、各方法含英文 doc comment；说明交互类型（ToolSource、StdioClientTransport）；引用 mcp-tool-devplan | 已完成 | 依赖 4.2；符合 AGENTS.md §Rust |

### 3.5 阶段五：示例与收尾

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 5.1 | react_mcp 示例 | `examples/react_mcp.rs`：McpToolSource 连接 mcp-filesystem-server；ReAct 图 think→act→observe；用户问「当前目录有哪些文件」 | 已完成 | 依赖 4.3, 3.8；用 MockLlm 返回 list_directory，`--features mcp` |
| 5.2 | 示例文档 | example 顶部 doc 说明运行方式、环境变量（如 MCP_SERVER_COMMAND）、依赖的 MCP Server | 已完成 | 依赖 5.1 |
| 5.3 | 文档同步 | implementation.md 任务表与本文对齐；本文任务表全部勾选完成 | 已完成 | 依赖 5.2 |
| 5.4 | react_exa 示例 | `examples/react_exa.rs`：McpToolSource 经 mcp-remote 连接 Exa 托管 MCP（https://mcp.exa.ai/mcp）；ChatZhipu + 真实工具（web_search_exa 等）；`--features zhipu`（mcp 默认） | 已完成 | 需 npx/mcp-remote、ZHIPU_API_KEY；见 [Exa MCP](https://github.com/exa-labs/exa-mcp-server) |

**表使用说明**：按阶段顺序执行；阶段 N 依赖阶段 N-1 的完成项。每项完成后在「状态」列改为「已完成」。

---

## 4. 文件布局

```
langgraph/src/tool_source/
├── mod.rs          # 按 feature 导出 McpToolSource
├── mock.rs         # MockToolSource（已有）
└── mcp/
    ├── mod.rs      # McpToolSource，实现 ToolSource
    └── session.rs  # McpSession：StdioClientTransport + initialize + request/response 同步封装

langgraph/tests/
├── mcp_session.rs      # McpSession 单元/集成测试
└── mcp_tool_source.rs  # McpToolSource 集成测试
```

直接使用 `mcp_client::stdio`，不新增 jsonrpc/stdio 模块。

---

## 5. 技术要点

- **异步**：StdioClientTransport 为同步 I/O，在 `async fn` 内用 `tokio::task::block_in_place` 包装，避免阻塞 tokio runtime。
- **类型映射**：`mcp_core::types::Tool` → `ToolSpec`（name, description, inputSchema）；`CallToolResult.content` 中 `type: "text"` 的 `text` → `ToolCallContent::text`。
- **初始化**：发 `initialize` → 收 `result` → 发 `notifications/initialized`，参考 mcp-filesystem-client。
- **一个类型一个文件**：McpSession 与 McpToolSource 分离，便于测试与复用（AGENTS.md §Rust）。

---

## 6. 任务表（开发时勾选）

| 阶段 | 项 | 状态 |
|------|----|------|
| 1 | 1.1 添加 mcp feature | 已完成 |
| 1 | 1.2 构建验证 | 已完成 |
| 2 | 2.1 McpSession 结构 | 已完成 |
| 2 | 2.2 transport 启动与 on_message | 已完成 |
| 2 | 2.3 initialize 握手 | 已完成 |
| 2 | 2.4 send_request | 已完成 |
| 2 | 2.5 wait_for_result | 已完成 |
| 2 | 2.6 close | 已完成 |
| 2 | 2.7 McpSession 单元测试 | 已完成 |
| 3 | 3.1 McpToolSource 结构 | 已完成 |
| 3 | 3.2 Tool → ToolSpec 映射 | 已完成 |
| 3 | 3.3 list_tools 实现 | 已完成 |
| 3 | 3.4 CallToolResult → ToolCallContent 映射 | 已完成 |
| 3 | 3.5 call_tool 实现 | 已完成 |
| 3 | 3.6 异步包装 | 已完成 |
| 3 | 3.7 实现 ToolSource trait | 已完成 |
| 3 | 3.8 McpToolSource 集成测试 | 已完成 |
| 4 | 4.1 条件导出 | 已完成 |
| 4 | 4.2 lib 导出 | 已完成 |
| 4 | 4.3 注释与文档 | 已完成 |
| 5 | 5.1 react_mcp 示例 | 已完成 |
| 5 | 5.2 示例文档 | 已完成 |
| 5 | 5.3 文档同步 | 已完成 |
| 5 | 5.4 react_exa 示例 | 已完成 |
