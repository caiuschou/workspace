# Rust LangGraph 开发计划（敏捷版）

## 项目概述

**rust-langgraph** 是 LangGraph 框架的 **Rust 原生实现**，采用类型安全的状态机、trait 多态和 actor 模式。

### 设计原则

1. **类型安全** - 状态机枚举，编译时保证正确性
2. **零成本抽象** - Trait 泛型，不运行时反射
3. **显式优于隐式** - 不魔法字符串，模式匹配
4. **所有权明确** - Actor 模式，channel 通信

### 敏捷开发方式

- **Sprint**：固定周期（建议 1~2 周），每个 Sprint 结束交付一个 **最小可交付产品（MVP）**。
- **MVP**：可在本 Sprint 内演示、运行或集成的最小功能集合。
- **验收**：每个 Sprint 有明确的「可演示结果」和验收清单，通过即视为该 Sprint 完成。
- **回溯**：Sprint 内未完成项放入 Backlog，可在后续 Sprint 按优先级纳入。

### 包布局

在仓库**根目录**下使用 **`rust-langgraph/`** 作为「rust-langgraph 包」的根目录，所有实现放在其内：

- **`rust-langgraph/`**：与 `crates/`、`examples/`、`mcp-impls/` 平级，作为 LangGraph 相关代码的容器。
- **`rust-langgraph/crates/langgraph`**：核心库（trait、Agent、状态机、工具、记忆等），由根 workspace 的 `Cargo.toml` 通过 `members = ["rust-langgraph/crates/langgraph", ...]` 引入。
- **`rust-langgraph/crates/*`**：后续可按需增加 `langgraph-openai`、`langgraph-xxx` 等实现包，均在根 workspace 的 `members` 中追加路径即可。

根 workspace 构建：在仓库根目录执行 `cargo build -p langgraph`；各 Sprint 中涉及「新建 crate / 示例」的路径均指 `rust-langgraph/` 下的路径。

---

## Sprint 总览

| Sprint | MVP（最小可交付产品） | 可演示结果 | 建议周期 |
|--------|------------------------|------------|----------|
| S1 | 可运行的 Echo Agent | `cargo run --example echo` 输入即回显 | 1 周 |
| S2 | 真实 Chat 单轮对话 | 调 LLM 完成一问一答 | 1 周 |
| S3 | 流式 Chat + 会话记忆 | 流式输出 token，多轮对话带历史 | 1~2 周 |
| S4 | ReAct + 单工具 | 问「3+5」能调工具并返回 8 | 1~2 周 |
| S5 | 工具生态 + 记忆扩展 | 多工具协作或简单 RAG 示例 | 1~2 周 |
| S6 | 多 Agent 雏形 | 发 Task 给 Worker，收到结果 | 1~2 周 |
| S7 | 工作流 + 研究团队示例 | research-team 示例跑通 | 1~2 周 |
| S8 | HTTP API 可调 | curl 调通 /chat、/health、/metrics | 1 周 |
| S9 | 可部署 + 文档 | docker compose up 可访问，文档齐全 | 1~2 周 |

```
S1: Echo Agent        ████████████████████ [MVP: 跑起来的第一个 Agent]
S2: Chat 单轮          ████████████████████ [MVP: 一问一答]
S3: 流式+记忆          ████████████████████ [MVP: 流式 + 多轮]
S4: ReAct+工具         ████████████████████ [MVP: 思考→调工具→回答]
S5: 工具+记忆扩展      ░░░░░░░░░░░░░░░░░░░░ [MVP: 多工具 / RAG]
S6: 多 Agent 雏形     ░░░░░░░░░░░░░░░░░░░░ [MVP: Worker 收发包]
S7: 工作流+研究团队    ░░░░░░░░░░░░░░░░░░░░ [MVP: 研究团队示例]
S8: HTTP API          ░░░░░░░░░░░░░░░░░░░░ [MVP: 服务可调]
S9: 部署+文档          ░░░░░░░░░░░░░░░░░░░░ [MVP: Docker + 文档]
```

