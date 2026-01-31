# 核心概念

Rust 风格的 Agent 架构设计。

## 核心抽象

```
┌─────────────────────────────────────────────────────────┐
│                      Agent                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │    LLM      │  │   Tools     │  │   Memory    │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                   State Machine                         │
│  ┌───────┐    ┌───────┐    ┌───────┐    ┌───────┐    │
│  │ Idle  │───▶│ Think │───▶│  Act  │───▶│  Done │    │
│  └───────┘    └───────┘    └───────┘    └───────┘    │
└─────────────────────────────────────────────────────────┘
```

## Agent Trait

```rust
/// Agent 核心抽象
#[async_trait]
pub trait Agent: Send + Sync {
    /// 状态类型
    type State: Clone + Send + Sync;

    /// 输入类型
    type Input: DeserializeOwned + Send;

    /// 输出类型
    type Output: Serialize + Send;

    /// 运行 Agent
    async fn run(&self, input: Self::Input) -> Result<Self::Output, AgentError>;

    /// 流式运行
    fn run_stream(
        &self,
        input: Self::Input,
    ) -> Pin<Box<dyn Stream<Item = Event<Self::State>> + Send + '_>>;
}

/// 事件流
pub enum Event<S> {
    Transition(S),
    Output(String),
    Error(AgentError),
    Done,
}
```

## 状态机

```rust
/// 状态机 trait
pub trait StateMachine: Clone + Send + Sync + 'static {
    /// 下一状态
    fn next(self) -> StateTransition;
}

/// 状态转换
pub enum StateTransition {
    Continue(Box<dyn StateMachine>),
    Done,
    Branch(Vec<Box<dyn StateMachine>>),
}
```

## LLM 抽象

```rust
/// LLM 客户端 trait
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 完成请求
    async fn complete(&self, prompt: &str) -> Result<String, LlmError>;

    /// 聊天请求
    async fn chat(&self, messages: &[Message]) -> Result<ChatResponse, LlmError>;

    /// 流式完成
    fn stream_complete(
        &self,
        prompt: &str,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send + '_>>;
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    System(String),
    User(String),
    Assistant(String),
    Tool { name: String, content: String },
}
```

## 工具系统

```rust
/// 工具 trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// 输入类型
    type Input: DeserializeOwned + Send;

    /// 输出类型
    type Output: Serialize + Send;

    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 参数 schema
    fn schema(&self) -> &serde_json::Value;

    /// 执行工具
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError>;
}

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<Box<str>, Box<dyn DynTool>>,
}

/// 动态工具 trait (用于类型擦除)
#[async_trait]
pub trait DynTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> &serde_json::Value;
    async fn invoke(&self, input: serde_json::Value) -> Result<serde_json::Value, ToolError>;
}
```

## 执行器

```rust
/// 异步执行器
pub struct Executor {
    runtime: tokio::runtime::Handle,
    metrics: Arc<Metrics>,
}

impl Executor {
    /// 创建新执行器
    pub fn new() -> Self {
        Self {
            runtime: tokio::runtime::Handle::current(),
            metrics: Arc::new(Metrics::new()),
        }
    }

    /// 执行 Agent
    pub async fn execute<A: Agent>(
        &self,
        agent: &A,
        input: A::Input,
    ) -> Result<A::Output, AgentError> {
        let start = Instant::now();
        let result = agent.run(input).await;
        let duration = start.elapsed();

        self.metrics.record(duration, result.is_ok());

        result
    }

    /// 并发执行多个 Agent
    pub async fn execute_many<A: Agent>(
        &self,
        agents: &[A],
        inputs: Vec<A::Input>,
    ) -> Vec<Result<A::Output, AgentError>> {
        let futures: Vec<_> = agents
            .iter()
            .zip(inputs)
            .map(|(agent, input)| self.execute(agent, input))
            .collect();

        join_all(futures).await
    }
}
```

## 消息传递

```rust
/// Actor 风格的 Agent
pub struct ActorAgent<A: Agent> {
    agent: A,
    tx: mpsc::Sender<AgentCommand>,
}

/// Agent 命令
pub enum AgentCommand {
    Run {
        input: serde_json::Value,
        respond: oneshot::Sender<Result<serde_json::Value, AgentError>>,
    },
    Stream {
        input: serde_json::Value,
        respond: mpsc::Sender<StreamEvent>,
    },
}

impl<A: Agent> ActorAgent<A> {
    /// 启动 Actor
    pub fn spawn(agent: A) -> Self {
        let (tx, mut rx) = mpsc::channel(16);

        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    AgentCommand::Run { input, respond } => {
                        // 处理...
                    }
                    _ => {}
                }
            }
        });

        Self { agent, tx }
    }
}
```

## 设计原则总结

| 原则 | 说明 | 实现 |
|------|------|------|
| **类型安全** | 编译时保证正确 | 泛型、trait bounds |
| **零成本** | 无运行时开销 | monomorphization |
| **显式** | 状态变化可见 | 状态机枚举 |
| **组合** | 小组件组合大功能 | trait 组合 |
| **并发** | 安全的并发 | Send + Sync bounds |
