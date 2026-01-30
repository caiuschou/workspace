# LLM 客户端抽象

本模块为 ReAct **Think** 节点提供 LLM 客户端抽象：输入消息列表，返回助手文本及可选的工具调用。设计见 [13-react-agent-design](../../../../docs/rust-langgraph/13-react-agent-design.md) §8.2。

## 类型

- **`LlmResponse`**：单轮响应，含 `content`（助手文本）和 `tool_calls`；`tool_calls` 为空表示本轮不调用工具，图走向 observe → END。
- **`LlmClient`**：异步 trait，`complete(messages)` 返回 `LlmResponse`。ThinkNode 通过它生成下一条助手消息和工具调用。

## 实现

- **`MockLlm`**：固定返回，用于测试和示例。可配置有无 `tool_calls`、是否多轮（第一次返回 tool_calls，第二次不返回）。
- **`OpenAILlm`**：真实 OpenAI Chat Completions API，需启用 feature `openai`，默认从环境变量 `OPENAI_API_KEY` 读 key，也可用 `with_config` 自定义；可选 `with_tools(tools)` 开启工具调用。

## MockLlm 用法

- `MockLlm::with_get_time_call()`：固定返回一条助手消息 + 一条 `get_time` 工具调用。
- `MockLlm::with_no_tool_calls(content)`：只返回文本，无工具调用（END 路径）。
- `MockLlm::new(content, tool_calls)`：自定义内容和工具调用。
- `MockLlm::first_tools_then_end()`：多轮用，第一次返回 get_time，第二次不返回。
- Builder：`with_content`、`with_tool_calls`。

## OpenAILlm 用法（feature `openai`）

编译：`cargo build --features openai`。需设置 `OPENAI_API_KEY` 或通过 `with_config` 传入。

- `OpenAILlm::new(model)`：用默认配置（环境变量）。
- `OpenAILlm::with_config(config, model)`：自定义 API key / base URL。
- `.with_tools(tools)`：传入工具列表（如 `ToolSource::list_tools()`）以启用工具调用。

## 与 ThinkNode 的关系

ThinkNode 持有 `Box<dyn LlmClient>`（如 MockLlm 或 OpenAILlm），调用 `complete(state.messages)`，将返回的 `content` 写入新助手消息、`tool_calls` 写入 `ReActState::tool_calls`。

## 文件

- `mod.rs`：`LlmResponse`、`LlmClient` 及导出。
- `mock.rs`：MockLlm。
- `openai.rs`：OpenAILlm（仅在 feature `openai` 下编译）。
