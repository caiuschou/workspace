# rust-langgraph：最简单版本 ChatOpenAI 方案

## 目标

在 rust-langgraph 中提供「最简单版本的 ChatOpenAI」：**消息进 → OpenAI Chat Completions → 助手消息出**。本版不包含流式、structured output、model profile、批量 generate 等，仅完成单轮对话与可选 tool_calls。

## 概念说明

**ChatOpenAI** 即「基于 OpenAI Chat Completions API 的对话客户端」：接收消息列表，转成 OpenAI 请求格式，调用 API，将返回的助手内容与可选 tool_calls 转成 rust-langgraph 的 `LlmResponse`。命名与常见生态（如 LangChain 的 ChatOpenAI）一致，便于理解与对照。

## 与 LangChain 对齐

OpenAI 相关设计与 LangChain 的 Chat 模型体系对齐，便于从 LangChain 迁移或对照文档。对应关系如下。

| LangChain（langchain_core / partners） | rust-langgraph |
|----------------------------------------|----------------|
| **BaseChatModel**（Chat 模型抽象） | **LlmClient** trait：消息进、助手内容 + 可选 tool_calls 出。 |
| **invoke** / **ainvoke**（单次调用） | **invoke**(messages)：单次请求，返回本轮助手结果。 |
| **BaseMessage**（SystemMessage / HumanMessage / AIMessage） | **Message** 枚举：`System(String)` / `User(String)` / `Assistant(String)`，与 OpenAI API 角色一致。 |
| **AIMessage**、**ChatResult**（含 generations[].message） | **LlmResponse**：`content: String`（助手文本）+ `tool_calls: Vec<ToolCall>`。 |
| **ChatOpenAI**（继承 BaseChatModel，调 OpenAI Chat Completions） | **ChatOpenAI**（实现 LlmClient，主类型）。 |
| **bind_tools**(tools) | **with_tools**(tools)：传入工具列表，API 可返回 tool_calls。 |
| 配置（api_key、base_url 等） | **OpenAIConfig** + **with_config**(config, model)；默认从 **OPENAI_API_KEY** 读 key。 |
| 消息 → OpenAI 请求格式 → API → 解析为 AIMessage/ChatResult | 消息 → **messages_to_request** → **client.chat().create** → 解析为 **LlmResponse**。 |

### rust-langgraph 已有功能

以下能力在代码中**已实现**，可直接使用（以 `crates/langgraph/src/llm/` 与 `message.rs` 为准）：

| 功能 | 说明 | 代码位置 / API |
|------|------|----------------|
| **单次调用 invoke ≈ invoke** | 消息进、助手内容 + 可选 tool_calls 出。 | `LlmClient::invoke(&[Message]) -> Result<LlmResponse, AgentError>`；`ChatOpenAI` 已实现。 |
| **消息类型 Message** | System / User / Assistant，与 OpenAI 角色一致。 | `message.rs`：`Message::system/user/assistant`。 |
| **响应类型 LlmResponse** | `content: String` + `tool_calls: Vec<ToolCall>`（含 name、arguments、id）。 | `llm/mod.rs`；由 `invoke` 返回。 |
| **OpenAI 客户端 ChatOpenAI** | 调 OpenAI Chat Completions，实现 `LlmClient`。 | `llm/openai.rs`，需 feature `openai`。 |
| **智谱客户端 ChatZhipu** | 调智谱 GLM Chat Completions（OpenAI 兼容），实现 `LlmClient`。 | `llm/zhipu.rs`，需 feature `openai`；`ZHIPU_API_KEY`、base `https://open.bigmodel.cn/api/paas/v4`。 |
| **构造与配置** | 默认从环境变量读 key；支持自定义 config。 | `ChatOpenAI::new(model)`、`ChatOpenAI::with_config(config, model)`。 |
| **工具调用 bind_tools ≈ with_tools** | 传入工具列表，API 可返回 tool_calls。 | `ChatOpenAI::with_tools(tools: Vec<ToolSpec>)`；请求中带 `tools`，响应解析为 `LlmResponse::tool_calls`。 |
| **环境变量 OPENAI_API_KEY** | 默认 API key 来源。 | `Client::new()` 使用 `OpenAIConfig` 默认行为（async_openai）。 |
| **Mock 实现** | 测试用固定响应、可配置有无 tool_calls。 | `MockLlm`（`llm/mock.rs`）。 |
| **与 ThinkNode 集成** | 图节点内使用 LLM 完成一轮推理。 | ThinkNode 持有 `Box<dyn LlmClient>`，调用 `invoke(state.messages)`，写回 `content` 与 `tool_calls`。 |

