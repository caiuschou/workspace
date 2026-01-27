# Rust LangGraph 开发计划

## 项目概述

**rust-langgraph** 是 LangGraph 框架的 **Rust 原生实现**，采用类型安全的状态机、trait 多态和 actor 模式。

### 设计原则

1. **类型安全** - 状态机枚举，编译时保证正确性
2. **零成本抽象** - Trait 泛型，不运行时反射
3. **显式优于隐式** - 不魔法字符串，模式匹配
4. **所有权明确** - Actor 模式，channel 通信

---

## 里程碑

```
M0: 项目启动           ████████████████████ 2025-01-27
M1: 核心 Trait         ░░░░░░░░░░░░░░░░░░░░   [计划中]
M2: 状态机与 Agent     ░░░░░░░░░░░░░░░░░░░░   [计划中]
M3: 工具与记忆系统     ░░░░░░░░░░░░░░░░░░░░   [计划中]
M4: 多 Agent 协作      ░░░░░░░░░░░░░░░░░░░░   [计划中]
M5: 生产级实现         ░░░░░░░░░░░░░░░░░░░░   [计划中]
```

---

## M1: 核心 Trait (Week 1-2, Day 1-14)

### 目标
定义核心抽象，建立类型安全的基础架构。

---

### 1.1 项目结构 (Day 1-2)

#### 1.1.1 Workspace 配置
- [ ] 根 `Cargo.toml` 添加 workspace
  ```toml
  [workspace]
  members = ["crates/langgraph"]
  resolver = "2"
  [workspace.dependencies]
  tokio = { version = "1.35", features = ["full"] }
  serde = { version = "1.0", features = ["derive"] }
  async-trait = "0.1"
  thiserror = "1.0"
  ```
- [ ] 配置 `crates/langgraph/Cargo.toml`
- [ ] 设置 `[package.metadata.docs.rs]`

#### 1.1.2 目录结构
- [ ] `src/lib.rs` 模块入口
- [ ] `src/traits.rs` 核心 trait
- [ ] `src/message.rs` 消息类型
- [ ] `src/error.rs` 错误类型
- [ ] `src/state.rs` 状态机
- [ ] `src/agent/mod.rs` Agent 模块
- [ ] `src/llm/mod.rs` LLM 模块
- [ ] `src/tool/mod.rs` 工具模块
- [ ] `src/actor/mod.rs` Actor 模块
- [ ] `src/memory/mod.rs` 记忆模块
- [ ] `examples/` 目录
- [ ] `tests/` 目录

#### 1.1.3 开发工具
- [ ] `.rustfmt.toml` 配置
- [ ] `clippy.toml` 配置
- [ ] `.github/workflows/ci.yml` (fmt + clippy + test + doc)

#### 1.1.4 基础文件
- [ ] `README.md`
- [ ] `LICENSE-APACHE` + `LICENSE-MIT`
- [ ] `CHANGELOG.md`

---

### 1.2 核心 Trait (`src/traits.rs`) (Day 3-6)

#### 1.2.1 Agent Trait
- [ ] 定义 `Agent` trait
  ```rust
  pub trait Agent: Send + Sync {
      type Input: Send + Sync;
      type Output: Send + Sync;
      type Error: std::error::Error + Send + Sync;
      fn name(&self) -> &str;
      async fn run(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
  }
  ```
- [ ] doc 注释 + 示例
- [ ] 单元测试：trait 约束

#### 1.2.2 StreamAgent Trait
- [ ] 定义 `StreamAgent` trait
  ```rust
  #[async_trait]
  pub trait StreamAgent: Agent {
      type StreamItem: Send;
      fn run_stream(&self, input: Self::Input)
          -> Pin<Box<dyn Stream<Item = Result<Self::StreamItem, Self::Error>> + Send>>;
  }
  ```
- [ ] doc 注释 + 示例

#### 1.2.3 LlmClient Trait
- [ ] 定义 `LlmClient` trait
  ```rust
  #[async_trait]
  pub trait LlmClient: Send + Sync {
      async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError>;
      async fn chat_stream(&self, req: ChatRequest)
          -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError>;
  }
  ```