**原里程碑与 Sprint 对应**（便于从旧版 M1–M5 迁移）：

| 原里程碑 | 对应 Sprint | 说明 |
|----------|-------------|------|
| M1 核心 Trait | S1 + S2 + S3 | 拆成「Echo 最少 trait → Chat 类型 → 流式/记忆」三个可交付阶段 |
| M2 状态机与 Agent | S3 + S4 | 流式/记忆在 S3，状态机 + ReAct + LLM 在 S4 |
| M3 工具与记忆 | S4 + S5 | ReAct+单工具在 S4，多工具+记忆扩展在 S5 |
| M4 多 Agent | S6 + S7 | Actor/Worker 雏形在 S6，工作流+研究团队在 S7 |
| M5 生产级 | S8 + S9 | HTTP API 在 S8，配置/部署/文档在 S9 |

---

## Sprint 1：可运行的 Echo Agent

**MVP**：能跑起来的第一个 Agent，输入即回显。

**验收标准**：
- [x] `cargo run -p langgraph --example echo -- "你好"` 输出 `你好`
- [x] `cargo test -p langgraph` 通过
- [x] `cargo clippy -p langgraph` 无新增告警

### 1.1 项目骨架

- [x] 根 workspace 的 `members` 中已有 `rust-langgraph/crates/langgraph`（与现有 workspace 一致：`resolver = "3"`、`edition = "2024"` 等）
- [x] `rust-langgraph/crates/langgraph/Cargo.toml`、`src/lib.rs` 模块入口已就绪，按需补全目录
- [x] 在 `rust-langgraph/crates/langgraph/src/` 下建立占位：`traits.rs`、`message.rs`、`error.rs`、`agent/mod.rs`、`llm/mod.rs`、`tool/mod.rs`、`state.rs`、`memory/mod.rs`、`actor/mod.rs`
- [x] `rust-langgraph/.rustfmt.toml`、`rust-langgraph/clippy.toml`、`.github/workflows/rust-langgraph-ci.yml`（fmt + clippy + test）
- [x] `rust-langgraph/README.md` 已存在；`rust-langgraph/crates/langgraph/` 下可按需增加 `LICENSE-APACHE`、`LICENSE-MIT`、`CHANGELOG.md`

### 1.2 最小 Trait 与类型

- [x] **Agent** trait（本 Sprint 仅需：`name`、`run(Input) -> Result<Output, Error>`，`Input/Output/Error` 关联类型）
- [x] **Message**：本 Sprint 仅需 `UserMessage { content }` 或等价最小结构，用于 Echo 输入
- [x] **Error**：`AgentError` 最小枚举（如 `ExecutionFailed(String)`），`thiserror` 派生

### 1.3 Echo Agent 与示例

- [x] `EchoAgent` 实现 `Agent`，`Input = String`，`Output = String`，`run` 原样返回
- [x] `examples/echo.rs`：从 env/args 取一句话，调 `EchoAgent::run`，打印结果

### S1 交付物

- [x] 可编译、可测试的 `rust-langgraph/crates/langgraph`（即 `-p langgraph`）
- [x] 可运行的 `cargo run -p langgraph --example echo -- "你好"` 并输出 `你好`

---

## Sprint 2：真实 Chat 单轮对话

**MVP**：调 LLM 完成一问一答（单轮、无记忆）。

**验收标准**：
- [x] 运行示例能向 OpenAI（或 mock）发一问并拿到一答
- [x] 至少有 `examples/chat.rs` 或等价入口可演示

### 2.1 LlmClient 与请求/响应类型

- [x] `LlmClient` trait：`async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError>`
- [x] `ChatRequest`、`ChatResponse`、`Usage`（prompt_tokens, completion_tokens）
- [x] `LlmError` 枚举（ApiError、RateLimit、Auth、InvalidRequest、Network、Parsing、StreamClosed）
- [x] 单元测试：序列化/反序列化

### 2.2 OpenAI 实现（或 Mock）