### 已有功能与 LangChain / LangGraph 的差异

在「已有功能」对齐的前提下，下列差异来自类型体系、API 形状和职责划分，迁移或对照时需注意。

| 维度 | LangChain / LangGraph | rust-langgraph（已有） | 说明 |
|------|------------------------|------------------------|------|
| **抽象归属** | LangGraph **不实现** LLM，依赖 `langchain_core` 的 `BaseChatModel` / `LanguageModelLike`，由调用方传入。 | **本库内**定义并实现 `LlmClient` 与 `ChatOpenAI`，不依赖外部 Chat 抽象。 | rust 侧自洽一套抽象，无 Python 的 langchain_core 依赖。 |
| **调用入口命名** | `invoke` / `ainvoke`（单次），统一在 `Runnable` 上。 | `invoke`（单次），在 `LlmClient` 上。 | 语义等价（单次请求），方法名不同；rust 无 Runnable 体系。 |
| **输入类型** | `LanguageModelInput`：可为 PromptValue、str、消息序列；`invoke` 前会 `_convert_input`。 | 仅 `&[Message]`，无裸 str、无 PromptValue。 | rust 侧只接受消息列表，简化类型；多模态/复杂 input 需在调用前转成 Message。 |
| **输出类型** | 单次返回 `AIMessage`；底层 `_generate` 返回 `ChatResult`（含 `generations: list[ChatGeneration]`）。 | 直接返回 `LlmResponse`（`content` + `tool_calls`），无 ChatResult / generations 包装。 | rust 侧一层扁平结构，无「多候选」generations；如需可后续加。 |
| **消息类型** | `BaseMessage` 子类：`SystemMessage`、`HumanMessage`、`AIMessage` 等，可有 `additional_kwargs`、多模态 content、tool_calls 等。 | `Message` 枚举：`System(String)` / `User(String)` / `Assistant(String)`，仅文本。 | rust 侧极简：纯文本、无 additional_kwargs、无独立 ToolMessage 角色（工具结果通常写在 Assistant 或状态里）。 |
| **工具绑定** | `bind_tools(tools)` 返回新的 Runnable，调用时带 tools。 | `with_tools(tools)` 为 `ChatOpenAI` 的 builder，同一实例带 tools。 | 效果等价（请求带 tools），API 形态不同：LangChain 是「绑定后新 runnable」，rust 是「构造时/链式配置」。 |
| **配置与模型** | ChatOpenAI 构造参数：model、api_key、base_url、temperature、max_tokens 等。 | `new(model)`、`with_config(config, model)`；**未暴露** temperature、max_tokens 等（用 API 默认）。 | rust 侧仅覆盖「谁调、去哪」；采样参数等属「本版未覆盖」的 model profile。 |
| **错误与回调** | 异常 + callback（on_llm_start、on_llm_new_token 等）、verbose。 | `Result<LlmResponse, AgentError>`，无内置 callback/verbose。 | rust 侧仅错误返回；观测/回调属「本版未覆盖」扩展点。 |
| **与图的集成** | LangGraph 节点接收 `LanguageModelLike`，内部调 `invoke`/`astream` 等。 | ThinkNode 接收 `Box<dyn LlmClient>`，内部只调 `invoke`。 | 概念一致（图节点持有模型、调单次/流式），rust 侧目前仅单次 `invoke`。 |

总结：已有功能在「单次调用、消息进/出、tool_calls、配置与工具」上与 LangChain/LangGraph 对齐；差异主要在类型更简（Message/LlmResponse 无多层包装）、无 Runnable/无 generations、无内置 callback/verbose/model profile，以及抽象归属在库内而非外部。

尚未实现（见下）：流式、批量 generate、with_structured_output、model profile（max_tokens/temperature 等）、callback/verbose。

**本版未覆盖（与 LangChain 的扩展点）**：下列能力当前未实现，后续若需要可与 LangChain 按同一抽象扩展。

