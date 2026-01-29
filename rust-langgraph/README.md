# rust-langgraph

LangGraph 风格的 Rust 实现：**state-in, state-out** 最简 Agent。

设计文档：仓库内 [docs/rust-langgraph/09-minimal-agent-design.md](../docs/rust-langgraph/09-minimal-agent-design.md)。

## 快速开始

```bash
# 从仓库根目录
cargo run -p langgraph --example echo -- "你好"
# 输出：你好
```

## 包结构

- **`crates/langgraph`**：核心库（Agent trait、Message、AgentError）；**AgentState 与 EchoAgent 由 Example 实现**，不在框架内。
- **`crates/langgraph-react`**：占位，Sprint 4 实现 ReAct

## 最简用法

在示例中定义 State 与 Agent，使用框架提供的 trait 与类型：

```rust
use langgraph::{Agent, AgentError, Message};

// 在 example 或业务代码中定义 State 与 Agent
#[derive(Clone, Default)]
struct AgentState { pub messages: Vec<Message> }

struct EchoAgent;
#[async_trait]
impl Agent for EchoAgent {
    fn name(&self) -> &str { "echo" }
    type State = AgentState;
    async fn run(&self, state: Self::State) -> Result<Self::State, AgentError> { ... }
}

let mut state = AgentState::default();
state.messages.push(Message::User("你好".into()));
state = agent.run(state).await?;
let output = state.messages.last(); // e.g. Assistant("你好")
```

开发计划见 [docs/rust-langgraph/ROADMAP.md](../docs/rust-langgraph/ROADMAP.md)。