- [x] `OpenAiConfig`（api_key, base_url, model, default_temperature）
- [x] `OpenAiClient` 实现 `LlmClient`，`chat()` 发 HTTP 请求并解析（可选 feature `openai`）；`MockLlmClient` 返回固定句或回显
- [ ] 若用真实 API：`RetryConfig` + `is_retryable(LlmError)` + 简单重试（放入 Backlog）

### 2.3 ChatAgent 单轮

- [x] `ChatAgent<C: LlmClient>`：`llm: C`、`system_prompt: Option<String>`
- [x] `ChatInput`、`ChatOutput`（本 Sprint 为 `String`）
- [x] 实现 `AsyncAgent`，单轮：把用户输入转为 messages，调 `llm.chat()`，返回内容
- [x] `examples/chat.rs`：从 args 读一问，调 ChatAgent，打印回答

### S2 交付物

- [x] 可运行的「一问一答」Chat 示例（Mock 默认；真实 API 需 `--features openai` 与 `OPENAI_API_KEY`）

---

## Sprint 3：流式 Chat + 会话记忆

**MVP**：流式输出 token，多轮对话带历史。

**验收标准**：
- [x] 有接口能流式返回 token（如按 chunk 打印或 SSE）
- [x] 多轮对话时，后续轮能看到历史消息（通过 SessionMemory）

### 3.1 流式接口

- [x] `StreamAgent` trait：`run_stream(Input) -> Pin<Box<dyn Stream<Item = Result<StreamItem, Error>> + Send>>`
- [x] `ChatStreamEvent`：`Token(String)`、`Done(String)`、`Error(LlmError)`
- [x] `LlmStreamClient::chat_stream(req)`：Mock 已实现；`LlmClient` 扩展为可选流式
- [x] OpenAI SSE 解析：`SseStream`、`[DONE]` 处理（OpenAiClient 已实现 `LlmStreamClient`，流式请求 + SSE 解析 + `data: [DONE]` 结束）

### 3.2 会话记忆

- [x] `Memory` trait：`add(Message)`、`get(limit)`、`clear()`、`count()`
- [x] `Message` 结构体 + `MessageRole` 枚举：`User`、`Assistant`、`System`、`Tool`（含 content/timestamp 等最小字段）
- [x] `SessionMemory`：`Arc<RwLock<Vec<Message>>>`、FIFO 容量限制、实现 `Memory`
- [x] `ToolCall`、`ToolResult`；`Message::user/assistant/system/tool` 构造

### 3.3 ChatAgent 接记忆与流式

- [x] `ChatAgent` 增加 `memory: Option<Arc<dyn Memory>>`、`with_memory()`
- [x] 实现 `StreamAgent`，内部调用 `llm.chat_stream()`，映射为 `ChatStreamEvent`
- [x] 单轮/多轮均把用户与助手消息写入 Memory，下次请求带 `get(limit)` 作为上下文
- [x] `examples/chat_stream.rs`：演示流式 + 多轮（`--multi "第一句" "第二句"`）

### S3 交付物

- [x] 流式 Chat 可演示（`cargo run -p langgraph --example chat_stream -- "你好"`）
- [x] 多轮对话带 SessionMemory 可演示（`cargo run -p langgraph --example chat_stream -- --multi "第一句" "第二句"`）

---

## Sprint 4：ReAct + 单工具

**MVP**：问「3+5 等于几」能调工具并返回 8。

**验收标准**：
- [x] 有 ReAct 示例，输入算术式能调用计算工具并输出结果
- [x] 状态在「思考 → 动作 → 观察 → 结束」间可区分

### 4.1 状态机与执行器

- [x] `StateMachine` trait：`State`、`Event`、`Output`，`transition(state, event) -> Result<StateTransition<State, Output>, StateError>`
- [x] `StateTransition<S,O>`：`Continue(S)`、`Output(O,S)`、`Done(O)`
- [x] `Runner<S,E,O>`：持有机器和当前状态，`run(events)` 迭代并步数限制
- [x] `StateError` 与相关错误类型

### 4.2 Tool 与注册（最小）

