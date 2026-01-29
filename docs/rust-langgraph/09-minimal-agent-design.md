# 最简单的 Rust 版 Agent 设计方案

参考 LangGraph，给出**最小可用**的 Rust agent 实现，便于先跑通再迭代。

---

## 1. LangGraph 与 Rust 的对应关系

| 要素 | LangGraph | Rust 最简 |
|------|-----------|-----------|
| **State** | `TypedDict`（如 `messages`） | 当前会话数据；无单独 Input/Output，`invoke(state)` 入、返回 state 出 |
| **Node** | `(state) -> partial state` | 一个可执行单元：接收状态、返回状态更新 |
| **Edge** | `START -> node -> END` | 调用方执行一次 `run(state)` 即完成一步 |

最简 agent = **一个状态（消息列表）+ 一个节点（如 chatbot）+ 固定边**。调用方把本轮输入放进 state 后 invoke，再从返回的 state 里取输出。

---

## 2. 设计目标

- **不先引入「图」类型**：用 trait 抽象「可执行的 agent」，等价于单节点 + 固定边。
- **仅 State，无 Input/Output**：与 LangGraph 对齐，签名为「state 进、state 出」。
- **最小类型**：统一错误、最小消息类型，便于后续接 LLM/Tools。

---

## 3. 最简 Agent 抽象

### 3.1 核心 Trait

```rust
/// Minimal agent: state in, state out. Aligns with LangGraph (no Input/Output).
#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;

    type State: Clone + Send + Sync + 'static;

    /// One step: receive state, return updated state.
    async fn run(&self, state: Self::State) -> Result<Self::State, AgentError>;
}
```

- `run(state)` = 执行当前唯一节点一次；入参、返回值均为 State。
- **State 由实现方定义**：`Agent::State` 的关联类型及字段由每个 Agent 实现自己决定；框架不强制统一 State 结构。
- 暂不引入：流式、工具、多节点图。

### 3.2 最小状态、消息与错误

- **State 的字段由实现方定义**：实现 `Agent` 时通过 `type State = YourStruct` 指定自己的状态类型（可含任意字段）。框架不提供 State 类型；`AgentState`（仅 `messages`）由 Example 实现，见 `examples/echo.rs`。

```rust
// 框架内：Message、AgentError
#[derive(Debug, Clone)]
pub enum Message {
    System(String),   // 系统提示，通常放消息列表最前
    User(String),     // 用户输入
    Assistant(String), // 模型/agent 回复
}

// AgentState 在 Example 中定义（examples/echo.rs），框架内无 State 类型
#[derive(Debug, Clone, Default)]
struct AgentState {
    pub messages: Vec<Message>,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}
```

与 LangGraph/LangChain 一致：System / User / Assistant 三种角色。

实现方自定义 State 示例（可选，不用框架的 `AgentState`）：

```rust
// 实现方自己定义状态字段
#[derive(Debug, Clone)]
struct MyState {
    pub messages: Vec<Message>,
    pub turn_count: u32,
}

impl Agent for MyAgent {
    type State = MyState;  // 由实现方定义
    // ...
}
```

---

## 4. 最简实现：Echo Agent

对应 ROADMAP **Sprint 1**，验证 trait 与类型设计。

```rust
pub struct EchoAgent;

impl EchoAgent {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Agent for EchoAgent {
    fn name(&self) -> &str { "echo" }
    type State = AgentState;

    async fn run(&self, state: Self::State) -> Result<Self::State, AgentError> {
        let mut messages = state.messages;
        let last = messages.last().and_then(|m| {
            if let Message::User(s) = m { Some(s.clone()) } else { None }
        });
        if let Some(content) = last {
            messages.push(Message::Assistant(content));
        }
        Ok(AgentState { messages })
    }
}
```

**使用方式**（与 LangGraph 一致：无 Input/Output，只有 State 进、State 出）：

```rust
let mut state = AgentState::default();
state.messages.push(Message::User("你好".into()));
state = agent.run(state).await?;
let output = state.messages.last(); // e.g. Assistant("你好")
```

---

## 5. 与 LangGraph 的逐层对应

| 层级 | LangGraph | 本方案（Rust） |
|------|-----------|----------------|
| 入/出 | `invoke(state)` 入、返回 state 出 | `run(state) -> State` |
| 状态 | `ChatState(messages)` | `AgentState { messages }` |
| 消息 | System / Human / AI | `Message::System` / `User` / `Assistant` |
| 节点 | `chatbot_node(state) -> partial` | `Agent::run(state) -> State` |
| 边 | `START -> chatbot -> END` | 调用方一次 `run(state)` |

**差异**：Python 用 channel + reducer 合并 partial state；本方案节点入参即整份 state，出参即整份新 state，合并在节点内部。若后续要做多节点或 partial + 合并，可参考 [10-reducer-design.md](10-reducer-design.md)。

---

## 6. 扩展路径

在保留最简 Agent 的前提下，按 ROADMAP 逐步加能力：

1. **S1**：EchoAgent（如上），无 Input/Output。
2. **S2**：`ChatAgent<L: LlmClient>`，`run(state)` 内取 messages、调 LLM、追加回复。
3. **S3**：加重试/流式/会话记忆，State 不变。
4. **S4**：ReAct，扩展 State（如 `ReActState`），加入工具与循环。

每一步保持「当前可运行的最简形态」。

---

## 7. 小结与归属

- **最简 Rust Agent** = trait `Agent`（`name` + `State` + `run(State)->Result<State, AgentError>`）+ 最小 Message/State/Error + EchoAgent。
- 与 LangGraph 一致：仅 State 进、State 出；单节点单步，等价于 `START -> node -> END`。
- 实现归属：ROADMAP **Sprint 1**（最小 Trait、Echo Agent 与示例）；后续 Sprint 按上节扩展。

---

## 8. 实现说明

| 项 | 状态 | 说明 |
|----|------|------|
| Agent trait（state-in/state-out） | 已完成 | `rust-langgraph/crates/langgraph/src/traits.rs` |
| Message / AgentError | 已完成 | `message.rs`、`error.rs`（框架内） |
| AgentState / EchoAgent | 已完成 | 由 **Example** 实现：`examples/echo.rs`（非框架内） |
| echo 示例 | 已完成 | `cargo run -p langgraph --example echo -- "你好"` 输出 `你好` |