| 扩展点 | LangChain 行为 | rust-langgraph 扩展方式（建议） |
|--------|----------------|----------------------------------|
| **流式 stream / astream** | `stream(input)` / `astream(input)` 产出 `AsyncIterator[AIMessageChunk]`，按 token 或片段返回。 | 在 `LlmClient` 上增加 `stream(&self, messages) -> impl Stream<Item = Result<LlmChunk, AgentError>>`，或单独 trait `LlmClientStream`；`ChatOpenAI` 内用 `async_openai` 的 stream API，产出 `LlmChunk { content_delta, tool_calls_delta }`。 |
| **批量 generate / agenerate** | `generate([messages1, messages2, ...])` 返回 `LLMResult`（多组 `ChatResult`），用于批量或并行请求。 | 在 `LlmClient` 上增加 `invoke_batch(&self, messages_list: &[Vec<Message>]) -> Result<Vec<LlmResponse>, AgentError>`，或单独方法；内部对每组 messages 调一次 API 或并发请求。 |
| **with_structured_output** | 将输出约束为 JSON Schema / Pydantic 模型，API 侧用 `response_format`（如 `json_schema`）保证结构。 | `ChatOpenAI::with_structured_output(schema)` 或 builder，请求时设置 `response_format`；返回仍为 `LlmResponse`，调用方将 `content` 反序列化为目标类型，或新增 `LlmResponse::structured<T>()`。 |
| **model profile** | 模型维度配置（max_tokens、temperature、top_p 等）在 ChatOpenAI 构造或调用时传入。 | `ChatOpenAI` 构造或 builder 增加 `max_tokens`、`temperature` 等字段，在 `CreateChatCompletionRequestArgs` 中传入；与 `with_config` 并列，仅影响请求体。 |
| **callback / verbose** | 调用前后及流式片段的回调（on_llm_start、on_llm_new_token 等）、verbose 打印请求/响应。 | 在 `LlmClient` 或 `ChatOpenAI` 上增加可选 `callbacks: Vec<Box<dyn LlmCallback>>`，在 `invoke`/`stream` 关键点调用；或通过 `RustTracing`/log 在实现内打点，由上层决定是否订阅。 |

扩展时保持 **LlmClient** 为单次 `invoke` 的稳定抽象，流式/批量可作为同一 trait 的默认实现（如 `fn stream(...) { unimplemented!() }`）或子 trait，避免破坏现有 ThinkNode 等调用方。

## 现状

| 项目 | 说明 |
|------|------|
| **抽象** | `LlmClient`：`invoke(messages) -> Result<LlmResponse, AgentError>`；`LlmResponse` 含 `content: String` 与 `tool_calls: Vec<ToolCall>`。 |
| **实现** | `ChatOpenAI`、`ChatZhipu` 已实现 `LlmClient`，支持默认配置、可选 `with_config`、可选 `with_tools`。 |
| **消息类型** | `Message` 枚举：`System(String)`、`User(String)`、`Assistant(String)`，与 OpenAI API 的 system/user/assistant 一一对应。 |
| **代码位置** | `rust-langgraph/crates/langgraph/src/llm/`：`mod.rs`（trait 与导出）、`openai.rs`（ChatOpenAI）、`zhipu.rs`（ChatZhipu）、`mock.rs`（MockLlm）；需 feature `openai`。 |

即：**主类型为 ChatOpenAI**，另有 **ChatZhipu**（智谱 GLM）；命名与文档已与 LangChain 对齐。

## 最简单版本定义

- **必做**：messages 进 → 转成 OpenAI 请求消息 → 调用 Chat Completions → 返回助手文本（及可选 `tool_calls`）。
- **不做（本版）**：流式、structured output、model profile、批量 generate 等。
- **命名**：对外暴露类型名 `ChatOpenAI`，便于与文档及常见生态一致。

## 数据流

1. 调用方构造 `Vec<Message>`（如 `[System(...), User(...)]` 或带历史 Assistant 的多轮）。
2. `LlmClient::invoke(&messages)` 内部将 `Message` 转为 `async_openai` 的 `ChatCompletionRequestMessage`（system/user/assistant 文本）。
3. 使用 `CreateChatCompletionRequestArgs` 构建请求（model、messages、可选 tools），调用 `client.chat().create(request)`。
4. 从响应的 `choices[0].message` 取 `content` 与 `tool_calls`，构造 `LlmResponse { content, tool_calls }` 返回。
5. 使用方（如 ThinkNode）将 `content` 写入新助手消息、`tool_calls` 写入状态，继续图执行。

## 实现方案

| 方案 | 做法 | 优点 | 缺点 |
|------|------|------|------|
| **A. 类型别名** | `pub type ChatOpenAI = ChatOpenAI` | 零逻辑、命名统一、与现有 `ChatOpenAI` 完全一致 | 仍暴露 `with_config` / `with_tools` |
| **B. Newtype** | `pub struct ChatOpenAI(ChatOpenAI)`，只暴露 `new(model)` 和 `LlmClient` | 最小 API：仅「model + invoke」 | 多一层包装，扩展时需转发 |