- [x] `Tool` trait：`name()`、`description()`、`parameters_schema()`、`execute(Value) -> Result<Value, ToolError>`
- [x] `ToolRegistry`：`register(Box<dyn Tool>)`、`get(name)`、`execute(name, args)`
- [x] `ToolError`、参数校验最少支持（如必填字段）

### 4.3 一个内置工具：Calculator

- [x] `CalculatorTool`，参数 schema 如 `{ "expression": string }`
- [x] 使用 `evalexpr` 或简单安全求值，实现 `Tool`
- [x] 单元测试（含非法表达式）

### 4.4 ReAct Agent

- [x] `ReActState`：`Thinking { query, iterations }`、`Acting { tool_calls }`、`Observing { results }`、`Done { answer }`
- [x] `ThinkNode`：`build_prompt()`、`parse_thought()`、`extract_tool_calls()`
- [x] `ActNode`：`execute(tool_calls)`，调用 `ToolRegistry`
- [x] `ObserveNode`：`process(results)`、`should_continue()`
- [x] `DEFAULT_REACT_PROMPT`、`build_prompt(query, tools, history)`、`format_tool_description()`
- [x] 把 Think/Act/Observe 接到 `StateMachine`/`Runner`，实现 ReAct 循环
- [x] `examples/react.rs`：问「3+5」等，输出工具结果与最终答案

### S4 交付物

- [x] ReAct 示例可调 Calculator 并返回正确算术结果

---

## Sprint 5：工具生态 + 记忆扩展

**MVP**：多工具协作或简单 RAG 示例可运行。

**验收标准**：
- [x] 至少再有两种内置工具（如 HttpRequest、FileOps）可用
- [x] 有 SessionMemory 之外的记忆（Profile 或 Vector 至少一种），并有简单示例（如 memory 或 rag）

### 5.1 内置工具扩充

- [x] `HttpRequestTool`：url、method、headers、body，实现 `Tool`，单元测试用 mock
- [x] `FileOpsTool`：read/write/list/exists，路径安全检查，实现 `Tool`

### 5.2 工具组合与校验

- [x] `ToolChain`：输出串到下一工具，实现 `Tool`
- [x] `validate_args(schema, args)`、`ValidationError`，在 `ToolRegistry::execute` 中使用
- [ ] （可选）`ParallelTools` 或 `ToolMap` 本 Sprint 做最小实现即可

### 5.3 记忆扩展

- [x] `SemanticMemory` trait：`add(content, embedding)`、`search(query_embedding, top_k) -> Vec<MemoryResult>`
- [x] `VectorMemory`：`MemoryEmbedding`（id, content, vector, metadata）、余弦相似度 `search`
- [x] `Embedder` trait：`embed(text)`、`embed_batch(texts)`；`MockEmbedder` 已实现，`OpenAiEmbedder` 可后续加 feature
- [ ] `ProfileMemory`（可选本 Sprint）：`add_profile`、`get_profile`、简单持久化或内存版

### 5.4 Memory Agent 或带记忆的 ReAct

- [ ] `MemoryAgent<C, E>` 或「ReAct + VectorMemory」：检索相关记忆注入 prompt，再回答（放入 Backlog）
- [x] `examples/tools.rs`：多工具串联或并行
- [x] `examples/memory.rs` 或 `examples/rag.rs`：Session + Vector/Profile 最小示例

### S5 交付物

- [x] 多工具示例可运行
- [x] 至少一个「记忆增强」示例（memory 或 rag）可运行

---

## Sprint 6：多 Agent 雏形

**MVP**：发一个 Task 给 Worker，能收到处理结果。

**验收标准**：
- [ ] 有 API 可「发任务 → 指定或默认 Worker 处理 → 返回结果」
- [ ] 至少一个 Worker 实现可用（如 EchoWorker 或最小 ResearcherWorker）

### 6.1 Actor 基础

- [ ] `ActorId`、`AgentMessage`（Task、Stop、Ping）、`Task` 结构体
- [ ] `Handler<S>` trait：`handle(msg, state) -> Result<(), ActorError>`
- [ ] `ActorAgent<S>`：id、inbox、state、handler，`run()` 消息循环
- [ ] `AgentChannel`、`ActorRef<S>`：`send`、`send_timeout`、`try_send`、`request()`（请求-响应）