- [ ] 定义 `ChatRequest` 结构体
- [ ] 定义 `ChatResponse` 结构体
- [ ] 定义 `Usage` 结构体 (prompt_tokens, completion_tokens)
- [ ] 单元测试：序列化

#### 1.2.4 Tool Trait
- [ ] 定义 `Tool` trait
  ```rust
  #[async_trait]
  pub trait Tool: Send + Sync {
      fn name(&self) -> &'static str;
      fn description(&self) -> &'static str;
      fn parameters_schema(&self) -> Value;
      async fn execute(&self, args: Value) -> Result<Value, ToolError>;
  }
  ```
- [ ] doc 注释 + 示例

#### 1.2.5 Memory Trait
- [ ] 定义 `Memory` trait
  ```rust
  #[async_trait]
  pub trait Memory: Send + Sync {
      async fn add(&self, message: Message) -> Result<(), MemoryError>;
      async fn get(&self, limit: usize) -> Result<Vec<Message>, MemoryError>;
      async fn clear(&self) -> Result<(), MemoryError>;
      async fn count(&self) -> Result<usize, MemoryError>;
  }
  ```
- [ ] 定义 `SemanticMemory` trait (向量检索)
  ```rust
  #[async_trait]
  pub trait SemanticMemory: Send + Sync {
      async fn add(&self, content: String, embedding: Vec<f32>) -> Result<(), MemoryError>;
      async fn search(&self, query: Vec<f32>, top_k: usize) -> Result<Vec<MemoryResult>, MemoryError>;
  }
  ```
- [ ] 定义 `MemoryResult` 结构体

#### 1.2.6 StateMachine Trait
- [ ] 定义 `StateMachine` trait
  ```rust
  pub trait StateMachine: Send + Sync {
      type State: Clone + Send + Sync;
      type Event: Send + Sync;
      type Output: Send + Sync;
      fn transition(&self, state: Self::State, event: Self::Event)
          -> Result<StateTransition<Self::State, Self::Output>, StateError>;
  }
  ```
- [ ] 定义 `StateTransition<S, O>` 枚举
  ```rust
  pub enum StateTransition<S, O> {
      Continue(S),
      Output(O, S),
      Done(O),
  }
  ```

---

### 1.3 消息类型 (`src/message.rs`) (Day 7-8)

#### 1.3.1 Message 枚举
- [ ] `Message` 枚举定义
  ```rust
  pub enum Message {
      User(UserMessage),
      Assistant(AssistantMessage),
      System(SystemMessage),
      Tool(ToolMessage),
  }
  ```
- [ ] `UserMessage` 结构体 (content, timestamp)
- [ ] `AssistantMessage` 结构体 (content, tool_calls, timestamp)
- [ ] `SystemMessage` 结构体 (content, timestamp)
- [ ] `ToolMessage` 结构体 (tool_call_id, content, timestamp)
- [ ] 实现 `From<String>` for `UserMessage`
- [ ] 实现 `Display` for `Message`
- [ ] 单元测试

#### 1.3.2 ToolCall 结构
- [ ] `ToolCall` 结构体 (id, name, arguments)
- [ ] `ToolResult` 结构体 (call_id, result)
- [ ] `Serialize`/`Deserialize` 实现
- [ ] 单元测试

#### 1.3.3 MessageBuilder
- [ ] `MessageBuilder` 结构体
- [ ] `user()` / `assistant()` / `system()` / `tool()` 方法
- [ ] `build()` 方法
- [ ] 单元测试

---

### 1.4 错误类型 (`src/error.rs`) (Day 9-10)

#### 1.4.1 AgentError
- [ ] 定义 `AgentError` 枚举
  ```rust
  pub enum AgentError {
      ExecutionFailed(String),
      LlmError(LlmError),
      ToolError(ToolError),
      InvalidState(String),
      Timeout(String),
  }
  ```
- [ ] `thiserror::Error` 派生
- [ ] `From<LlmError>` 实现
- [ ] `From<ToolError>` 实现

#### 1.4.2 LlmError
- [ ] 定义 `LlmError` 枚举
  ```rust
  pub enum LlmError {
      ApiError { code: u16, message: String },
      RateLimitError { retry_after: Option<u64> },
      AuthenticationError,
      InvalidRequest(String),
      NetworkError(String),
      ParsingError(String),
      StreamClosed,
  }
  ```

