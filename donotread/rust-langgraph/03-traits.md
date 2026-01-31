# Trait 抽象

用 trait 定义行为，实现编译时多态和零成本抽象。

## 核心 Trait

```rust
/// Agent trait
#[async_trait]
pub trait Agent: Send + Sync {
    /// 状态类型
    type State: Clone + Send + Sync + 'static;

    /// 输入类型
    type Input: DeserializeOwned + Send + 'static;

    /// 输出类型
    type Output: Serialize + Send + 'static;

    /// 运行 Agent
    async fn run(&self, input: Self::Input) -> Result<Self::Output, AgentError>;
}
```

## LLM Trait

```rust
/// LLM 客户端
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 非流式完成
    async fn complete(&self, prompt: &str) -> Result<String, LlmError>;

    /// 聊天
    async fn chat(&self, messages: &[Message]) -> Result<ChatResponse, LlmError>;

    /// 流式完成
    fn stream_complete<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send + 'a>>;
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    System(String),
    User(String),
    Assistant(String),
    Tool { name: String, content: String },
}

/// 聊天响应
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub finish_reason: FinishReason,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    Error,
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

## 工具 Trait

```rust
/// 工具定义
#[async_trait]
pub trait Tool: Send + Sync {
    /// 输入类型 - 关联类型实现编译时类型安全
    type Input: DeserializeOwned + Send + 'static;

    /// 输出类型
    type Output: Serialize + Send + 'static;

    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 参数 schema (JSON Schema)
    fn schema(&self) -> &serde_json::Value;

    /// 执行工具
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError>;
}

/// 简单工具 - 用于无参数的工具
#[async_trait]
pub trait SimpleTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self) -> Result<String, ToolError>;
}

/// 为实现 SimpleTool 的类型自动实现 Tool
#[async_trait]
impl<T: SimpleTool + 'static> Tool for T {
    type Input = ();
    type Output = String;

    fn name(&self) -> &str {
        SimpleTool::name(self)
    }

    fn description(&self) -> &str {
        SimpleTool::description(self)
    }

    fn schema(&self) -> &serde_json::Value {
        &serde_json::json!({"type": "object"})
    }

    async fn execute(&self, _input: ()) -> Result<Self::Output, ToolError> {
        SimpleTool::execute(self).await
    }
}
```

## 工具实现示例

```rust
/// 计算器工具
pub struct Calculator;

#[async_trait]
impl Tool for Calculator {
    type Input = CalculatorInput;
    type Output = CalculatorOutput;

    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "执行数学计算"
    }

    fn schema(&self) -> &serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式，如 1 + 2"
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        // 简单实现
        Ok(CalculatorOutput {
            result: format!("计算结果: {}", input.expression),
        })
    }
}

#[derive(Deserialize)]
pub struct CalculatorInput {
    pub expression: String,
}

#[derive(Serialize)]
pub struct CalculatorOutput {
    pub result: String,
}

/// 搜索工具
pub struct Search {
    client: reqwest::Client,
    api_key: String,
}

#[async_trait]
impl Tool for Search {
    type Input = SearchInput;
    type Output = Vec<SearchResult>;

    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "搜索网络信息"
    }

    fn schema(&self) -> &serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        let response = self.client
            .get(&format!("https://api.search.com?q={}", input.query))
            .header("Authorization", &self.api_key)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }
}

#[derive(Deserialize)]
pub struct SearchInput {
    pub query: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}
```

## 工具组合

```rust
/// 工具链
pub struct ToolChain<A, B> {
    a: A,
    b: B,
}

impl<A, B> ToolChain<A, B>
where
    A: Tool,
    B: Tool<Input = A::Output>,
{
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

#[async_trait]
impl<A, B> Tool for ToolChain<A, B>
where
    A: Tool + Send + Sync,
    B: Tool<Input = A::Output> + Send + Sync,
{
    type Input = A::Input;
    type Output = B::Output;

    fn name(&self) -> &str {
        &format!("{}_chain_{}", self.a.name(), self.b.name())
    }

    fn description(&self) -> &str {
        self.a.description()
    }

    fn schema(&self) -> &serde_json::Value {
        self.a.schema()
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        let mid = self.a.execute(input).await?;
        self.b.execute(mid).await
    }
}

/// 工具映射 - 将输入映射后执行
pub struct ToolMap<T, F> {
    tool: T,
    f: Arc<dyn Fn(serde_json::Value) -> T::Input + Send + Sync>,
}

impl<T, F> ToolMap<T, F>
where
    T: Tool,
    F: Fn(serde_json::Value) -> T::Input + Send + Sync + 'static,
{
    pub fn new(tool: T, f: F) -> Self {
        Self {
            tool,
            f: Arc::new(f),
        }
    }
}

#[async_trait]
impl<T, F> Tool for ToolMap<T, F>
where
    T: Tool + Send + Sync,
    F: Fn(serde_json::Value) -> T::Input + Send + Sync + 'static,
{
    type Input = serde_json::Value;
    type Output = T::Output;

    fn name(&self) -> &str {
        self.tool.name()
    }

    fn description(&self) -> &str {
        self.tool.description()
    }

    fn schema(&self) -> &serde_json::Value {
        self.tool.schema()
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        let mapped = (self.f)(input);
        self.tool.execute(mapped).await
    }
}
```

## 内存 Trait

```rust
/// 记忆存储
#[async_trait]
pub trait Memory: Send + Sync {
    /// 存储消息
    async fn store(&self, key: &str, value: &str) -> Result<(), MemoryError>;

    /// 获取消息
    async fn get(&self, key: &str) -> Result<Option<String>, MemoryError>;

    /// 搜索相关消息
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<String>, MemoryError>;
}

/// 会话记忆
pub struct ConversationMemory<M: Memory> {
    memory: M,
    session_id: String,
}

impl<M: Memory> ConversationMemory<M> {
    pub fn new(memory: M, session_id: String) -> Self {
        Self { memory, session_id }
    }

    pub async fn add(&self, role: &str, content: &str) -> Result<(), MemoryError> {
        let key = format!("{}:{}", self.session_id, uuid::Uuid::new_v4());
        self.memory.store(&key, content).await
    }

    pub async fn get_history(&self) -> Result<Vec<String>, MemoryError> {
        self.memory.search(&self.session_id, 100).await
    }
}
```
