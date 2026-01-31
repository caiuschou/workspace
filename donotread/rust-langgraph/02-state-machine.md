# 状态机设计

用枚举和 trait 构建类型安全的状态机。

## 状态机模式

```rust
/// 状态机 trait
pub trait StateMachine: Clone + Send + Sync + 'static {
    /// 下一状态
    fn transition(self, ctx: &mut Context) -> StateTransition;

    /// 当前状态名称
    fn name(&self) -> &str;
}

/// 状态转换结果
pub enum StateTransition {
    /// 继续到下一状态
    Next(Box<dyn StateMachine>),

    /// 等待异步操作
    Wait(Box<dyn Future<Output = Box<dyn StateMachine>> + Send>),

    /// 分支到多个状态
    Branch(Vec<Box<dyn StateMachine>>),

    /// 完成
    Done,

    /// 错误
    Error(AgentError),
}

/// Agent 上下文
pub struct Context {
    /// LLM 客户端
    pub llm: Arc<dyn LlmClient>,

    /// 工具注册表
    pub tools: Arc<ToolRegistry>,

    /// 用户输入
    pub input: String,

    /// 累积的数据
    pub data: HashMap<Box<str>, serde_json::Value>,
}
```

## ReAct 状态机示例

```rust
/// ReAct 状态机
#[derive(Debug, Clone)]
pub enum ReActMachine {
    /// 初始状态
    Start { query: String },

    /// 思考状态
    Thinking {
        query: String,
        thought: String,
        step: usize,
    },

    /// 行动状态
    Acting {
        query: String,
        thought: String,
        tool: ToolCall,
        step: usize,
    },

    /// 观察状态
    Observing {
        query: String,
        thought: String,
        observation: String,
        step: usize,
    },

    /// 完成状态
    Done {
        answer: String,
    },
}

impl StateMachine for ReActMachine {
    fn transition(self, ctx: &mut Context) -> StateTransition {
        match self {
            ReActMachine::Start { query } => {
                StateTransition::Next(Box::new(ReActMachine::Thinking {
                    query,
                    thought: String::new(),
                    step: 0,
                }))
            }

            ReActMachine::Thinking { query, step, .. } => {
                // 调用 LLM 思考
                let prompt = format!("问题: {}\n请思考下一步...", query);
                StateTransition::Wait(Box::new(async move {
                    let thought = ctx.llm.complete(&prompt).await.unwrap();
                    Box::new(ReActMachine::Thinking { query, thought, step })
                }))
            }

            ReActMachine::Acting { query, thought, tool, step } => {
                // 执行工具
                StateTransition::Wait(Box::new(async move {
                    let result = ctx.tools.invoke(&tool).await.unwrap();
                    Box::new(ReActMachine::Observing {
                        query,
                        thought,
                        observation: result,
                        step: step + 1,
                    })
                }))
            }

            ReActMachine::Observing { query, thought, observation, step } => {
                // 判断是否完成
                if step >= 10 {
                    StateTransition::Next(Box::new(ReActMachine::Done {
                        answer: observation,
                    }))
                } else {
                    StateTransition::Next(Box::new(ReActMachine::Thinking {
                        query,
                        thought: format!("{} -> {}", thought, observation),
                        step,
                    }))
                }
            }

            ReActMachine::Done { .. } => StateTransition::Done,
        }
    }

    fn name(&self) -> &str {
        match self {
            ReActMachine::Start { .. } => "start",
            ReActMachine::Thinking { .. } => "thinking",
            ReActMachine::Acting { .. } => "acting",
            ReActMachine::Observing { .. } => "observing",
            ReActMachine::Done { .. } => "done",
        }
    }
}
```

## 状态机运行器

```rust
/// 状态机运行器
pub struct Runner {
    ctx: Context,
    max_steps: usize,
}

impl Runner {
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx,
            max_steps: 100,
        }
    }

    /// 运行状态机
    pub async fn run(mut self, mut initial: Box<dyn StateMachine>) -> Result<String, AgentError> {
        let mut current = initial;
        let mut steps = 0;

        loop {
            if steps >= self.max_steps {
                return Err(AgentError::MaxStepsReached);
            }

            tracing::debug!("Step {}: {}", steps, current.name());

            current = match current.transition(&mut self.ctx) {
                StateTransition::Next(next) => next,
                StateTransition::Wait(fut) => fut.await,
                StateTransition::Branch(mut machines) => {
                    // 并发执行分支，取第一个成功的结果
                    let results: Vec<_> = futures::future::join_all(
                        machines.into_iter().map(|m| async move {
                            let mut runner = Runner::new(self.ctx.clone());
                            runner.run(m).await
                        })
                    )
                    .await;

                    results
                        .into_iter()
                        .find(|r| r.is_ok())
                        .unwrap()?
                }
                StateTransition::Done => return Ok(self.ctx.data.get("answer").unwrap().to_string()),
                StateTransition::Error(e) => return Err(e),
            };

            steps += 1;
        }
    }
}
```

## 类型状态模式

```rust
/// 类型状态：编译时保证状态转换正确
pub struct Agent<S> {
    state: PhantomData<S>,
    llm: Arc<dyn LlmClient>,
}

/// 状态标记
pub struct Uninitialized;
pub struct Ready;
pub struct Running;
pub struct Done;

impl Agent<Uninitialized> {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self {
            state: PhantomData,
            llm,
        }
    }

    pub fn initialize(self) -> Agent<Ready> {
        Agent {
            state: PhantomData,
            llm: self.llm,
        }
    }
}

impl Agent<Ready> {
    pub fn start(self, input: String) -> Agent<Running> {
        Agent {
            state: PhantomData,
            llm: self.llm,
        }
    }
}

impl Agent<Running> {
    pub async fn run(self) -> Result<String, AgentError> {
        self.llm.complete("...").await
    }

    pub fn finish(self) -> Agent<Done> {
        Agent {
            state: PhantomData,
            llm: self.llm,
        }
    }
}
```

## 状态机宏

```rust
/// 简化状态机定义的宏
#[macro_export]
macro_rules! state_machine {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $(
                $variant:ident $( { $($field:ident : $ty:ty),* } )?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $(
                $variant $( { $($field : $ty),* } )?
            ),*
        }

        impl StateMachine for $name {
            fn transition(self, ctx: &mut Context) -> StateTransition {
                match self {
                    $(
                        $name::$variant $( { $($field),* } )? => {
                            self.$variant_transition(ctx)
                        }
                    )*
                }
            }

            fn name(&self) -> &str {
                match self {
                    $(
                        $name::$variant $( { .. } )? => stringify!($variant),
                    )*
                }
            }
        }
    };
}

// 使用
state_machine! {
    pub enum MyMachine {
        Start { query: String },
        Thinking { query: String, step: usize },
        Done { answer: String },
    }
}
```
