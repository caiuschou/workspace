# 最简 ReAct Agent 方案

在 [09-minimal-agent-design](09-minimal-agent-design.md) 的扩展路径 S4 中，ReAct = 推理(Think) + 行动(Act) + 观察(Observe)，配合工具与循环。本文给出**最小可用**的 Rust ReAct agent 设计：先跑通一轮「Think → Act → Observe」，再引入条件边实现循环。

**依赖**：[09-minimal-agent-design](09-minimal-agent-design.md)、[11-state-graph-design](11-state-graph-design.md)、[mcp-integration](mcp-integration/README.md)（工具来源）。

---

## 1. 目标与对应关系

| 要素 | ReAct 通用 | Rust 最简 |
|------|------------|-----------|
| **Think** | LLM 推理，产出自然语言和/或 tool_calls | 节点：读 state.messages，调 LLM（或 mock），写 assistant 消息 + tool_calls |
| **Act** | 执行 tool_calls | 节点：读 tool_calls，通过 ToolSource（如 MCP）执行，产出 tool_results |
| **Observe** | 将 tool_results 写回 state，决定是否继续 | 节点：把 tool_results 写入 state（如追加 messages），返回下一跳或 END |
| **循环** | 有 tool_calls → 回 Think；无 → END | 条件边：observe 后根据「是否还有待执行/待观察」决定 think \| END |
| **工具来源** | Tool registry / MCP | [ToolSource](mcp-integration/README.md) 抽象，默认 MCP |

设计目标：**最简** = 三节点（think, act, observe）+ 扩展 State（messages + tool_calls + tool_results）；先支持「固定一轮工具」或「条件边一轮」，再迭代多轮与流式。

---

## 2. ReAct State 扩展

在现有 `messages: Vec<Message>` 基础上，增加工具相关字段，供 Think/Act/Observe 读写。

### 2.1 最简 ReActState

```text
ReActState {
  messages: Vec<Message>,           // 对话历史（含 User/Assistant/System）
  tool_calls: Vec<ToolCall>,        // 当前轮次 LLM 产出的工具调用（Think 写，Act 读）
  tool_results: Vec<ToolResult>,    // Act 执行结果（Act 写，Observe 合并进 messages 或仅清空）
}
```

- **ToolCall**：至少 `name: String`, `arguments: serde_json::Value`（或 `String`），与 MCP `tools/call` 对齐。
- **ToolResult**：至少 `call_id` 或 `name` + `content: String`，与 MCP `result.content[].text` 对齐。

若希望与现有 `Message` 统一，可扩展 `Message` 为 `AssistantWithCalls { content, tool_calls }` 和 `ToolResult(id, content)`，由 reducer 合并；最简方案可先单独字段，不破坏现有 Echo/Chat 的 `AgentState`。

### 2.2 Message 扩展（可选）

为与 LangChain/LangGraph 对齐，可增加：

- `Message::AssistantWithToolCalls { content: String, tool_calls: Vec<ToolCall> }`
- `Message::ToolResult { call_id: String, content: String }`

最简实现可先只用 `ReActState` 的 `tool_calls` / `tool_results` 两个字段，不扩展 `Message` 枚举，待 S2 再统一。

---

## 3. 最简图结构

### 3.1 节点与边

- **think**：读 `state.messages`，调 LLM（或 mock），写一条 assistant 消息；若有工具调用则写 `state.tool_calls`，否则 `tool_calls` 为空表示本轮结束。
- **act**：读 `state.tool_calls`，对每条调用 `ToolSource::call_tool(name, args)`，写 `state.tool_results`。
- **observe**：读 `state.tool_results`，写回 state（如追加到 messages 或单独字段），并清空/标记已处理；**条件边**：若本轮有 tool_calls 且已观察完 → 下一跳 **think**（继续下一轮），否则 → **END**。

最简**先不做条件边**：固定链 **think → act → observe → END**，即「一轮工具」；实现与测试通过后，再在 `observe` 节点返回下一跳 id，图执行器根据返回值走条件边。

### 3.2 条件边（迭代）

- 图执行器需支持：节点返回 `Next::Node(id)` 或 `Next::End`。
- observe 节点：若 `state.tool_calls` 非空且已把 results 合并进 state → 返回 `Next::Node("think")`，否则 `Next::End`。
- 实现方式见 [11-state-graph-design](11-state-graph-design.md) 扩展路径「条件边」。

---

## 4. 节点职责与 ToolSource

| 节点 | 输入 | 输出 | 依赖 |
|------|------|------|------|
| **think** | state.messages | 新 assistant 消息 + state.tool_calls（可选） | LLM 客户端或 MockLLM；可选 ToolSource::list_tools() 拼 prompt |
| **act** | state.tool_calls | state.tool_results | ToolSource（如 McpToolSource） |
| **observe** | state.tool_results | 更新 state（写回 messages 或内部字段），清空 tool_calls/tool_results；若支持条件边则返回下一跳 | 无 |

工具来源统一为 [ToolSource](mcp-integration/README.md)：think 用 `list_tools()` 拼进 prompt；act 用 `call_tool(name, args)` 执行。不依赖自建 Tool trait，MCP 作为默认实现。

