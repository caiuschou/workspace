# LLM + ReAct Agent 方案

## 目标

在 rust-langgraph 中提供 **「LLM + ReAct Agent」** 的组合方案：**真实 LLM（ChatOpenAI / ChatZhipu）** 接入 **ReAct 图**（Think → Act → Observe），完成「消息进 → 推理 → 工具调用 → 观察 → 助手消息出」的闭环。本方案不新增核心抽象，仅明确**如何将已有 LlmClient 与 ReAct 图组合**，以及可选扩展（工具绑定、多轮循环）。

## 依赖

- [09-minimal-agent-design](09-minimal-agent-design.md)：最简 Agent / State 设计
- [11-state-graph-design](11-state-graph-design.md)：StateGraph、Node、invoke、Next
- [13-react-agent-design](13-react-agent-design.md)：ReAct 三节点、ReActState、ToolSource
- [14-chat-openai](14-chat-openai.md)：ChatOpenAI、LlmClient、with_tools
- [mcp-integration](mcp-integration/README.md)：ToolSource 抽象（工具发现与调用）

## 现状

以下能力**已实现**，可直接组合：

| 组件 | 说明 | 代码位置 |
|------|------|----------|
| **LlmClient** | `invoke(messages) -> LlmResponse`（content + tool_calls） | `llm/mod.rs` |
| **ChatOpenAI** | 实现 LlmClient，调 OpenAI Chat Completions；`with_tools(tools)` | `llm/openai.rs`，feature `zhipu` |
| **ChatZhipu** | 实现 LlmClient，智谱 GLM（OpenAI 兼容） | `llm/zhipu.rs`，feature `zhipu` |
| **MockLlm** | 固定响应、可配置 tool_calls，用于测试 | `llm/mock.rs` |
| **ThinkNode** | 持 `Box<dyn LlmClient>`，读 messages、调 `invoke`、写 assistant + tool_calls | `react/think_node.rs` |
| **ActNode** | 持 `Box<dyn ToolSource>`，读 tool_calls、`call_tool`、写 tool_results | `react/act_node.rs` |
| **ObserveNode** | 读 tool_results、写回 messages、清空 tool_*；线性 `new()` 或多轮 `with_loop()` | `react/observe_node.rs` |
| **ReActState** | messages + tool_calls + tool_results | `state/react_state.rs` |
| **StateGraph** | add_node / add_edge / compile / invoke | `graph/` |
| **ToolSource** | `list_tools()`、`call_tool(name, args)`；MockToolSource 已实现 | `tool_source/` |

当前 **react_linear** 示例使用 **MockLlm + MockToolSource**；**chat-openai** 示例仅用 **ChatOpenAI**，无 ReAct。本方案即：**用 ChatOpenAI（或 ChatZhipu）替换 MockLlm**，接入 ReAct 图，并可选绑定工具、多轮循环。

## 数据流

```
User 消息（+ 可选 System / 历史）
    ↓
ThinkNode：LlmClient::invoke(messages) → content + tool_calls
    ↓
写 assistant 消息、state.tool_calls
    ↓
ActNode：对每条 tool_calls 调用 ToolSource::call_tool → tool_results
    ↓
ObserveNode：tool_results 写回 messages（如 User 形式），清空 tool_calls / tool_results
    ↓
Next::Continue（线性）→ 下一节点；Next::Node("think")（多轮）→ 回 Think；Next::End → 结束
```

- **线性链**：think → act → observe → END，一轮工具即结束。
- **多轮**：observe 后若有 tool_calls 且已观察完 → 回到 think；否则 END。使用 `ObserveNode::with_loop()`。

## 方案

### 1. 最小组合（必做）

- **ThinkNode** 持 `Box<dyn LlmClient>`：用 **ChatOpenAI::new(model)** 或 **ChatZhipu** 替换 MockLlm。
- **ActNode** 持 `Box<dyn ToolSource>`：先用 **MockToolSource::get_time_example()**（或自定义 Mock），保证链路跑通。
- **图**：线性链 **think → act → observe**，`ObserveNode::new()`。
- **入口**：新增示例 **react_openai**（或扩展现有示例），从 args 读用户输入，构造 `ReActState { messages, .. }`，invoke，打印最终 messages。
- **构建**：需 `--features zhipu`，环境变量 `OPENAI_API_KEY`（或智谱 `ZHIPU_API_KEY` 等）。

无需改 ThinkNode / ActNode / StateGraph 实现；只替换 LLM 与 ToolSource 的具体类型，并加一个可运行的示例。

### 2. 带工具调用（推荐）

- **ChatOpenAI::with_tools(tools)**：tools 来自 **ToolSource::list_tools()**（如 MockToolSource 或 McpToolSource）。
- **ThinkNode** 仍只调 `LlmClient::invoke`；工具列表由**构造 ChatOpenAI 时**传入，不需 ThinkNode 感知 ToolSource。
- 流程：先 `tool_source.list_tools().await?` → `ChatOpenAI::new(model).with_tools(tools)` → `ThinkNode::new(Box::new(llm))`。

这样模型才会返回 `tool_calls`，Act 才能执行工具。

### 3. 多轮 ReAct（可选）