#### 1.4.3 ToolError / MemoryError / StateError
- [ ] `ToolError` 枚举定义
- [ ] `MemoryError` 枚举定义
- [ ] `StateError` 枚举定义
- [ ] `thiserror::Error` 派生
- [ ] 单元测试

---

### 1.5 测试 (Day 11-14)

#### 1.5.1 单元测试
- [ ] traits.rs 测试 (覆盖率 >80%)
- [ ] message.rs 测试 (覆盖率 100%)
- [ ] error.rs 测试 (覆盖率 90%)
- [ ] 覆盖率报告生成

#### 1.5.2 文档测试
- [ ] 所有 trait doc 示例
- [ ] `cargo test --doc` 通过
- [ ] `cargo doc --open` 无警告

---

### M1 交付物
- [ ] 可编译的 `langgraph` crate
- [ ] 完整的 trait 定义
- [ ] 单元测试覆盖率 >80%

---

## M2: 状态机与 Agent (Week 3-4, Day 15-28)

### 目标
实现状态机执行器和基础 Agent。

---

### 2.1 状态机 (`src/state.rs`) (Day 15-18)

#### 2.1.1 Runner 执行器
- [ ] `Runner<S, E, O>` 结构体
  ```rust
  pub struct Runner<S: Clone, E, O> {
      machine: Box<dyn StateMachine<State = S, Event = E, Output = O>>,
      state: S,
      max_steps: usize,
  }
  ```
- [ ] `new()` 构造函数
- [ ] `with_max_steps()` 方法
- [ ] `run()` 方法 (迭代 events)
- [ ] `run_stream()` 方法
- [ ] 步数限制检查

#### 2.1.2 类型状态模式 (可选)
- [ ] `Init` / `Running` / `Done` 标记
- [ ] `TypeStateMachine<S>` 结构体
- [ ] `start()` / `finish()` 方法
- [ ] 编译时状态检查

#### 2.1.3 Checkpoint 持久化
- [ ] `Checkpoint` trait 定义
  ```rust
  #[async_trait]
  pub trait Checkpoint: Send + Sync {
      async fn save(&self, key: &str, state: Vec<u8>) -> Result<(), CheckpointError>;
      async fn load(&self, key: &str) -> Result<Option<Vec<u8>>, CheckpointError>;
  }
  ```
- [ ] `MemoryCheckpoint` 实现 (`RwLock<HashMap>`)
- [ ] `FileCheckpoint` 实现
- [ ] 单元测试

---

### 2.2 Chat Agent (`src/agent/chat.rs`) (Day 19-22)

#### 2.2.1 ChatAgent 结构体
- [ ] `ChatAgent<C>` 结构体
  ```rust
  pub struct ChatAgent<C: LlmClient> {
      llm: C,
      system_prompt: Option<String>,
      memory: Option<Arc<dyn Memory>>,
      max_history: Option<usize>,
  }
  ```
- [ ] `ChatInput` 结构体
- [ ] `ChatOutput` 结构体
- [ ] `Agent` trait 实现
- [ ] `new()` / `builder()` 方法
- [ ] `with_system_prompt()` / `with_memory()` 方法

#### 2.2.2 SessionMemory
- [ ] `SessionMemory` 结构体
  ```rust
  pub struct SessionMemory {
      messages: Arc<RwLock<Vec<Message>>>,
      capacity: usize,
  }
  ```
- [ ] `Memory` trait 实现
- [ ] FIFO 容量限制
- [ ] token 计数估算

#### 2.2.3 流式响应
- [ ] `StreamAgent` trait 实现
- [ ] `ChatStreamEvent` 枚举
  ```rust
  pub enum ChatStreamEvent {
      Token(String),
      Done(String),
      Error(LlmError),
  }
  ```
- [ ] `run_stream()` 方法

#### 2.2.4 PromptTemplate
- [ ] `PromptTemplate` 结构体
- [ ] `render(variables: HashMap)` 方法
- [ ] 支持 `{{var}}` 语法
- [ ] 支持 `{{#if}}...{{/if}}` 条件

---

### 2.3 ReAct Agent (`src/agent/react.rs`) (Day 23-26)

