# Rust LangGraph Agent 开发指南

本文档介绍如何使用 **Rust 风格** 的设计模式构建 Agent 应用。

## 设计理念

| Python LangGraph | Rust LangGraph |
|------------------|----------------|
| 动态类型状态 | 强类型状态机 |
| 函数闭包 | Trait 对象 |
| 运行时路由 | 编译时路由 |
| 字典传值 | 泛型约束 |
| 同步阻塞 | 异步流 |

### 核心原则

1. **编译时正确性** - 用类型系统代替运行时检查
2. **零成本抽象** - Trait 和泛型，不运行时反射
3. **显式优于隐式** - 状态机枚举，不魔法字符串
4. **所有权明确** - 不共享可变状态，用 channel 通信

## 文档目录

| 文档 | 内容 |
|------|------|
| [01-concepts.md](01-concepts.md) | 核心概念和架构 |
| [02-state-machine.md](02-state-machine.md) | 状态机设计 |
| [03-traits.md](03-traits.md) | Trait 抽象 |
| [04-agent.md](04-agent.md) | Agent 实现 |
| [09-minimal-agent-design.md](09-minimal-agent-design.md) | **最简 Rust Agent 设计方案**（参考 LangGraph） |
| [10-reducer-design.md](10-reducer-design.md) | **Reducer 设计**（partial state 合并、add_messages） |
| [11-state-graph-design.md](11-state-graph-design.md) | **StateGraph 设计**（线性链、Node、invoke、扩展路径） |
| [13-react-agent-design.md](13-react-agent-design.md) | ReAct Agent 设计（Think/Observe、LlmClient、tool_calls） |
| [14-chat-openai.md](14-chat-openai.md) | **最简单版本 ChatOpenAI 方案**（ChatOpenAI、ChatZhipu、LlmClient、数据流与实现选项） |
| [15-llm-react-agent.md](15-llm-react-agent.md) | **LLM + ReAct Agent 方案**（ChatOpenAI/ChatZhipu 接入 ReAct 图、with_tools、多轮、任务表） |
| [mcp-integration/README.md](mcp-integration/README.md) | **用 MCP 替代 Tool 的最简集成**（ToolSource、MCP 客户端、实现说明见 [implementation](mcp-integration/implementation.md)） |
| [05-tools.md](05-tools.md) | 工具调用系统 |
| [06-memory-agent.md](06-memory-agent.md) | **记忆 Agent** |
| [MEMORY_VS_LANGGRAPH_STORE.md](MEMORY_VS_LANGGRAPH_STORE.md) | rust-langgraph Memory 与 LangGraph MemoryStore 对照 |
| [07-multi-agent.md](07-multi-agent.md) | 多 Agent 协作 |
| [08-production.md](08-production.md) | 生产级实现 |

## 快速对比

### Python 风格 (避免)

```rust
// ❌ 过度使用 Option 嵌套
pub struct State {
    pub thought: Option<String>,
    pub action: Option<String>,
    pub observation: Option<String>,
}

// ❌ 字符串解析判断状态
fn should_continue(&self) -> bool {
    self.thought.as_ref()
        .map_or(false, |t| t.contains("完成"))
}

// ❌ 闭包捕获
pub struct Tool {
    handler: Arc<dyn Fn(Value) -> Pin<Box<dyn Future<Output = String>>>>,
}
```

### Rust 风格 (推荐)

```rust
// ✅ 状态机枚举
pub enum AgentState {
    Thinking { prompt: String },
    Acting { tool: ToolCall },
    Observing { result: String },
    Done { answer: String },
}

// ✅ 模式匹配
fn is_done(&self) -> bool {
    matches!(self, AgentState::Done { .. })
}

// ✅ Trait 定义
#[async_trait]
pub trait Tool: Send + Sync {
    type Input: DeserializeOwned;
    type Output: Serialize;

    fn name(&self) -> &str;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError>;
}
```

## 项目结构

```
crates/langgraph/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── context.rs      # Agent 上下文
│   │   └── runner.rs       # 执行器
│   ├── state/
│   │   ├── mod.rs
│   │   └── machine.rs      # 状态机
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── client.rs       # LLM client trait
│   │   └── stream.rs       # 流式响应
│   ├── tool/
│   │   ├── mod.rs
│   │   ├── registry.rs     # 工具注册
│   │   └── invoke.rs       # 工具调用
│   └── rt/
│       ├── mod.rs
│       └── executor.rs     # 异步执行器
└── examples/
    ├── chat/
    ├── react/
    └── multi/
```

## 简单示例

```rust
use langgraph::prelude::*;

// 定义状态机
#[derive(Debug)]
enum MyState {
    Idle,
    Processing { count: usize },
    Done { result: String },
}

// 定义 Agent
struct MyAgent {
    llm: Arc<dyn LlmClient>,
}

impl Agent for MyAgent {
    type State = MyState;
    type Output = String;

    async fn run(&self, input: &str) -> Result<Self::Output, AgentError> {
        let mut state = MyState::Idle;

        loop {
            state = match state {
                MyState::Idle => {
                    let response = self.llm.complete(input).await?;
                    MyState::Processing { count: 0 }
                }
                MyState::Processing { count } if count < 3 => {
                    MyState::Processing { count: count + 1 }
                }
                MyState::Processing { .. } => {
                    MyState::Done { result: "完成".to_string() }
                }
                MyState::Done { result } => {
                    return Ok(result);
                }
            };
        }
    }
}
```

## 参考资料

- [LangChain vs LangGraph 对比](LANGCHAIN_COMPARISON.md) - 框架对比分析
- [LangGraph 官方文档](https://langchain-ai.github.io/langgraph/)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)
- [Tokio 官方文档](https://tokio.rs/)
- [Python 版本文档](../langgraph-agent/README.md)