- 使用 **ObserveNode::with_loop()**，observe 后根据「是否有 tool_calls 且已观察」返回 `Next::Node("think")` 或 `Next::End`。
- 图需支持条件边（已支持，见 11-state-graph-design、13-react-agent-design §8.5）。
- 可选新增 **react_openai_loop** 示例，或在同一示例中通过 flag 切换线性 / 多轮。

### 4. 可选扩展

- **Think 接 ToolSource::list_tools**：由 ThinkNode 在运行时拉取工具列表拼 prompt（13 文档 §8.6 任务 6.1）。当前最简方案不依赖此步，工具仅通过 `with_tools` 绑定即可。
- **McpToolSource**：替换 MockToolSource，对接真实 MCP Server；见 mcp-integration。
- **ChatZhipu**：与 ChatOpenAI 同等用法，换 `ZHIPU_API_KEY` 等配置即可。

## 使用方式

### 最小用法（ChatZhipu 智谱 + MockToolSource，线性）

为演示工具调用，需 `with_tools`；否则模型通常不返回 `tool_calls`，Act 不执行工具。

```rust
use langgraph::{
    ActNode, ChatZhipu, CompiledStateGraph, Message, MockToolSource,
    ObserveNode, ReActState, StateGraph, ThinkNode, ToolSource,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user_input = std::env::args().nth(1).unwrap_or_else(|| "What time is it?".to_string());

    let tool_source = MockToolSource::get_time_example();
    let tools = tool_source.list_tools().await?;
    let llm = ChatZhipu::new("glm-4-flash").with_tools(tools);
    let think = ThinkNode::new(Box::new(llm));
    let act = ActNode::new(Box::new(tool_source));
    let observe = ObserveNode::new();

    let mut graph = StateGraph::<ReActState>::new();
    graph
        .add_node("think", Box::new(think))
        .add_node("act", Box::new(act))
        .add_node("observe", Box::new(observe))
        .add_edge("think")
        .add_edge("act")
        .add_edge("observe");

    let compiled = graph.compile()?;
    let state = ReActState {
        messages: vec![Message::user(user_input)],
        tool_calls: vec![],
        tool_results: vec![],
    };

    let final_state = compiled.invoke(state).await?;
    for m in &final_state.messages {
        println!("{:?}", m);
    }
    Ok(())
}
```

需 `cargo run -p langgraph --example react_zhipu --features zhipu -- "What time is it?"`，且 `ZHIPU_API_KEY` 已设置。

### 带工具绑定（with_tools）

最小用法示例已包含 `with_tools`。若 ToolSource 为 McpToolSource 等，同样先 `list_tools().await?`，再 `ChatZhipu::with_tools(tools)`（或 ChatOpenAI），Act 使用同一 ToolSource。这样 LLM 才知道可调用工具，才会返回 `tool_calls`。

### 多轮循环

将 `ObserveNode::new()` 换为 `ObserveNode::with_loop()`，并确保图支持条件边（observe → think 或 END）。其余与线性链相同。

## 任务表

按 [AGENTS.md](../../AGENTS.md) 要求，任务规划到下表；完成一项即将「状态」改为「已完成」。

| 序号 | 任务 | 交付物 / 子项 | 状态 | 说明 |
|------|------|----------------|------|------|
| 1 | 方案文档 | 本文档 15-llm-react-agent.md | 已完成 | 目标、现状、数据流、方案、使用方式、任务表 |
| 2 | 更新 README 文档目录 | docs/rust-langgraph/README.md 增加 15-llm-react-agent 链接 | 已完成 | 与 13、14 并列 |
| 3 | 最小示例 react_zhipu | 示例：ThinkNode(ChatZhipu) + ActNode(MockToolSource) + ObserveNode::new()，线性链 | 已完成 | `cargo run -p langgraph --example react_zhipu --features zhipu -- "What time is it?"`；需 ZHIPU_API_KEY |
| 4 | 示例带 with_tools | react_zhipu 中 ToolSource::list_tools → ChatZhipu::with_tools，Act 用同源 ToolSource | 已完成 | 保证模型返回 tool_calls，Act 能执行 |
| 5 | 多轮示例（可选） | ObserveNode::with_loop + 条件边，或 react_zhipu_loop 示例 | 待办 | 见 13-react-agent-design §8.5 |
| 6 | Think 接 list_tools（可选） | ThinkNode 构建 prompt 时调用 ToolSource::list_tools | 待办 | 13 文档 §8.6 任务 6.1 |
| 7 | McpToolSource 接入（可选） | ReAct 示例配置为 McpToolSource | 待办 | 依赖 mcp-integration |

## 小结

- **LLM + ReAct Agent** = 用 **ChatOpenAI（或 ChatZhipu）** 作为 ThinkNode 的 LlmClient，**ToolSource**（如 MockToolSource）作为 ActNode 的工具来源，**StateGraph** 组织 think → act → observe（线性或条件多轮）。
- **最小落地**：新增 `react_zhipu` 示例，替换 MockLlm 为 ChatZhipu（智谱）、Act 用 MockToolSource，带 `with_tools`，跑通一轮工具调用即可。
- **迭代**：多轮循环、Think 内 list_tools、McpToolSource 等按需排期。