#### 2.3.1 ReActState 枚举
- [ ] `ReActState` 枚举
  ```rust
  pub enum ReActState {
      Thinking { query: String, iterations: u32 },
      Acting { tool_calls: Vec<ToolCall> },
      Observing { results: Vec<ToolResult> },
      Done { answer: String },
  }
  ```
- [ ] `Clone` / `Debug` / `Display` 实现

#### 2.3.2 ThinkNode
- [ ] `ThinkNode<C>` 结构体
- [ ] `build_prompt()` 方法
- [ ] `parse_thought()` 方法 (提取思考内容)
- [ ] `extract_tool_calls()` 方法 (解析工具调用)
- [ ] 单元测试

#### 2.3.3 ActNode
- [ ] `ActNode` 结构体
- [ ] `execute(tool_calls)` 方法
- [ ] 工具并发执行 (`tokio::spawn`)
- [ ] 错误处理和重试
- [ ] 单元测试

#### 2.3.4 ObserveNode
- [ ] `ObserveNode` 结构体
- [ ] `process(results)` 方法
- [ ] `should_continue()` 判断

#### 2.3.5 ReAct Prompt
- [ ] `DEFAULT_REACT_PROMPT` 常量
- [ ] `build_prompt(query, tools, history)` 函数
- [ ] `format_tool_description()` 函数

---

### 2.4 LLM 实现 (`src/llm/`) (Day 27-28)

#### 2.4.1 OpenAI Client
- [ ] `OpenAiConfig` 结构体
  ```rust
  pub struct OpenAiConfig {
      pub api_key: String,
      pub base_url: Option<String>,
      pub model: String,
      pub default_temperature: f32,
  }
  ```
- [ ] `OpenAiClient` 结构体
- [ ] `LlmClient` trait 实现
- [ ] `chat()` 方法 (HTTP 请求 + 解析)
- [ ] `chat_stream()` 方法
- [ ] 单元测试 (mock HTTP)

#### 2.4.2 流式响应
- [ ] `SseStream` 结构体
- [ ] SSE 解析器
- [ ] `Stream` trait 实现
- [ ] 处理 `[DONE]` 标记

#### 2.4.3 重试逻辑
- [ ] `RetryConfig` 结构体
  ```rust
  pub struct RetryConfig {
      pub max_retries: u32,
      pub initial_delay: Duration,
      pub backoff_factor: f64,
  }
  ```
- [ ] `is_retryable()` for `LlmError`
- [ ] 指数退避 + jitter
- [ ] 单元测试

---

### 2.5 示例 (Day 28)

- [ ] `examples/chat.rs` - Chat Agent 示例
- [ ] `examples/react.rs` - ReAct Agent 示例
- [ ] 各示例的 `README.md`

---

### M2 交付物
- [ ] 可运行的 `ChatAgent`
- [ ] 可运行的 `ReActAgent`
- [ ] OpenAI LLM 实现
- [ ] 2 个示例

---

## M3: 工具与记忆系统 (Week 5-6, Day 29-42)

### 目标
实现类型安全的工具调用系统和记忆功能。

---

### 3.1 工具注册 (`src/tool/registry.rs`) (Day 29-31)

#### 3.1.1 ToolRegistry
- [ ] `ToolRegistry` 结构体
  ```rust
  pub struct ToolRegistry {
      tools: HashMap<&'static str, Box<dyn Tool>>,
  }
  ```
- [ ] `new()` 构造函数
- [ ] `register(tool)` 方法
- [ ] `get(name)` 方法
- [ ] `list()` 方法
- [ ] `execute(name, args)` 方法
- [ ] `execute_all()` 并发执行

#### 3.1.2 DynTool
- [ ] `DynTool` trait (类型擦除)
- [ ] `Box<dyn Tool>` 实现

#### 3.1.3 工具验证
- [ ] JSON Schema 验证
- [ ] `validate_args(schema, args)` 方法
- [ ] `ValidationError` 定义

---

### 3.2 内置工具 (`src/tool/builtin.rs`) (Day 32-34)

#### 3.2.1 HttpRequest 工具
- [ ] `HttpRequestTool` 结构体
- [ ] `Tool` trait 实现
- [ ] 参数 schema (url, method, headers, body)
- [ ] HTTP 执行逻辑
- [ ] 单元测试 (mock)