### 6.2 监督与路由（最小）

- [ ] `SupervisionStrategy`：OneForOne / OneForAll / AllForOne，重启逻辑最小实现
- [ ] `Router` trait：`route(task, workers) -> Option<usize>`
- [ ] `RoundRobinRouter` 或 `LeastBusyRouter` 至少一种

### 6.3 Worker 与 Supervisor

- [ ] `Worker` trait：`name`、`description`、`async fn handle(Task) -> TaskResult`
- [ ] `TaskResult` 结构体
- [ ] 一个具体 Worker（如 `EchoWorker` 或 `ResearcherWorker` 雏形）实现 `Worker`
- [ ] `Supervisor`：持有一组 `ActorRef<WorkerState>` 和 `Router`，能接收 Task 并派发到 Worker，返回 TaskResult
- [ ] `WorkerActor<S>`：把 `Worker` 包装成可被 `ActorAgent` 驱动的 Handler（本 Sprint 做最小适配即可）

### 6.4 示例

- [ ] `examples/multi-agent.rs` 或等价：创建 Supervisor + 一个 Worker，发一个 Task，打印 TaskResult

### S6 交付物

- [ ] 「发 Task → Worker 处理 → 得结果」可演示

---

## Sprint 7：工作流 + 研究团队示例

**MVP**：research-team 示例跑通，多 Worker 按流程协作。

**验收标准**：
- [ ] `examples/research-team.rs`（或等价）能跑通：例如「研究 → 分析 → 写作」等步骤
- [ ] 工作流支持顺序、并行、分支中的至少两种

### 7.1 工作流引擎

- [ ] `Step` 枚举：`Execute { agent, input }`、`Parallel { steps }`、`Sequence { steps }`、`Branch { condition, then, else }`、`Loop { condition, body, max_iterations }`
- [ ] `Condition` 类型
- [ ] `WorkflowExecutor`：`execute(step)`，实现串行、并发、分支、循环
- [ ] `WorkflowBuilder`：`execute`/`parallel`/`sequence`/`branch`/`loop`、`build()`

### 7.2 研究团队 Worker

- [ ] `ResearcherWorker`：搜索/聚合/验证逻辑，实现 `Worker`
- [ ] `AnalystWorker`：分析/比较/洞察，实现 `Worker`
- [ ] `WriterWorker`：草稿/修订/格式，`WritingStyle` 枚举，实现 `Worker`
- [ ] 将三者接入 Supervisor + Workflow，组成 research-team 流程

### 7.3 路由与负载（若 S6 未做完）

- [ ] `LeastBusyRouter` / `SemanticRouter`（可先做简单版）
- [ ] `WorkerMetrics`、`get_least_busy()` 等按需实现

### 7.4 示例与文档

- [ ] `examples/research-team.rs` 可运行，README 说明如何运行与预期输出

### S7 交付物

- [ ] research-team 示例跑通，工作流可见（顺序/并行/分支等）

---

## Sprint 8：HTTP API 可调

**MVP**：通过 HTTP 调用 chat、健康检查与指标。

**验收标准**：
- [ ] `curl` 能调通 `POST /api/v1/chat`、`GET /health`、`GET /metrics`
- [ ] 流式 Chat 可用 SSE（如 `POST /api/v1/chat/stream`）

### 8.1 路由与状态

- [ ] `ApiState` 持有要暴露的 Agent / LlmClient 等
- [ ] `create_router()`：挂载 `/health`、`/metrics`、`/api/v1/chat`、`/api/v1/chat/stream`
- [ ] `GET /health` 返回 200 与简单 JSON
- [ ] `GET /metrics` 返回 Prometheus 文本格式（可先返回占位或少量指标）

### 8.2 Chat 端点

