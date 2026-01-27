# Agent 实现

完整实现一个类型安全的 Agent。

## Chat Agent

```rust
use std::sync::Arc;
use std::pin::Pin;

/// 聊天 Agent
pub struct ChatAgent<L: LlmClient> {
    llm: L,
    system_prompt: String,
    history: Vec<Message>,
}

impl<L: LlmClient> ChatAgent<L> {
    pub fn new(llm: L) -> Self {
        Self {
            llm,
            system_prompt: "You are a helpful assistant.".to_string(),
            history: Vec::new(),
        }
    }

    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }

    pub fn with_memory(mut self, capacity: usize) -> Self {
        self.history = Vec::with_capacity(capacity);
        self
    }
}

#[async_trait]
impl<L: LlmClient + Send + Sync> Agent for ChatAgent<L> {
    type State = ChatState;
    type Input = String;
    type Output = String;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, AgentError> {
        let mut messages = vec![Message::System(self.system_prompt.clone())];
        messages.extend(self.history.clone());
        messages.push(Message::User(input));

        let response = self.llm.chat(&messages).await?;
        Ok(response.content)
    }

    fn run_stream(
        &self,
        input: Self::Input,
    ) -> Pin<Box<dyn Stream<Item = Event<Self::State>> + Send + '_>> {
        Box::pin(async_stream::stream! {
            yield Event::Start;

            let mut messages = vec![Message::System(self.system_prompt.clone())];
            messages.push(Message::User(input.clone()));

            let mut stream = self.llm.stream_complete(&messages.join("\n")).await;

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => yield Event::Chunk(text),
                    Err(e) => yield Event::Error(e.into()),
                }
            }

            yield Event::Done;
        })
    }
}

/// 聊天状态
#[derive(Debug, Clone)]
pub struct ChatState {
    pub messages: Vec<Message>,
}

/// 事件
pub enum Event<S> {
    Start,
    Chunk(String),
    State(S),
    Error(AgentError),
    Done,
}
```

## ReAct Agent

```rust
/// ReAct Agent
pub struct ReActAgent<L: LlmClient, T: ToolRegistry> {
    llm: L,
    tools: T,
    max_iterations: usize,
}

impl<L: LlmClient, T: ToolRegistry> ReActAgent<L, T> {
    pub fn new(llm: L, tools: T) -> Self {
        Self {
            llm,
            tools,
            max_iterations: 10,
        }
    }

    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }
}

#[async_trait]
impl<L: LlmClient + Send + Sync, T: ToolRegistry + Send + Sync> Agent for ReActAgent<L, T> {
    type State = ReActState;
    type Input = String;
    type Output = String;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, AgentError> {
        let mut state = ReActState::new(input);

        for _ in 0..self.max_iterations {
            match state {
                ReActState::Thinking { ref query, step } => {
                    // 思考下一步
                    let prompt = self.build_think_prompt(&state);
                    let response = self.llm.complete(&prompt).await?;

                    // 解析响应
                    if let Some(tool_call) = self.parse_tool_call(&response) {
                        state = ReActState::Acting {
                            query: query.clone(),
                            thought: response,
                            tool: tool_call,
                            step,
                        };
                    } else {
                        // 没有工具调用，直接返回答案
                        return Ok(response);
                    }
                }
                ReActState::Acting { ref tool, step, .. } => {
                    // 执行工具
                    let result = self.tools.execute(tool).await?;
                    state = ReActState::Observing {
                        observation: result,
                        step: step + 1,
                        // ...
                    };
                }
                ReActState::Observing { ref observation, step, .. } => {
                    // 观察结果，决定下一步
                    if step >= self.max_iterations {
                        return Ok(observation.clone());
                    }
                    state = ReActState::Thinking {
                        // ...
                        step,
                    };
                }
            }
        }

        Err(AgentError::MaxIterations)
    }
}

/// ReAct 状态
#[derive(Debug, Clone)]
pub enum ReActState {
    Thinking {
        query: String,
        thought: Option<String>,
        step: usize,
    },
    Acting {
        query: String,
        thought: String,
        tool: ToolCall,
        step: usize,
    },
    Observing {
        query: String,
        thought: String,
        observation: String,
        step: usize,
    },
}
```

## Agent Builder

```rust
/// Agent 构建器
pub struct AgentBuilder<L, T> {
    llm: Option<L>,
    tools: Option<T>,
    max_iterations: Option<usize>,
    system_prompt: Option<String>,
}

impl<L, T> AgentBuilder<L, T> {
    pub fn new() -> Self {
        Self {
            llm: None,
            tools: None,
            max_iterations: None,
            system_prompt: None,
        }
    }

    pub fn llm(mut self, llm: L) -> Self {
        self.llm = Some(llm);
        self
    }

    pub fn tools(mut self, tools: T) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = Some(n);
        self
    }

    pub fn system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }
}

impl<L: LlmClient, T: ToolRegistry> AgentBuilder<L, T> {
    pub fn build_chat(self) -> Result<ChatAgent<L>, BuildError> {
        let llm = self.llm.ok_or(BuildError::MissingLlm)?;
        let mut agent = ChatAgent::new(llm);

        if let Some(prompt) = self.system_prompt {
            agent = agent.with_system_prompt(prompt);
        }

        Ok(agent)
    }

    pub fn build_react(self) -> Result<ReActAgent<L, T>, BuildError> {
        let llm = self.llm.ok_or(BuildError::MissingLlm)?;
        let tools = self.tools.ok_or(BuildError::MissingTools)?;

        let mut agent = ReActAgent::new(llm, tools);

        if let Some(n) = self.max_iterations {
            agent = agent.with_max_iterations(n);
        }

        Ok(agent)
    }
}

// 使用示例
let agent = AgentBuilder::new()
    .llm(openai_client)
    .tools(tool_registry)
    .max_iterations(5)
    .build_react()?;
```

## 组合 Agent

```rust
/// 组合多个 Agent
pub struct CompositeAgent<A, B> {
    primary: A,
    fallback: B,
}

impl<A, B> CompositeAgent<A, B>
where
    A: Agent,
    B: Agent<Input = A::Input, Output = A::Output>,
{
    pub fn new(primary: A, fallback: B) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait]
impl<A, B> Agent for CompositeAgent<A, B>
where
    A: Agent + Send + Sync,
    B: Agent<Input = A::Input, Output = A::Output> + Send + Sync,
{
    type State = (A::State, B::State);
    type Input = A::Input;
    type Output = A::Output;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, AgentError> {
        self.primary
            .run(input.clone())
            .or_else(|_| self.fallback.run(input))
            .await
    }
}

/// 顺序执行的 Agent
pub struct SequenceAgent<A, B> {
    first: A,
    second: B,
}

impl<A, B> SequenceAgent<A, B>
where
    A: Agent,
    B: Agent<Input = A::Output>,
{
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

#[async_trait]
impl<A, B> Agent for SequenceAgent<A, B>
where
    A: Agent + Send + Sync,
    B: Agent<Input = A::Output> + Send + Sync,
{
    type State = (A::State, B::State);
    type Input = A::Input;
    type Output = B::Output;

    async fn run(&self, input: Self::Input) -> Result<Self::Output, AgentError> {
        let mid = self.first.run(input).await?;
        self.second.run(mid).await
    }
}
```
