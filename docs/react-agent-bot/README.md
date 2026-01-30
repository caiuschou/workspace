# ReAct Agent 对话机器人（新 Crate）方案

全新 crate：**ReAct Agent**，作为对话机器人，具备**持久化记忆**与 **MCP 能力**。本文档用于讨论方案，确定后再按任务表实现。

---

## 1. 目标简述

| 能力       | 说明 |
|------------|------|
| **ReAct Agent** | 推理(Think) + 行动(Act) + 观察(Observe) 循环，支持多轮工具调用 |
| **对话机器人**   | 多轮对话，按会话(thread)区分，用户发消息、Agent 回复 |
| **持久化记忆**   | 会话历史与图状态可落盘，重启后按 thread_id 恢复 |
| **MCP 能力**    | 工具来自 MCP Server（如 list_tools / call_tool），可配置 stdio/HTTP 等 |
| **Exa Web Search** | 通过 Exa MCP 提供网页搜索（如 `web_search_exa`、`get_code_context_exa`、`company_research_exa`），可选启用 |
| **Fetch MCP（抓内容）** | 通过官方 Fetch MCP Server 提供 `fetch` 工具：按 URL 抓取网页并转 Markdown，可选启用 |

---

## 2. 定位与依赖（待你拍板）

### 2.1 新 Crate 的定位

- **选项 A：应用层 Crate（推荐）**  
  - 新 crate 依赖现有 **rust-langgraph**（`langgraph`），在其上做「组装」与「配置」。  
  - 职责：提供开箱即用的 ReAct 对话机器人（库 API + 可选 CLI/HTTP）。  
  - 优点：复用已有 ReAct 节点、Checkpointer、ToolSource、McpToolSource，落地快。  
  - 缺点：与 `langgraph` 绑定，版本随 workspace 升级。

- **选项 B：完全独立 Crate**  
  - 不依赖 `langgraph`，自实现 ReAct 循环、状态、记忆、MCP 客户端。  
  - 优点：可独立发布、不跟 rust-langgraph 演进强绑。  
  - 缺点：工作量大，且与现有 `docs/rust-langgraph`、`mcp-integration` 设计重复。

**建议**：采用 **选项 A**，新 crate 作为「基于 langgraph 的对话机器人应用层」，名称例如 `react-agent-bot` 或 `conversation-agent`。

### 2.2 依赖关系（若选 A）

```
react-agent-bot (新 crate)
├── langgraph (path: rust-langgraph/crates/langgraph)  // ReAct 图、记忆、ToolSource
├── mcp_client / mcp_core (可选，若需在 bot 内直接建 MCP 连接时)
└── 其他：tokio, serde, thiserror, tracing 等
```

- **MCP**：优先通过 `langgraph` 的 `McpToolSource`（已对接 workspace 的 mcp-client）使用 MCP；若需 HTTP/SSE 等其它 transport，可在本 crate 或 langgraph 中扩展。

---

## 3. 架构草图

```
                    ┌─────────────────────────────────────────┐
                    │         react-agent-bot (本 crate)       │
                    │                                         │
  User Message ────▶│  Agent::chat(thread_id, message)        │
                    │       │                                  │
                    │       ▼                                  │
                    │  CompiledStateGraph::invoke(state, config)│
                    │       │  config.thread_id = thread_id    │
                    │       ▼                                  │
                    │  Think ──▶ Act ──▶ Observe ──▶ (loop/END)│
                    │       │      │         │                 │
                    │       ▼      ▼         ▼                 │
                    │  LLM    ToolSource  写回 messages         │
                    │         (MCP)                            │
                    └─────────────┬───────────────────────────┘
                                  │
                    ┌─────────────▼───────────────────────────┐
                    │  langgraph                              │
                    │  - ReActState, ThinkNode, ActNode,      │
                    │    ObserveNode, StateGraph              │
                    │  - Checkpointer (MemorySaver /          │
                    │    SqliteSaver) → 持久化会话             │
                    │  - ToolSource / McpToolSource           │
                    └─────────────────────────────────────────┘
```

- **持久化记忆**：通过 `config.thread_id` + `Checkpointer`（如 SqliteSaver）实现；每次 `chat` 传入同一 `thread_id` 即同一会话，历史与状态由 checkpoint 恢复/写入。
- **MCP**：`ActNode` 使用 `ToolSource`；本 crate 负责构造 `McpToolSource`（例如 stdio 子进程或后续 HTTP）并注入图。

---

## 4. 持久化记忆