**推荐**：先采用 **方案 A**。理由：  
- 现有 `ChatOpenAI::new(model)` 已是「最简单」用法；  
- 类型别名即可满足「ChatOpenAI」命名与文档一致；  
- 若后续需要「仅最小 API」再改为 Newtype 或增加 `ChatOpenAI::minimal()` 等构造函数。

## 依赖与 feature

- **Feature**：`openai`（在 `crates/langgraph/Cargo.toml` 中声明）。  
- **依赖**：`async-openai`（可选，仅当 `openai` 开启时），用于 Chat Completions。  
- **环境**：默认从 `OPENAI_API_KEY` 读 API key；或通过 `ChatOpenAI::with_config(config, model)` 传入自定义 `OpenAIConfig`（含 base URL 等）。

## 使用方式

- **最小用法**：`ChatOpenAI::new("gpt-4o-mini")` 或 `ChatOpenAI::new("gpt-4o-mini")`（方案 A 下即 `ChatOpenAI`），然后 `client.invoke(&messages).await`。  
- **自定义配置**：`ChatOpenAI::with_config(config, "gpt-4o-mini")`。  
- **带工具**：`.with_tools(tools)`（如 `ToolSource::list_tools()`），API 可能返回 `tool_calls`，由 `LlmResponse::tool_calls` 提供。  
- **与 ThinkNode**：ThinkNode 持有 `Box<dyn LlmClient>`，调用 `invoke(state.messages)`，将返回的 `content` 与 `tool_calls` 写回状态；见 [13-react-agent-design](13-react-agent-design.md) §8.2。

## ChatOpenAI 使用说明

### 编译与依赖

启用 `openai` feature 后即可使用 `ChatOpenAI`（即当前文档中的 ChatOpenAI 实现）：

```bash
cargo build --features openai
```

依赖：`langgraph` crate 中已声明可选依赖 `async-openai`，仅当 `openai` 开启时链接。

### 环境变量

- **OPENAI_API_KEY**（必填）：调用 OpenAI Chat Completions 时默认从该环境变量读取 API key。未设置时请求会失败。
- 若使用兼容 OpenAI 的代理或自建服务，可通过 `ChatOpenAI::with_config(config, model)` 传入自定义 `OpenAIConfig`（含 `base_url` 等），不依赖默认环境。

### 基本用法：单轮对话

1. 构造消息列表 `Vec<Message>`：通常包含 `Message::System(...)`（可选）和 `Message::User(...)`。
2. 使用 `ChatOpenAI::new(model)` 创建客户端，`model` 为模型名（如 `"gpt-4o-mini"`、`"gpt-4o"`）。
3. 调用 `client.invoke(&messages).await`，得到 `Result<LlmResponse, AgentError>`。
4. 从 `LlmResponse` 中取 `content`（助手文本）和 `tool_calls`（本版未传 tools 时一般为空）。

示例（伪代码）：

```rust
use langgraph::{Message, ChatOpenAI, LlmClient}; // 需 feature "openai"

let messages = vec![
    Message::system("You are a helpful assistant."),
    Message::user("Hello, say hi in one sentence."),
];
let client = ChatOpenAI::new("gpt-4o-mini");
let response = client.invoke(&messages).await?;
println!("{}", response.content);
// response.tool_calls 为空（未设置 with_tools）
```

### 多轮对话

将历史助手回复也放入 `messages`，再追加新的用户消息，然后调用 `invoke`。例如：`[system, user, assistant, user]` 表示一轮历史 + 新一轮用户输入，API 会基于完整上下文生成下一句助手回复。

### 自定义配置（API key / base URL）

当需要自定义 endpoint 或 API key 时，使用 `with_config`：

```rust
use async_openai::config::OpenAIConfig;

let config = OpenAIConfig::new()
    .with_api_key("sk-...")
    .with_base_url("https://your-proxy.com/v1");
let client = ChatOpenAI::with_config(config, "gpt-4o-mini");
// 之后用法同基本用法：client.invoke(&messages).await
```

### 带工具调用（tool_calls）

