# StateGraph 设计方案

与 LangGraph 的 `StateGraph` 对齐：多节点 + 边，`invoke(state)` 按图执行。本文给出 Rust 侧的最简实现（线性链）与扩展路径。

**文档结构**：§1 对应关系与目标 → §2 节点抽象 → §3 StateGraph 结构 → §4 线性链与执行 → §5 使用方式与示例 → §6 扩展路径 → §7 小结与实现说明。

**依赖**：[09-minimal-agent-design](09-minimal-agent-design.md)（Agent / State）、[10-reducer-design](10-reducer-design.md)（partial state 与 merge）。

---

## 1. 对应关系与目标

| 概念 | LangGraph | Rust 方案 |
|------|-----------|-----------|
| **图** | `StateGraph(State)` | `StateGraph<S>`：泛型状态类型 `S` |
| **节点** | `add_node(name, fn)`，`fn(state) -> partial` | 命名节点：`add_node(id, Node<S>)`，节点 `run(state) -> S` 或 partial |
| **边** | `add_edge(START, "n1")`, `add_edge("n1", "n2")`, `add_edge("n2", END)` | 最简：线性边序 `[n1, n2, ...]` 表示 START→n1→n2→…→END |
| **编译** | `graph.compile()` | `graph.compile()` 得到可执行图（校验边、固定执行顺序） |
| **执行** | `compiled.invoke(state)` | `invoke(state)` 按边序执行节点，state 依次传递 |

设计目标：

- **与 Agent 兼容**：单节点图等价于一次 `Agent::run`；多节点时图编排多个「节点」。
- **先线性链**：仅支持 START→n1→…→nk→END，无条件边、无环；后续再加条件边与循环。
- **State 一致**：整图一个状态类型 `S`；节点返回完整 `S` 或 partial，partial 时由运行时 `merge`（见 [10-reducer-design](10-reducer-design.md)）。

---

## 2. 节点抽象

图上的「节点」需要统一接口：接收 state、返回 state（完整或 partial）。与 [09-minimal-agent-design](09-minimal-agent-design.md) 中的 Agent 对齐：

- **方案 A**：节点即 `dyn Agent`，但 `Agent` 的 `State` 是关联类型，不同 Agent 可有不同 State，图需要**单一** state 类型，故不能直接把 `Agent` 当图节点用，除非约束「所有节点共享同一 State 类型」。
- **方案 B**：单独定义 **Node trait**，泛型为 state 类型 `S`，图只接受 `Node<S>`。实现上可将 `Agent` 适配为 `Node<S>`（当 `Agent::State == S` 时）。

推荐 **方案 B**：图用 `Node<S>`，Agent 通过适配器作为节点。

```rust
/// Node: one step in a graph. Receives state S, returns updated S (full or partial).
/// Used by StateGraph to run a single step. Aligns with LangGraph node (state) -> partial.
#[async_trait]
pub trait Node<S>: Send + Sync
where
    S: Clone + Send + Sync + 'static,
{
    /// Node id (e.g. "chat", "tool").
    fn id(&self) -> &str;

    /// One step: state in, state out. Graph runner passes result to next node or returns.
    async fn run(&self, state: S) -> Result<S, AgentError>;
}
```

**与 Agent 的关系**：当图中所有节点共享同一 `S` 时，可为 `Agent` 实现 `Node<S>`（`where Agent::State = S`），这样现有 EchoAgent、ChatAgent 可直接作为图的节点。Rust 中可用 blanket impl 约束 `A::State = S`，或先在 example 中手写 `impl Node<AgentState> for EchoAgent`。

---

## 3. StateGraph 结构

- **状态类型**：泛型 `S`，满足 `Clone + Send + Sync + 'static`。
- **节点存储**：`HashMap<String, Box<dyn Node<S>>>` 或 `Vec<(String, Box<dyn Node<S>>)>`，key 为节点 id。
- **边**：最简形态下仅需**线性链**，用 `Vec<String>` 表示顺序，如 `["chat", "tool"]` 表示 START→chat→tool→END。

```rust
/// State graph: nodes + linear edge order. No conditional edges in minimal version.
pub struct StateGraph<S> {
    nodes: HashMap<String, Box<dyn Node<S>>>,
    /// Linear chain: [id1, id2, ...] => START -> id1 -> id2 -> ... -> END
    edge_order: Vec<String>,
}

impl<S> StateGraph<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self { ... }

    /// Adds a node; id must be unique. Replaces if same id.
    pub fn add_node(&mut self, id: impl Into<String>, node: Box<dyn Node<S>>) -> &mut Self { ... }

    /// Appends an edge from current chain end to this node. Order of add_edge defines the chain.
    /// Minimal: no START/END in API; first add_edge is from START, last leads to END.
    pub fn add_edge(&mut self, to_id: impl Into<String>) -> &mut Self { ... }

    /// Builds executable graph: validates all edge targets are registered nodes.
    pub fn compile(self) -> Result<CompiledStateGraph<S>, CompilationError> { ... }
}
```