#### 3.2.2 FileOps 工具
- [ ] `FileOpsTool` 结构体
- [ ] `Tool` trait 实现
- [ ] 参数 schema (operation, path, content)
- [ ] 支持 read/write/list/exists
- [ ] 路径安全检查
- [ ] 单元测试

#### 3.2.3 Calculator 工具
- [ ] `CalculatorTool` 结构体
- [ ] `Tool` trait 实现
- [ ] 使用 `evalexpr` crate
- [ ] 沙箱执行
- [ ] 单元测试

---

### 3.3 工具组合 (`src/tool/compose.rs`) (Day 35-36)

#### 3.3.1 ToolChain
- [ ] `ToolChain` 结构体
- [ ] `Tool` trait 实现
- [ ] 输出传递给下一个工具
- [ ] 单元测试

#### 3.3.2 ToolMap
- [ ] `ToolMap` 结构体
- [ ] 输入/输出映射函数
- [ ] 单元测试

#### 3.3.3 并发执行
- [ ] `ParallelTools` 结构体
- [ ] `join_all` 并发执行
- [ ] 结果聚合

---

### 3.4 记忆系统 (`src/memory/`) (Day 37-40)

#### 3.4.1 SessionMemory (`session.rs`)
- [ ] `SessionMemory` 结构体 (消息历史)
- [ ] `Memory` trait 实现
- [ ] FIFO 容量限制
- [ ] 时间窗口限制 (可选)
- [ ] 单元测试

#### 3.4.2 ProfileMemory (`profile.rs`)
- [ ] `ProfileMemory` 结构体 (长期记忆)
  ```rust
  pub struct ProfileMemory {
      storage: Arc<dyn StorageBackend>,
      summarizer: Option<Arc<dyn Summarizer>>,
  }
  ```
- [ ] `add_profile()` 方法
- [ ] `get_profile()` 方法
- [ ] `update_profile()` 方法
- [ ] `summarize()` 方法 (LLM 摘要)
- [ ] 单元测试

#### 3.4.3 VectorMemory (`vector.rs`)
- [ ] `VectorMemory` 结构体 (语义记忆)
  ```rust
  pub struct VectorMemory {
      embeddings: Vec<MemoryEmbedding>,
      dimension: usize,
  }
  ```
- [ ] `MemoryEmbedding` 结构体 (id, content, vector, metadata)
- [ ] `add(content, embedding)` 方法
- [ ] `search(query_embedding, top_k)` 方法
- [ ] 余弦相似度计算
- [ ] 单元测试

#### 3.4.4 Embedder (`src/llm/embedder.rs`)
- [ ] `Embedder` trait
  ```rust
  #[async_trait]
  pub trait Embedder: Send + Sync {
      async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>;
      async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbedError>;
  }
  ```
- [ ] `OpenAiEmbedder` 实现
- [ ] 单元测试

---

### 3.5 Memory Agent (`src/agent/memory.rs`) (Day 41-42)

#### 3.5.1 MemoryAgent 结构
- [ ] `MemoryAgent` 结构体
  ```rust
  pub struct MemoryAgent<C: LlmClient, E: Embedder> {
      llm: C,
      embedder: E,
      session_memory: Arc<SessionMemory>,
      profile_memory: Arc<ProfileMemory>,
      vector_memory: Arc<VectorMemory>,
  }
  ```
- [ ] `Agent` trait 实现

#### 3.5.2 信息提取与存储
- [ ] `extract_info()` 方法 (LLM 提取实体)
- [ ] `store_profile()` 方法
- [ ] `store_vector()` 方法

#### 3.5.3 记忆检索
- [ ] `retrieve_relevant()` 方法
- [ ] 语义搜索 + 知识注入
- [ ] 构建增强 prompt

#### 3.5.4 记忆管理策略
- [ ] 摘要策略 (消息数量阈值)
- [ ] 重要性评分
- [ ] 定期清理

---

### 3.6 示例 (Day 42)

- [ ] `examples/tools.rs` - 工具使用示例
- [ ] `examples/memory.rs` - 记忆系统示例
- [ ] `examples/rag.rs` - RAG 示例
- [ ] 各示例的 `README.md`

---

