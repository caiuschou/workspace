# 具体实现（classic + partners）

## langchain_classic/chat_models

- **base.py**：从 `langchain_core` 的 `BaseChatModel` 做兼容与工厂，提供 **`init_chat_model(model, model_provider=..., **kwargs)`**，根据 `model` 或 `model_provider` 选择厂商并返回对应 Chat 模型实例。
- 各文件对应厂商：`openai.py`、`anthropic.py`、`bedrock.py`、`vertexai.py`、`ollama.py` 等。

## partners/openai（示例）

- **ChatOpenAI** 等继承 `BaseChatModel`，实现：
  - 将 LangChain 的 `BaseMessage` 转成 OpenAI API 的 messages 格式；
  - 调用官方 `openai` SDK 的 chat completions（含流式）；
  - 将返回转成 `AIMessage` / `AIMessageChunk`、`ChatResult` / `ChatGenerationChunk`；
  - 支持 tool_calls、structured output、model profile 等。

## langchain_classic/llms

- 传统 LLM（string in/out）的各厂商实现：`openai.py`、`anthropic.py`、`ollama.py` 等。