- **短期记忆（会话）**：使用 langgraph 的 **Checkpointer**。  
  - 默认推荐 **SqliteSaver**：数据目录可配置（如 `~/.react-agent-bot/` 或环境变量），进程重启后按 `thread_id` 恢复。  
  - 开发/测试可用 **MemorySaver**（仅进程内）。
- **长期记忆（可选）**：若需要跨会话、用户级记忆，可后续接 langgraph 的 **Store**（如 SqliteStore），namespace 用 `user_id` 等；首版可只做会话级持久化。

---

## 5. MCP 能力

- **工具发现与执行**：由 langgraph 的 **ToolSource** 抽象完成；本 crate 使用 **McpToolSource**（stdio 子进程）或后续扩展的 HTTP/SSE MCP 客户端。
- **配置方式（建议）**：  
  - 配置 MCP Server 的启动方式：例如 `command + args`（stdio），或 URL（HTTP）。  
  - 支持「无 MCP」：ToolSource 用 Mock 或空实现，仅 LLM 对话。
- **Resources**：若后续需要 MCP Resources，可接 langgraph 的 ResourceSource + Context 节点；首版可只做 Tools。

### 5.1 Exa Web Search

- **能力**：通过 **Exa 官方 MCP**（`https://mcp.exa.ai/mcp`）提供网页搜索类工具，供 ReAct 在 Think 时选择调用，例如：
  - `web_search_exa`：通用网页搜索
  - `get_code_context_exa`：代码/文档上下文检索
  - `company_research_exa`：公司/产品调研
- **接入方式**：Exa MCP 为 HTTP 端点，需用 **stdio→HTTP 桥**（如 `npx -y mcp-remote <url>`）让 `McpToolSource` 通过子进程与之通信；实现可参考 `rust-langgraph/crates/langgraph/examples/react_exa.rs`。
- **配置（建议）**：  
  | 配置/环境变量 | 说明 | 默认 |
  |---------------|------|------|
  | `exa.enabled` / `EXA_WEB_SEARCH` | 是否启用 Exa 工具源 | `false` |
  | `EXA_API_KEY` | Exa API Key（可选，未设置时使用托管端点可能限流） | - |
  | `MCP_EXA_URL` | Exa MCP 端点 | `https://mcp.exa.ai/mcp` |
  | `MCP_REMOTE_CMD` | 桥接命令 | `npx` |
  | `MCP_REMOTE_ARGS` | 桥接参数（需包含 URL 或通过逻辑追加） | `-y mcp-remote` |
- **AgentConfig**：支持预设「Exa 工具源」：当启用 Exa 时，用上述环境变量或配置项构造 `McpToolSource::new_with_env(cmd, args, [("EXA_API_KEY", key), ...])`，作为图的唯一 ToolSource 或与其他 MCP 工具合并（合并方式可后续设计，首版可仅「Exa 或 其他 MCP 二选一」）。

### 5.2 Fetch MCP Server（官方，偏「抓内容」）