**编译**：`compile()` 检查 `edge_order` 中每个 id 都在 `nodes` 中，然后返回 `CompiledStateGraph<S>`，持有节点与边序，不可再修改。

---

## 4. 线性链与执行

**CompiledStateGraph**：只读的图，提供 `invoke`。

```rust
/// Compiled graph: immutable structure, supports invoke only.
pub struct CompiledStateGraph<S> {
    nodes: HashMap<String, Box<dyn Node<S>>>,
    edge_order: Vec<String>,
}

impl<S> CompiledStateGraph<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Runs START -> n1 -> n2 -> ... -> END with given state. Returns final state.
    pub async fn invoke(&self, state: S) -> Result<S, AgentError> {
        let mut state = state;
        for id in &self.edge_order {
            let node = self.nodes.get(id).expect("compiled graph has all nodes");
            state = node.run(state).await?;
        }
        Ok(state)
    }
}
```

- **执行语义**：从 START 开始，按 `edge_order` 依次执行节点；每节点 `state = node.run(state)?`；最后返回 END 时的 state。
- **节点返回**：当前最简约定为**完整 state**；若后续支持 partial，则节点返回 `S` 的「更新部分」，由运行时调用 `state.merge(partial)`（需 `S: Merge` 或每字段 reducer，见 [10-reducer-design](10-reducer-design.md)），本阶段可只做完整 state 传递。

---

## 5. 使用方式与示例

**构建图**：

```rust
let mut graph = StateGraph::<AgentState>::new();
graph
    .add_node("echo", Box::new(EchoAgent::new()))
    .add_edge("echo");
// 单节点链：START -> echo -> END

let compiled = graph.compile().expect("valid graph");
let state = AgentState { messages: vec![Message::User("hi".into())] };
let state = compiled.invoke(state).await?;
```

**多节点**（例如 chat → tool）：

```rust
graph
    .add_node("chat", Box::new(ChatAgent::new(llm)))
    .add_node("tool", Box::new(ToolNode::new(tools)))
    .add_edge("chat")
    .add_edge("tool");
let compiled = graph.compile()?;
let state = compiled.invoke(initial_state).await?;
```

**与记忆**：调用方在多次 `compiled.invoke(state)` 之间持有同一 `state`，即 [09-minimal-agent-design](09-minimal-agent-design.md) 中的「最简单记忆」；图内不持状态。

---

## 6. 扩展路径

| 阶段 | 内容 |
|------|------|
| **当前** | 线性链、节点返回完整 state、无 Checkpointer |
| **条件边** | 节点返回下一跳 id（或 END）；`invoke` 中根据返回值选择下一节点，支持分支与循环 |
| **Partial state** | 节点返回 `S::Partial` 或 `AgentStateUpdate`，运行时 `state.merge(partial)`，需 State 实现 merge/reducer（见 [10-reducer-design](10-reducer-design.md)） |
| **Checkpointer** | 每步或每节点后持久化 state，支持 thread_id、时间旅行、interrupt/resume（见 ROADMAP、MEMORY_VS_LANGGRAPH_STORE） |

实现顺序建议：先完成线性链 + 完整 state，再考虑 partial + merge，最后条件边与 Checkpointer。

---

## 7. 小结与实现说明

- **StateGraph** = 节点表 + 线性边序；`compile()` 得到 `CompiledStateGraph`；`invoke(state)` 按边序执行节点，state 依次传递。
- **Node<S>** = 图节点抽象，可与 Agent 通过适配器复用；单节点图等价于一次 Agent::run。
- **最简**：仅线性链、完整 state；扩展为条件边、partial + merge、Checkpointer。

| 项 | 状态 | 说明 |
|----|------|------|
| Node trait | 待实现 | `langgraph/src/graph/node.rs` 或 `graph/mod.rs` |
| StateGraph<S> | 待实现 | `add_node` / `add_edge` / `compile` |
| CompiledStateGraph<S> | 待实现 | `invoke(state)` |
| Agent as Node<S> | 待实现 | 适配器或 blanket impl（同 State 时） |
| 线性链示例 | 待实现 | `examples/state_graph_echo.rs` 或扩展现有 echo |
