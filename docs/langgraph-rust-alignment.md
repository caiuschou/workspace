# langgraph-rust 与 Python LangGraph 对齐说明

## 定位

- **langgraph-rust**：LangGraph 的 Rust 实现，仓库 <https://github.com/caiuschou/langgraph-rust>。
- **源代码位置**：本仓库内 **`thirdparty/langgraph-rust`**（由 `git clone` 放入，不提交，见 .gitignore）。实现与设计对照时在该路径查看 StateGraph、Node、invoke 等实现。
- 本仓库通过示例与 Agora 设计与之对接。

## 图 / 状态 / 边：是否一致

### 设计意图：一致

- 不改变 **State + Node + StateGraph.invoke** 的模型；消息能力通过 State 形状与运行时约定实现。
- 节点接口：节点读 state、返回更新与下一步（见下「源代码对照」中 Node 签名）。
- 执行边界：`invoke` 前写 state、`invoke` 后读 state，由外部运行时负责投递与存储。

### Python 侧要点（对照用）

| 要素 | 说明 |
|------|------|
| State | 应用状态，可有带 reducer 的 channel（如 `messages` + `add_messages`） |
| Node | 函数 `(state) -> state_updates` |
| Edge | `add_edge(START, "node")`、`add_edge("node", END)`、`add_conditional_edges(...)` |
| 执行 | `invoke(input, config)`，config 中常用 `thread_id` |

### thirdparty/langgraph-rust 源代码对照（已确认）

以下基于 `thirdparty/langgraph-rust` 中 `langgraph/src` 阅读结果。

| 要素 | Python 侧 | langgraph-rust 实现 | 是否一致 |
|------|------------|----------------------|----------|
| **State** | 应用状态，可有 reducer/channel | 图泛型 `S: Clone + Send + Sync + 'static`，由调用方定义（如 `AgentState`、`ReActState`）；无内置 reducer，等价于「应用状态」 | ✅ 一致 |
| **StateGraph** | 图 = 节点 + 边 | `StateGraph::<S>::new()`，`add_node(id, Box<dyn Node<S>>)`，`add_edge(to_id)` 顺序构成链（首节点即入口，末节点后为 END），`compile()` / `compile_with_checkpointer()` → `CompiledStateGraph<S>` | ✅ 一致 |
| **Node** | `(state) -> state_updates` | `Node::run(&self, state: S) -> Result<(S, Next), AgentError>`：返回**新 state** 与 **Next**（`Continue` / `Node(id)` / `End`），用于条件边与结束。`Agent` 作为节点时通过 blanket impl 返回 `(state, Next::Continue)` | ✅ 一致（返回 updates + 路由） |
| **Edge** | `add_edge( from, to )`、`add_conditional_edges` | 线性边：`add_edge("id")` 顺序即 START→…→END。条件边：由节点 `run` 返回 `Next::Node(id)` 或 `Next::End`，由 `CompiledStateGraph::invoke` 根据 `Next` 选择下一节点或结束 | ✅ 一致 |
| **invoke** | `invoke(input, config)` | `CompiledStateGraph::invoke(state: S, config: Option<RunnableConfig>) -> Result<S, AgentError>`；调用方 invoke 前写入 state，invoke 后从返回值读取 state | ✅ 一致 |
| **config / thread_id** | config 中常用 `thread_id` | `RunnableConfig { thread_id, checkpoint_id, checkpoint_ns, user_id }`，与 `compile_with_checkpointer` 配合时 invoke 后按 `thread_id` 持久化 checkpoint | ✅ 一致 |

### 实现细节

- **边类型**：条件边通过 `Next::Node(id)` / `Next::End` 实现，与 Python 的 conditional edges 对应；线性边由 `add_edge` 顺序定义。
- **State 的 reducer/channel**：核心未内置 reducer；若需「channel + reducer」语义，由应用在 state 类型内自行实现（如 `ReActState` 的 messages/tool_calls/tool_results）。
- **Checkpoint/thread**：已实现；`Checkpointer`、`RunnableConfig::thread_id`、`invoke` 后保存，见 `memory/` 与 `compiled.rs`。

## 实现与设计对照（本仓库）

| 对照项 | 设计意图 | 本仓库现状 | 是否一致 |
|--------|----------|------------|----------|
| **图 / StateGraph** | State + Node + StateGraph.invoke，节点返回 state + 路由 | **thirdparty/langgraph-rust** 已实现：`StateGraph`、`Node::run(state) -> (S, Next)`、`CompiledStateGraph::invoke`；见上「源代码对照」 | ✅ 一致 |
| **invoke 边界** | invoke 前写 state、invoke 后读 state，由外部运行时负责 | **thirdparty/langgraph-rust** 中 `invoke(state, config)` 入参为 state、返回值为 state，边界与设计一致 | ✅ 一致 |
| **Agora 消息类型** | [langgraph-agent-messaging-design](../agora/docs/langgraph-agent-messaging-design.md) 要求：agora-protocol 定义 AgentId、RequestId、EnvelopeBody、AgentEnvelope | `agora-protocol` 仅定义 `AgentRecord`，**未**定义 AgentId、RequestId、EnvelopeBody、AgentEnvelope | **不一致** |
| **MessagingState / 助理图** | State 含 MessagingState（inbound/outbound），助理图与邮件 Agent 图示例 | 任务表为「未开始」，无 MessagingState、无示例图实现 | **不一致** |

**结论**：

- **设计意图**（文档中的图/状态/边、invoke 边界、消息能力约定）与 Python LangGraph 对齐描述一致；**thirdparty/langgraph-rust** 源代码已阅读，图/状态/边/invoke/config（含 thread_id、Checkpoint）与设计及 Python 侧要点一致。
- **本仓库**：图实现位于 **thirdparty/langgraph-rust**（不提交）；Agora 协议层尚未按 [langgraph-agent-messaging-design](../agora/docs/langgraph-agent-messaging-design.md) 实现信封与 MessagingState，故 Agora 侧实现与设计仍不一致。

## 相关文档

- [agora/docs/langgraph-agent-messaging-design.md](../agora/docs/langgraph-agent-messaging-design.md) — 在 langgraph-rust 上做消息能力的约定。
- [docs/langgraph-agent/README.md](langgraph-agent/README.md) — Python LangGraph 概念与用法。

## 更新记录与任务

| 日期 | 更新内容 |
|------|----------|
| 2025-01-31 | 将 langgraph-rust 克隆至 thirdparty/langgraph-rust；阅读源码，新增「源代码对照」、修正节点接口描述、更新「实现与设计对照」结论。 |

| 序号 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 1 | Agora 在 agora-protocol 中定义 AgentId、RequestId、EnvelopeBody、AgentEnvelope | 未开始 | 见 [langgraph-agent-messaging-design](../agora/docs/langgraph-agent-messaging-design.md) 任务表 |
| 2 | 本表「实现与设计对照」中 Agora 消息类型 / MessagingState 改为已实现 | 待 Agora 任务完成后 | 完成上述任务后在此标记并更新结论 |