---

## 5. 最简实现范围

1. **State**：定义 `ReActState`（messages + tool_calls + tool_results）及 `ToolCall` / `ToolResult` 类型。
2. **节点**：实现三个节点 ThinkNode、ActNode、ObserveNode，均实现 `Node<ReActState>`；ActNode 持有 `Box<dyn ToolSource>`。
3. **图**：线性链 think → act → observe，用现有 `StateGraph::add_node` / `add_edge` / `compile` / `invoke`。
4. **LLM**：先用 **MockLLM**（如固定返回「调用 get_time 工具」），保证图与 ToolSource 调用路径跑通；再接入真实 LLM。
5. **工具**：先用 mock ToolSource（返回固定工具列表、固定 call 结果），再接 MCP 客户端。
6. **条件边与多轮**：第二阶段；先验收「单轮 Think → Act → Observe → END」。

---

## 6. 与现有设计的关系

- **Agent / StateGraph**：不变；ReAct 是「多节点图」的一种形态，节点可为 `Node<ReActState>`，无需新 trait。
- **记忆**：仍为调用方持有 state；多次 `invoke(state)` 之间由调用方传入同一 state，messages 即历史。
- **MCP**：工具发现与调用通过 [mcp-integration](mcp-integration/README.md) 的 ToolSource 完成，ReAct 只依赖 ToolSource 接口。

---

## 7. 小结与任务表

- **最简 ReAct** = ReActState（messages + tool_calls + tool_results）+ 三节点（think, act, observe）+ 线性链一轮 + ToolSource（MCP 或 mock）。
- **迭代**：条件边 → 多轮循环；扩展 Message 统一 tool_calls/tool_results；流式输出。

| 项 | 状态 | 说明 |
|----|------|------|
| ReActState / ToolCall / ToolResult | 待实现 | 新类型，可放在 `state/react_state.rs` 或等价模块 |
| ThinkNode | 待实现 | 依赖 LLM（先 Mock），写 messages + tool_calls |
| ActNode | 待实现 | 依赖 ToolSource，读 tool_calls 写 tool_results |
| ObserveNode | 待实现 | 写回 messages 或合并结果，清空 tool_* |
| 线性链 think→act→observe | 待实现 | 使用现有 StateGraph，example 跑通一轮 |
| 条件边 observe→think \| END | 待实现 | 见 11-state-graph-design 扩展 |
| MockLLM / Mock ToolSource | 待实现 | 用于示例与测试 |

实现归属：在 `langgraph` 或独立 example 中实现 ReActState 与三节点；任务规划与验收见下文开发计划。

---

## 8. 开发计划（细致表格）

开发按阶段推进，每项完成后将「状态」改为「已完成」，并在对应代码处补充注释引用本文（如 `13-react-agent-design.md`）。

### 8.1 阶段一：State 与类型定义

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 1.1 | ReActState 定义 | `struct ReActState { messages, tool_calls, tool_results }`，满足 `Clone + Send + Sync + 'static` | 已完成 | 复用现有 `Message`；新字段见 §2.1 |
| 1.2 | ToolCall 类型 | `name: String`, `arguments: serde_json::Value`（或 `String`），与 MCP `tools/call` 对齐 | 已完成 | 可选：`id` 与 tool_result 对应 |
| 1.3 | ToolResult 类型 | `call_id` 或 `name` + `content: String`，与 MCP `result.content[].text` 对齐 | 已完成 | 与 ToolCall 一一对应便于 Observe 合并 |
| 1.4 | 模块与导出 | 新模块如 `state/react_state.rs` 或 `react/state.rs`，在 `lib.rs` 中导出 | 已完成 | 与现有 `message.rs`、`graph/` 并列 |
| 1.5 | 单元测试 | 构造 ReActState / ToolCall / ToolResult，序列化或合并逻辑（若有） | 已完成 | 无外部依赖 |

### 8.2 阶段二：Mock 与基础设施

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 2.1 | MockLLM 抽象 | 定义「可调用、返回 assistant 文本 + 可选 tool_calls」的接口（trait 或闭包） | 待实现 | ThinkNode 依赖；先不接真实 LLM |
| 2.2 | MockLLM 实现 | 固定返回一条 assistant 消息 + 固定一条 ToolCall（如 `get_time`），用于跑通图 | 待实现 | 可配置「无 tool_calls」以测 END 路径 |
| 2.3 | Mock ToolSource | 实现 `ToolSource`：`list_tools()` 返回固定工具列表，`call_tool(name, args)` 返回固定文本 | 待实现 | 见 [mcp-integration](mcp-integration/README.md)；可先于 McpToolSource 存在 |
| 2.4 | Mock 单元测试 | MockLLM / Mock ToolSource 行为符合预期，Act 可调 call_tool 并拿到结果 | 待实现 | 无 MCP Server |