- [ ] `POST /api/v1/chat`：body 为 ChatRequest，返回 ChatResponse，内部调 ChatAgent
- [ ] `POST /api/v1/chat/stream`：SSE 流式返回 token
- [ ] 请求/响应结构体与现有 `ChatRequest`/`ChatResponse` 对齐

### 8.3 中间件与可观测性最小

- [ ] CORS、请求 ID、请求日志中间件
- [ ] （可选）速率限制（如 tower_governor）可放在 S9
- [ ] 指标：`agent_requests_total`、`agent_duration_seconds`、`llm_tokens_total` 等至少声明并可在 `/metrics` 中暴露

### 8.4 其它 Agent 端点（可选）

- [ ] `POST /api/v1/react`、`POST /api/v1/workflow`、`GET /api/v1/agents` 可做最小占位，在 S9 完善

### S8 交付物

- [ ] 服务启动后，`curl` 能调通 `/health`、`/metrics`、`/api/v1/chat`（及流式）

---

## Sprint 9：可部署 + 文档

**MVP**：Docker 跑起服务，文档与示例齐全，便于他人使用。

**验收标准**：
- [ ] `docker compose up` 后可访问 HTTP 服务（含 /health）
- [ ] 有「从零到跑起来」的 getting-started 文档
- [ ] 所有示例带 README 或顶层说明，且可运行

### 9.1 配置与多环境

- [ ] `AppConfig`：server、llm、logging、telemetry
- [ ] `ServerConfig`、`LlmConfig`、`LoggingConfig`、`TelemetryConfig`
- [ ] 环境变量加载（如 `LANGGRAPH_*`）、TOML 配置文件
- [ ] Profile：Dev / Test / Prod

### 9.2 可观测性

- [ ] Prometheus 指标完善并挂到 `/metrics`
- [ ] OpenTelemetry + Jaeger 或 stdout，Span 层级（agent_execution、llm_call、tool_execution）
- [ ] 结构化日志（JSON 生产、pretty 开发），请求 ID 注入

### 9.3 部署

- [ ] Dockerfile：多阶段、非 root、健康检查
- [ ] docker-compose：langgraph 服务，可选 PostgreSQL/Redis，网络与依赖说明

### 9.4 健康与就绪

- [ ] `HealthStatus`、`check_liveness()`、`check_readiness()`，依赖检查（若有时）
- [ ] `/health` 可返回详细状态（可选）

### 9.5 文档与示例

- [ ] `docs/getting-started.md`：环境、安装、快速开始
- [ ] `docs/agents.md`、`docs/tools.md`、`docs/memory.md`、`docs/multi-agent.md`、`docs/deployment.md` 覆盖主要能力
- [ ] OpenAPI/Swagger（如 utoipa）：`/api/docs`、`/api/openapi.json`
- [ ] 各示例 README：如何跑、预期输出、必要环境变量

### S9 交付物

- [ ] `docker compose up` 可访问服务
- [ ] getting-started 与各模块文档可用，示例均可按文档运行

---

## Backlog（按需纳入后续 Sprint）

以下在对应 Sprint 未做完时可写入 Backlog，按优先级在后续 Sprint 中实现：

- ~~**OpenAI SSE 流式**~~：已完成。`OpenAiClient` 已实现 `LlmStreamClient`，解析 SSE、`data: [DONE]` 结束。
- **类型状态机**：`Init`/`Running`/`Done` 标记、`TypeStateMachine<S>`、编译时状态约束
- **Checkpoint**：`Checkpoint` trait、`MemoryCheckpoint`、`FileCheckpoint`，与状态机/Agent 集成
- **PromptTemplate**：`{{var}}`、`{{#if}}...{{/if}}`，与 ChatAgent/ReAct 集成
- **ToolMap / ParallelTools**：输入输出映射、并发执行聚合
- **SemanticRouter**：基于 LLM 的 Worker 路由
- **速率限制**：tower_governor 等在 API 层
- **完整 OpenAPI**：所有端点带 `utoipa::path` 与示例请求/响应

---

## Crate 结构（目标形态）

以下为全量完成后的目标结构，各 Sprint 只创建与本 Sprint MVP 相关的目录与文件。所有路径均相对于 **`rust-langgraph/`** 根目录。

