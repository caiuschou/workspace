# 最简单的 Rust 版 Agent 与 StateGraph 设计

参考 LangGraph，给出**最小可用**的 Rust agent 与图编排实现，先跑通再迭代。

---

## 1. 对应关系与目标

| 要素 | LangGraph | Rust 最简 |
|------|-----------|-----------|
| **State** | `TypedDict`（如 `messages`） | 调用方持有；`invoke(state)` 入、返回 state 出 |
| **Node** | `(state) -> partial state` | 可执行单元：接收状态、返回状态（完整或 partial） |
| **Edge** | `START -> node -> END`（或多节点链） | 单节点：一次 `run(state)`；多节点：StateGraph 按边顺序执行 |
| **记忆** | Checkpointer / 调用方持 state | 调用方在多次 invoke 间持有同一 state，`messages` 即会话历史 |

设计目标：**仅 State，无 Input/Output**；先 trait 抽象单节点（Agent），再引入最简图（StateGraph 线性链）；最小类型（Message、AgentError）。

---

## 2. 最简 Agent

### 2.1 Trait 与类型

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    type State: Clone + Send + Sync + 'static;
    /// One step: state in, state out.
    async fn run(&self, state: Self::State) -> Result<Self::State, AgentError>;
}
```

框架提供 `Message`（System/User/Assistant）、`AgentError`。State 由实现方定义（例：`AgentState { messages: Vec<Message> }` 在 example 中，见 `examples/echo.rs`）。

### 2.2 Echo Agent 示例

```rust
struct EchoAgent;
type State = AgentState; // { messages: Vec<Message> }

#[async_trait]
impl Agent for EchoAgent {
    fn name(&self) -> &str { "echo" }
    type State = AgentState;
    async fn run(&self, state: Self::State) -> Result<Self::State, AgentError> {
        let mut messages = state.messages;
        if let Some(Message::User(s)) = messages.last() {
            messages.push(Message::Assistant(s.clone()));
        }
        Ok(AgentState { messages })
    }
}
```

使用：`state.messages.push(Message::User(...)); state = agent.run(state).await?;` 调用方在多次 `run` 间持有同一 `state`，即最简单记忆（无持久化）。详见 §4。

---

## 3. StateGraph 最简方案

在单节点 Agent 之上，引入**图**：多节点 + 边。最简形态 = **线性链**，无条件边、无环。

| 概念 | LangGraph | Rust 最简 |
|------|-----------|-----------|
| 图 | `StateGraph(State).add_node().add_edge()` | `StateGraph<S>`：节点列表 + 边序（如 `[n1, n2]`） |
| 运行 | `compile().invoke(state)` | `invoke(state)` 从 START 按边序执行节点，state 依次传递 |
| 节点产出 | partial state，按 reducer 合并 | 每节点 `run(state) -> State`；可约定返回完整 state 或 partial，partial 时由运行时 `state.merge(partial)`（见 [10-reducer-design](10-reducer-design.md)） |

**最简 StateGraph**：

- **结构**：`StateGraph<S>` 持有一组命名节点（如 `Vec<(String, Box<dyn Node<S>>)`）和线性边序 `[id_1, id_2, ...]`，表示 `START -> id_1 -> id_2 -> ... -> END`。
- **执行**：`invoke(state)` 循环 `for id in &edge_order { state = nodes[id].run(state)?; }` 返回最终 state。
- **与 Agent 关系**：单节点时等价于 Agent；多节点时图负责编排。暂不引入条件边、循环、Checkpointer。

完整设计（节点抽象、编译、扩展路径）见 [11-state-graph-design](11-state-graph-design.md)；条件边、partial + reducer、Checkpointer 见 ROADMAP 与 [10-reducer-design](10-reducer-design.md)。

---

## 4. 最简单的记忆

不引入 Memory trait：**记忆 = 调用方持有的 State**。每次 `run(state)` 或 `graph.invoke(state)` 传入当前 state，拿回更新后的 state，下一轮再传入；`state.messages` 即会话历史。无持久化、无容量管理；持久化/跨会话见 S3、S5 与 `MEMORY_VS_LANGGRAPH_STORE.md`。

---

## 5. 扩展路径

1. **S1**：EchoAgent，无 Input/Output。
2. **S2**：ChatAgent，`run(state)` 内调 LLM、追加回复。
3. **S3**：流式/会话记忆（State 不变）。
4. **S4**：ReAct，扩展 State、工具与循环；StateGraph 多节点 + 条件边。
5. **S5**：工具生态、记忆扩展。

每一步保持「当前可运行的最简形态」。

---

## 6. 小结与实现说明

- **最简 Rust** = trait `Agent`（state 进/出）+ Message/AgentError + EchoAgent；**记忆** = 调用方持 state；**StateGraph 最简** = 线性链、多节点顺序执行、state 传递。
- 实现归属：Sprint 1（Agent、Echo）；StateGraph 线性链可在 S2/S3 引入；reducer/partial 见 [10-reducer-design](10-reducer-design.md)。

| 项 | 状态 | 说明 |
|----|------|------|
| Agent trait | 已完成 | `langgraph/src/traits.rs` |
| Message / AgentError | 已完成 | `message.rs`、`error.rs` |
| AgentState / EchoAgent | 已完成 | Example：`examples/echo.rs` |
| echo 示例 | 已完成 | `cargo run -p langgraph --example echo -- "你好"` |
| StateGraph 线性链 | 待实现 | 多节点编排，见 §3 |