### M3 交付物
- [ ] 完整的工具系统
- [ ] 3+ 内置工具
- [ ] 记忆系统 (Session + Profile + Vector)
- [ ] Memory Agent
- [ ] 3 个示例

---

## M4: 多 Agent 协作 (Week 7-8, Day 43-56)

### 目标
实现 Actor 模式的多 Agent 系统。

---

### 4.1 Actor 框架 (`src/actor/`) (Day 43-46)

#### 4.1.1 ActorId 和 AgentMessage
- [ ] `ActorId` 新类型
- [ ] `AgentMessage` 枚举
  ```rust
  pub enum AgentMessage {
      Task(Task),
      Stop,
      Ping,
  }
  ```
- [ ] `Task` 结构体

#### 4.1.2 Handler Trait
- [ ] `Handler<S>` trait
  ```rust
  #[async_trait]
  pub trait Handler<S>: Send + Sync {
      async fn handle(&mut self, msg: AgentMessage, state: &mut S) -> Result<(), ActorError>;
  }
  ```

#### 4.1.3 ActorAgent
- [ ] `ActorAgent<S>` 结构体
  ```rust
  pub struct ActorAgent<S> {
      id: ActorId,
      inbox: mpsc::Receiver<AgentMessage>,
      state: S,
      handler: Box<dyn Handler<S>>,
  }
  ```
- [ ] `run()` 方法 (消息循环)

#### 4.1.4 Channel 通信
- [ ] `AgentChannel` 结构体
- [ ] `send()` / `send_timeout()` / `try_send()` 方法
- [ ] `ActorRef<S>` 引用类型
- [ ] `request()` 方法 (请求-响应模式)

#### 4.1.5 监督策略
- [ ] `SupervisionStrategy` 枚举
- [ ] `OneForOne` / `OneForAll` / `AllForOne` 策略
- [ ] 重启逻辑

---

### 4.2 Supervisor (`src/supervisor.rs`) (Day 47-49)

#### 4.2.1 Supervisor 结构
- [ ] `Supervisor` 结构体
  ```rust
  pub struct Supervisor {
      workers: Vec<ActorRef<WorkerState>>,
      router: Box<dyn Router>,
      llm: Arc<dyn LlmClient>,
  }
  ```
- [ ] `TaskResult` 结构体

#### 4.2.2 Router Trait
- [ ] `Router` trait
  ```rust
  pub trait Router: Send + Sync {
      fn route(&self, task: &Task, workers: &[ActorRef<WorkerState>]) -> Option<usize>;
  }
  ```

#### 4.2.3 路由策略
- [ ] `RoundRobinRouter` 实现
- [ ] `LeastBusyRouter` 实现
- [ ] `SemanticRouter` 实现 (LLM 决策)
- [ ] 单元测试

#### 4.2.4 负载均衡
- [ ] `WorkerMetrics` 结构体
- [ ] 指标收集
- [ ] `get_least_busy()` 方法

---

### 4.3 Worker 实现 (`src/worker/`) (Day 50-53)

#### 4.3.1 Worker Trait
- [ ] `Worker` trait
  ```rust
  #[async_trait]
  pub trait Worker: Send + Sync {
      fn name(&self) -> &str;
      fn description(&self) -> &str;
      async fn handle(&self, task: Task) -> TaskResult;
  }
  ```

#### 4.3.2 ResearcherWorker
- [ ] `ResearcherWorker` 结构体
- [ ] `Worker` trait 实现
- [ ] 搜索 + 聚合 + 验证逻辑
- [ ] 单元测试

#### 4.3.3 AnalystWorker
- [ ] `AnalystWorker` 结构体
- [ ] `Worker` trait 实现
- [ ] 分析 + 比较 + 洞察生成
- [ ] 单元测试

#### 4.3.4 WriterWorker
- [ ] `WriterWorker` 结构体
- [ ] `WritingStyle` 枚举
- [ ] `Worker` trait 实现
- [ ] 草稿 + 修订 + 格式化
- [ ] 单元测试

#### 4.3.5 Worker Actor 包装
- [ ] `WorkerActor<S>` 结构体
- [ ] `Worker` → `ActorAgent` 转换

---

### 4.4 工作流 (`src/workflow.rs`) (Day 54-55)