若希望模型返回工具调用，先通过 `with_tools` 传入工具列表（如来自 `ToolSource::list_tools()`），再调用 `invoke`。返回的 `LlmResponse::tool_calls` 非空时，调用方需执行对应工具并把结果以观察形式写回消息或状态，再决定是否继续调用 `invoke`（参见 ReAct 循环）。

```rust
let tools: Vec<ToolSpec> = tool_source.list_tools(); // 从 ToolSource 获取
let client = ChatOpenAI::new("gpt-4o-mini").with_tools(tools);
let response = client.invoke(&messages).await?;
for tc in &response.tool_calls {
    // tc.name, tc.arguments, tc.id；执行工具后写入 state / messages
}
```

### 在 ThinkNode 中使用

ThinkNode 接收 `Box<dyn LlmClient>`，可将 `ChatOpenAI` 装箱后传入，由 ThinkNode 内部对 `state.messages` 调用 `invoke`，并把返回的 `content` 与 `tool_calls` 写回 ReAct 状态。无需在业务代码中手动调用 `invoke`，只需构造好 ThinkNode 时传入 LLM 客户端即可。详见 [13-react-agent-design](13-react-agent-design.md) §8.2。

```rust
let llm: Box<dyn LlmClient> = Box::new(ChatOpenAI::new("gpt-4o-mini"));
let think = ThinkNode::new(llm);
// 将 think 作为节点加入 StateGraph，按 ReAct 流程运行
```

## 开发计划

### 阶段划分

| 阶段 | 目标 | 优先级 | 状态 |
|------|------|--------|------|
| **阶段 0** | 命名与文档收尾：ChatOpenAI 主类型、llm 模块与 README 导出与说明 | P0 | **已完成** |
| **阶段 1** | model profile：max_tokens、temperature、top_p 等可配置，与 LangChain 对齐 | P1 | 待办 |
| **阶段 2** | 流式 stream：LlmClient 扩展或子 trait，ChatOpenAI 支持 stream，产出 LlmChunk | P2 | 待办 |
| **阶段 3** | 可选扩展：with_structured_output、invoke_batch、callback/verbose（按需排期） | P3 | 待办 |

### 任务表

| 序号 | 任务 | 阶段 | 状态 | 备注 |
|------|------|------|------|------|
| 1 | 编写方案与对齐文档（本文档） | 0 | 已完成 | 目标、对齐、差异、扩展点 |
| 2 | 将 OpenAILlm 重命名为 ChatOpenAI（主类型，非别名） | 0 | 已完成 | 已执行：openai.rs 中 struct/impl 均为 ChatOpenAI |
| 3 | 更新 llm/mod.rs 与 llm/README.md：导出 ChatOpenAI、使用说明 | 0 | 已完成 | 依赖任务 2 |
| 4 | 更新 crate 根 lib.rs 导出 ChatOpenAI（若需对外暴露） | 0 | 已完成 | 依赖任务 2 |
| 5 | model profile：ChatOpenAI 增加 max_tokens、temperature、top_p 等 builder/字段 | 1 | 待办 | 请求体参数，见「本版未覆盖」表 |
| 6 | 流式：定义 LlmChunk / Stream，LlmClient 扩展或 LlmClientStream，ChatOpenAI::stream | 2 | 待办 | 可选默认实现 unimplemented，不破坏现有 invoke |
| 7 | with_structured_output（可选）：builder + response_format，反序列化辅助 | 3 | 待办 | 按需 |
| 8 | invoke_batch 或 generate 批量（可选） | 3 | 待办 | 按需 |
| 9 | callback / verbose（可选）：LlmCallback trait 或 tracing 打点 | 3 | 待办 | 按需 |

### 执行顺序建议

- **阶段 0**：任务 2 → 3 → 4（已完：ChatOpenAI 主类型、llm 与 lib 导出、README）。
- **阶段 1**：任务 5 独立，与 ChatOpenAI 请求构建处对接。
- **阶段 2**：任务 6 在阶段 0/1 稳定后进行，保证 LlmClient::invoke 不变。
- **阶段 3**：按需求择一或多，无强依赖顺序。

### 阶段 0 完成情况

阶段 0 已全部完成：ChatOpenAI 为主类型（原 OpenAILlm 已重命名），`llm/mod.rs`、`llm/README.md`、crate 根 `lib.rs` 已导出并更新说明；编译通过（`cargo build --features openai`）。后续按需进行阶段 1（model profile）及以后。

### 方案交付状态

**本方案交付完成。** 阶段 0 全部完成（任务 1–4）；阶段 1–3（任务 5–9）为后续迭代，按需排期。