### 8.3 阶段三：三节点实现

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 3.1 | ThinkNode 结构 | 持有 MockLLM（或 `Box<dyn LlmClient>`），实现 `Node<ReActState>`，id 如 `"think"` | 待实现 | 依赖 1.x, 2.1, 2.2 |
| 3.2 | ThinkNode::run | 读 `state.messages`，调 LLM，写一条 assistant 消息 + 可选 `state.tool_calls`；无工具时 tool_calls 为空 | 待实现 | 先不接 ToolSource::list_tools，prompt 可写死 |
| 3.3 | ActNode 结构 | 持有 `Box<dyn ToolSource>`，实现 `Node<ReActState>`，id 如 `"act"` | 待实现 | 依赖 1.x, 2.3；ToolSource 见 mcp-integration |
| 3.4 | ActNode::run | 读 `state.tool_calls`，对每条 `call_tool(name, args)`，写 `state.tool_results`；无 tool_calls 时 results 为空 | 待实现 | 错误处理：单条失败是否短路整图可配置 |
| 3.5 | ObserveNode 结构 | 无外部依赖，实现 `Node<ReActState>`，id 如 `"observe"` | 待实现 | 依赖 1.x |
| 3.6 | ObserveNode::run | 读 `state.tool_results`，写回 state（追加到 messages 或内部字段），清空 `tool_calls` / `tool_results`；返回完整 state | 待实现 | 线性链阶段不返回下一跳，仅更新 state |
| 3.7 | 节点单元测试 | 各节点单独喂入 ReActState，断言输出 state 形状与内容 | 待实现 | 使用 MockLLM / Mock ToolSource |

### 8.4 阶段四：线性链与示例

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 4.1 | 构建图 | `StateGraph::<ReActState>::new().add_node("think", ...).add_node("act", ...).add_node("observe", ...).add_edge("think").add_edge("act").add_edge("observe")` | 待实现 | 依赖 3.x；使用现有 StateGraph API |
| 4.2 | compile + invoke | `graph.compile()?` 得到 `CompiledStateGraph`，`invoke(initial_state).await?` 得到最终 state | 待实现 | 与 11-state-graph-design 一致 |
| 4.3 | Example 程序 | 如 `examples/react_linear.rs`：构造初始 state（一条 User 消息），invoke，打印最终 messages 或最后一条 assistant | 待实现 | 可配置 MockLLM 返回「调用 get_time」 |
| 4.4 | 集成测试 | 从 User 输入到得到 tool_results 写回 messages 的整条链路；CI 可跑 | 待实现 | 不依赖真实 LLM / MCP Server |
| 4.5 | 文档与注释 | README 或本文 §5 引用 example 运行方式；代码内注释引用 13-react-agent-design | 待实现 | 便于后续接手 |

### 8.5 阶段五：条件边与多轮（迭代）

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 5.1 | Next 类型 | 定义 `enum Next { Node(String), End }`，或与现有图 API 统一（如节点返回 `Option<String>` 表示下一跳） | 待实现 | 见 11-state-graph-design 扩展「条件边」 |
| 5.2 | Node 返回下一跳 | 节点 `run` 返回 `Result<(S, Next), AgentError>` 或图执行器从 state 中读取「下一跳」字段 | 待实现 | 需扩展 Node trait 或 StateGraph 约定 |
| 5.3 | 图执行器条件边 | `invoke` 循环中根据节点返回的 Next 选择下一节点或结束 | 待实现 | 依赖 5.1, 5.2 |
| 5.4 | ObserveNode 返回 Next | 若本轮有 tool_calls 且已观察完 → `Next::Node("think")`，否则 `Next::End` | 待实现 | 依赖 3.6 与 5.1 |
| 5.5 | 多轮示例与测试 | 示例中 MockLLM 首轮返回 tool_calls，Observe 后回到 think，二轮返回无 tool_calls 结束；断言两轮消息与 results | 待实现 | 验收多轮循环 |

### 8.6 阶段六：可选扩展

| 序号 | 任务 | 交付物 / 子项 | 状态 | 依赖与说明 |
|------|------|----------------|------|------------|
| 6.1 | Think 接 ToolSource::list_tools | ThinkNode 构建 prompt 时调用 `list_tools()`，将工具描述拼进 system/user 消息 | 待实现 | 依赖 mcp-integration ToolSource |
| 6.2 | Message 扩展 | `Message::AssistantWithToolCalls { content, tool_calls }`、`Message::ToolResult { call_id, content }`；ReActState 可选沿用或迁移到 messages 内 | 待实现 | 与 LangChain/LangGraph 对齐，见 §2.2 |
| 6.3 | 真实 LLM 接入 | 替换 MockLLM 为真实 API 客户端，解析返回中的 tool_calls（如 OpenAI JSON mode） | 待实现 | 非最简范围，按需排期 |
| 6.4 | McpToolSource 接入 | ReAct 示例配置为使用 McpToolSource，对接真实或 mock MCP Server | 待实现 | 依赖 mcp-integration 实现；见 mcp-integration/implementation.md 任务表 |

---

**表使用说明**：按阶段顺序执行；阶段五依赖阶段四，阶段六可与阶段四、五并行或后续迭代。每项完成后在「状态」列改为「已完成」。