#### 4.4.1 Step 枚举
- [ ] `Step` 枚举
  ```rust
  pub enum Step {
      Execute { agent: ActorRef<()>, input: Value },
      Parallel { steps: Vec<Step> },
      Sequence { steps: Vec<Step> },
      Branch { condition: Condition, then: Box<Step>, else: Box<Step> },
      Loop { condition: Condition, body: Box<Step>, max_iterations: Option<usize> },
  }
  ```
- [ ] `Condition` 类型定义

#### 4.4.2 WorkflowExecutor
- [ ] `WorkflowExecutor` 结构体
- [ ] `execute(step)` 方法
- [ ] 串行执行逻辑
- [ ] 并发执行逻辑 (`join_all`)
- [ ] 条件分支逻辑
- [ ] 循环执行逻辑
- [ ] 单元测试

#### 4.4.3 WorkflowBuilder
- [ ] `WorkflowBuilder` 结构体
- [ ] `execute()` / `parallel()` / `sequence()` 方法
- [ ] `branch()` / `loop()` 方法
- [ ] `build()` 方法
- [ ] 单元测试

---

### 4.5 示例 (Day 56)

- [ ] `examples/multi-agent.rs` - 多 Agent 示例
- [ ] `examples/research-team.rs` - 研究团队示例
- [ ] 各示例的 `README.md`

---

### M4 交付物
- [ ] Actor 框架
- [ ] Supervisor 实现
- [ ] 3 个内置 Worker
- [ ] 工作流引擎
- [ ] 2 个示例

---

## M5: 生产级实现 (Week 9-10, Day 57-70)

### 目标
HTTP API、可观测性、部署支持。

---

### 5.1 HTTP API (`src/api/`) (Day 57-59)

#### 5.1.1 路由配置
- [ ] `ApiState` 结构体
- [ ] `create_router()` 函数
- [ ] `GET /health` 健康检查
- [ ] `GET /metrics` Prometheus 指标

#### 5.1.2 Chat 端点
- [ ] `POST /api/v1/chat` 单轮对话
  - [ ] `ChatRequest` / `ChatResponse` 结构体
  - [ ] `chat_handler()` 函数
- [ ] `POST /api/v1/chat/stream` 流式对话
  - [ ] SSE 支持
  - [ ] `stream_chat_handler()` 函数

#### 5.1.3 Agent 端点
- [ ] `POST /api/v1/react` ReAct 执行
- [ ] `POST /api/v1/workflow` 工作流执行
- [ ] `GET /api/v1/agents` 列出 Agent

#### 5.1.4 中间件
- [ ] CORS 中间件
- [ ] 速率限制中间件 (tower_governor)
- [ ] 请求 ID 中间件
- [ ] 日志中间件

---

### 5.2 配置 (`src/config.rs`) (Day 60)

#### 5.2.1 AppConfig
- [ ] `AppConfig` 结构体
  ```rust
  pub struct AppConfig {
      pub server: ServerConfig,
      pub llm: LlmConfig,
      pub logging: LoggingConfig,
      pub telemetry: TelemetryConfig,
  }
  ```
- [ ] `ServerConfig` (host, port, workers)
- [ ] `LlmConfig` (provider, api_key, model)
- [ ] `LoggingConfig` (level, format)
- [ ] `TelemetryConfig` (metrics, tracing, jaeger_endpoint)

#### 5.2.2 环境变量加载
- [ ] `from_env()` 方法
- [ ] `LANGGRAPH_` 前缀支持
- [ ] 配置文件支持 (TOML)

#### 5.2.3 多环境支持
- [ ] `Profile` 枚举
- [ ] `Dev` / `Test` / `Prod` 配置

---

### 5.3 可观测性 (`src/telemetry/`) (Day 61-63)

#### 5.3.1 Metrics
- [ ] `prometheus` 集成
- [ ] 指标定义
  - `agent_requests_total` (Counter)
  - `agent_duration_seconds` (Histogram)
  - `llm_tokens_total` (Counter)
  - `llm_duration_seconds` (Histogram)
  - `tool_calls_total` (Counter)
  - `active_agents` (Gauge)
- [ ] `/metrics` 端点