- **能力**：**Fetch MCP Server** 为 MCP 官方维护的 Server（[modelcontextprotocol/fetch](https://github.com/modelcontextprotocol/servers/tree/main/src/fetch)），偏「抓取网页内容」：提供 `fetch` 工具，根据 URL 拉取网页并转为 Markdown，供 ReAct 在 Think 时调用。
- **工具**：
  - `fetch`：抓取指定 URL 内容并转为 Markdown  
    - `url`（必填）：要抓取的 URL  
    - `max_length`（可选）：返回最大字符数，默认 5000  
    - `start_index`（可选）：从某字符索引开始截取，便于分块读取长页  
    - `raw`（可选）：是否返回原始内容（不转 Markdown）
- **用途**：与 Exa（搜索+摘要）互补——Exa 偏「搜索与检索」，Fetch 偏「按 URL 抓内容」；可同时启用（多 ToolSource 合并）或二选一。
- **接入方式**：Fetch 为 **stdio** 启动的 MCP Server（如 `uvx fetch-mcp`、`npx @modelcontextprotocol/server-fetch` 等），本 crate 用 `McpToolSource::new(command, args)` 直接 spawn 子进程，无需 HTTP 桥。
- **配置（建议）**：  
  | 配置/环境变量 | 说明 | 默认 |
  |---------------|------|------|
  | `fetch.enabled` / `FETCH_MCP_ENABLED` | 是否启用 Fetch 工具源 | `false` |
  | `FETCH_MCP_CMD` | 启动命令（如 `uvx`、`npx`） | `uvx` |
  | `FETCH_MCP_ARGS` | 启动参数（如 `fetch-mcp`、`-y @modelcontextprotocol/server-fetch`） | `fetch-mcp` |
- **AgentConfig**：支持预设「Fetch 工具源」；启用时用上述 command+args 构造 `McpToolSource`；与 Exa 可并存（多 ToolSource 合并时需在 langgraph 或本 crate 内做聚合）。

---

## 6. 对外接口（建议）

- **库**  
  - `Agent` 或 `ReactAgentBot`：  
    - `new(config: AgentConfig) -> Self`  
    - `chat(&self, thread_id: impl Into<String>, user_message: impl Into<String>) -> Result<AssistantMessage, AgentError>`  
  - `AgentConfig`：LLM 配置、Checkpointer（SqliteSaver 路径 / MemorySaver）、ToolSource（MCP 命令或 None）、**Exa Web Search / Fetch MCP（抓内容）** 开关与可选配置、可选 Store 等。
- **可选 CLI**：读 stdin 或单条命令 `chat "用户消息"`，打印助手回复；可带 `--thread-id`。
- **可选 HTTP**：例如 POST `/chat`，body `{ "thread_id", "message" }`，返回助手回复；可后续迭代。

首版建议：**只做库 + 一个最小 CLI 示例**，HTTP 留作后续。

---

## 7. Crate 与 Workspace 布局

- **Crate 名**：`react-agent-bot` 或 `conversation-agent`（你定一个即可）。
- **位置**：  
  - 若希望与 langgraph 并列：`rust-langgraph/crates/react-agent-bot` 或 workspace 根下 `crates/react-agent-bot`。  
  - 推荐放在 **workspace 根下** `crates/react-agent-bot`，便于与 `mcp-client`、`mcp-server` 等平级，依赖 `langgraph` 用 path 指向 `../rust-langgraph/crates/langgraph`。
- **Workspace**：在根 `Cargo.toml` 的 `members` 中加入 `crates/react-agent-bot`。

---

## 8. 任务表（规划用，待方案确定后细化）

| 序号 | 任务 | 状态 | 说明 |
|------|------|------|------|
| 1 | 确定定位与依赖（A 还是 B） | 待定 | 见 §2 |
| 2 | 确定 Crate 名与目录位置 | 待定 | 见 §7 |
| 3 | 创建 Crate 骨架与依赖 | 未开始 | Cargo.toml、lib.rs、errors |
| 4 | AgentConfig 与 Agent 类型 | 未开始 | 配置 LLM、Checkpointer、ToolSource |
| 5 | 组装 ReAct 图（Think/Act/Observe） | 未开始 | 使用 langgraph，带条件边多轮 |
| 6 | 集成 Checkpointer（SqliteSaver + 可选 MemorySaver） | 未开始 | 持久化会话 |
| 7 | 集成 McpToolSource（stdio） | 未开始 | 配置 command+args 或禁用 |
| 7.1 | **Exa Web Search 集成** | 未开始 | 可选启用；mcp-remote 桥接 Exa MCP；EXA_API_KEY / MCP_EXA_URL 等配置；参考 langgraph `react_exa` |
| 7.2 | **Fetch MCP Server 集成** | 未开始 | 可选启用；官方 Fetch MCP（抓内容），stdio spawn；FETCH_MCP_CMD / FETCH_MCP_ARGS；可与 Exa 并存（多 ToolSource 聚合） |
| 8 | 实现 `chat(thread_id, message)` | 未开始 | invoke 图 + config.thread_id |
| 9 | 单元/集成测试 | 未开始 | 含 Mock LLM、Mock ToolSource |
| 10 | 最小 CLI 示例 | 未开始 | 可选 |
| 11 | 文档与 README | 未开始 | 使用方式、配置说明 |

---

## 9. 需要你拍板的点

1. **定位**：选 **A（依赖 langgraph）** 还是 **B（完全独立）**？  
2. **Crate 名**：`react-agent-bot` / `conversation-agent` 或其它？  
3. **目录**：`crates/react-agent-bot` 还是 `rust-langgraph/crates/react-agent-bot`？  
4. **首版范围**：是否只做「库 + 持久化 + MCP(stdio)」，CLI/HTTP 是否放到下一阶段？  
5. **长期记忆**：首版是否要 Store（跨会话记忆），还是只做会话级 Checkpointer？

你确认以上几点后，我可以按你的选择把任务表细化成可执行的开发步骤（并写入同一目录下的 `02-tasks.md` 或直接更新本文档任务表），再开始实现。

---

## 10. 更多扩展想法

更多可选能力与调研来源见 **[ideas.md](ideas.md)**，包括：ReAct 鲁棒性（max_iterations、解析重试）、MCP 扩展（Brave Search、Memory、Filesystem、Git、Time）、记忆与 RAG 分工、流式输出与 UX、Guardrails 与人在回路、成本与性能等。