```
rust-langgraph/
├── README.md
├── crates/
│   ├── langgraph/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── traits.rs
│   │   │   ├── message.rs
│   │   │   ├── error.rs
│   │   │   ├── state.rs
│   │   │   ├── agent/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── chat.rs
│   │   │   │   ├── react.rs
│   │   │   │   └── memory.rs
│   │   │   ├── tool/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── registry.rs
│   │   │   │   ├── builtin.rs
│   │   │   │   └── compose.rs
│   │   │   ├── memory/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── session.rs
│   │   │   │   ├── profile.rs
│   │   │   │   └── vector.rs
│   │   │   ├── actor/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── channel.rs
│   │   │   │   └── supervise.rs
│   │   │   ├── supervisor.rs
│   │   │   ├── worker/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── researcher.rs
│   │   │   │   ├── analyst.rs
│   │   │   │   └── writer.rs
│   │   │   ├── workflow.rs
│   │   │   ├── llm/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── openai.rs
│   │   │   │   ├── stream.rs
│   │   │   │   ├── retry.rs
│   │   │   │   └── embedder.rs
│   │   │   ├── config.rs
│   │   │   ├── telemetry.rs
│   │   │   └── api/
│   │   │       ├── mod.rs
│   │   │       ├── routes.rs
│   │   │       ├── handlers.rs
│   │   │       ├── sse.rs
│   │   │       └── middleware.rs
│   │   ├── examples/
│   │   │   ├── echo.rs      (S1)
│   │   │   ├── chat.rs      (S2)
│   │   │   ├── react.rs     (S4)
│   │   │   ├── tools.rs     (S5)
│   │   │   ├── memory.rs    (S5)
│   │   │   ├── rag.rs       (S5)
│   │   │   ├── multi-agent.rs (S6)
│   │   │   └── research-team.rs (S7)
│   │   ├── tests/
│   │   └── benches/
│   └── (可选) langgraph-openai/ 等实现包
```

---

## 对比：Python vs Rust

| 方面 | Python LangGraph | Rust LangGraph |
|------|------------------|----------------|
| 状态 | `dict[str, Any]` | `enum StateMachine` |
| 节点 | `def func(state)` | `trait Node` |
| 路由 | 字符串匹配 | 模式匹配 |
| 工具 | `Callable` | `trait Tool` |
| 并发 | 线程池 + 锁 | Actor + Channel |
| 错误 | 异常 | `Result<T, E>` |

---

## 下一步

1. **Sprint 4 已完成**：`StateMachine`/`StateTransition`/`Runner`/`StateError`、`Tool`/`ToolRegistry`/`ToolError`、`CalculatorTool`（evalexpr）在 langgraph；ReAct 已拆为独立包 `langgraph-react`（`ReActState`/`ReActAgent`、`build_prompt`/`parse_thought`/`format_tool_description`）。ReAct 示例：`cargo run -p langgraph-react --example react -- "3+5等于几"` 输出 `8`；使用 `SequenceMockLlmClient` 按序返回 Action 与 Answer。
2. **Sprint 5 已完成**：`HttpRequestTool`/`FileOpsTool`、`ToolChain`、`validate_args`/`ValidationError`；`SemanticMemory`/`VectorMemory`/`Embedder`/`MockEmbedder`；`examples/tools.rs`、`examples/memory.rs` 可运行。多工具示例：`cargo run -p langgraph --example tools`；记忆示例：`cargo run -p langgraph --example memory`。
3. **Sprint 6 启动**：多 Agent 雏形（Actor、Worker、Supervisor、发 Task 收结果）。
4. **每个 Sprint 结束**：对照「验收标准」做一次演示或脚本检查，未完成项记入 Backlog。
5. **后续新包**：新增实现包时，在 `rust-langgraph/crates/` 下建目录，并在根 `Cargo.toml` 的 `members` 中追加路径，如 `"rust-langgraph/crates/langgraph-openai"`。

---

*最后更新: 2026-01-28*