#### 5.3.2 Tracing
- [ ] `opentelemetry` 集成
- [ ] Span 层级 (agent_execution, llm_call, tool_execution)
- [ ] Jaeger 导出器
- [ ] Stdout 导出器 (开发)

#### 5.3.3 结构化日志
- [ ] `tracing-subscriber` 配置
- [ ] JSON 格式 (生产)
- [ ] Pretty 格式 (开发)
- [ ] 请求 ID 注入

---

### 5.4 部署 (Day 64-65)

#### 5.4.1 Dockerfile
- [ ] 多阶段构建
- [ ] 非 root 用户
- [ ] 健康检查

#### 5.4.2 docker-compose.yml
- [ ] langgraph 服务
- [ ] 可选 PostgreSQL
- [ ] 可选 Redis
- [ ] 网络配置

#### 5.4.3 健康检查
- [ ] `HealthStatus` 结构体
- [ ] `check_liveness()` 方法
- [ ] `check_readiness()` 方法
- [ ] 依赖检查

---

### 5.5 文档 (Day 66-70)

#### 5.5.1 API 文档
- [ ] `utoipa` 集成
- [ ] `#[utoipa::path]` 标注
- [ ] Swagger UI (`/api/docs`)
- [ ] ReDoc (`/api/redoc`)
- [ ] OpenAPI spec (`/api/openapi.json`)

#### 5.5.2 教程
- [ ] `docs/getting-started.md`
  - 环境要求 / 安装 / 快速开始
- [ ] `docs/agents.md`
  - Chat Agent / ReAct Agent / 自定义 Agent
- [ ] `docs/tools.md`
  - 内置工具 / 自定义工具 / 工具组合
- [ ] `docs/memory.md`
  - Session / Profile / Vector 记忆
- [ ] `docs/multi-agent.md`
  - Worker / Supervisor / 工作流
- [ ] `docs/deployment.md`
  - Docker / 配置 / 监控

#### 5.5.3 示例更新
- [ ] 所有示例添加 `README.md`
- [ ] 确保可运行
- [ ] 添加详细注释

---

### M5 交付物
- [ ] HTTP API 服务
- [ ] 配置系统
- [ ] 可观测性集成
- [ ] Docker 部署
- [ ] 完整文档

---

## Crate 结构

```
crates/langgraph/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── traits.rs         # 核心 trait
│   ├── message.rs        # 消息类型
│   ├── error.rs          # 错误类型
│   ├── state.rs          # 状态机
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── chat.rs       # Chat Agent
│   │   ├── react.rs      # ReAct Agent
│   │   └── memory.rs     # Memory Agent
│   ├── tool/
│   │   ├── mod.rs
│   │   ├── registry.rs   # 工具注册
│   │   ├── builtin.rs    # 内置工具
│   │   └── compose.rs    # 工具组合
│   ├── memory/
│   │   ├── mod.rs
│   │   ├── session.rs    # 会话记忆
│   │   ├── profile.rs    # 长期记忆
│   │   └── vector.rs     # 语义记忆
│   ├── actor/
│   │   ├── mod.rs
│   │   ├── channel.rs    # Channel 通信
│   │   └── supervise.rs  # 监督策略
│   ├── supervisor.rs     # Supervisor
│   ├── worker/
│   │   ├── mod.rs
│   │   ├── researcher.rs
│   │   ├── analyst.rs
│   │   └── writer.rs
│   ├── workflow.rs       # 工作流
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── openai.rs
│   │   ├── stream.rs
│   │   ├── retry.rs
│   │   └── embedder.rs   # 向量嵌入
│   ├── config.rs         # 配置
│   ├── telemetry.rs      # 可观测性
│   └── api/              # HTTP API
│       ├── mod.rs
│       ├── routes.rs
│       ├── handlers.rs
│       ├── sse.rs
│       └── middleware.rs
├── examples/
│   ├── chat/
│   ├── react/
│   ├── tools/
│   ├── memory/
│   ├── rag/
│   ├── multi-agent/
│   └── research-team/
├── tests/
└── benches/
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

1. **立即 (Day 1)**: 创建 `crates/langgraph/` 目录，配置 workspace
2. **本周 (Day 1-5)**: 实现核心 trait 和消息类型
3. **下周 (Day 6-10)**: 实现状态机和第一个 Agent

---

*最后更新: 2025-01-27*
