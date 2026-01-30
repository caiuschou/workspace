# 核心抽象（langchain_core）

## 模块入口与两类模型

- **`language_models/__init__.py`**：导出两类模型——**Chat 模型**（消息进、消息出）和 **LLM**（字符串进、字符串出，偏旧接口）。Chat 的入口抽象为 `BaseChatModel`。

## BaseLanguageModel（base.py）

- 所有语言模型的基类：`BaseLanguageModel`，继承 `RunnableSerializable[LanguageModelInput, LanguageModelOutputVar]`。
- 抽象方法：`generate_prompt` / `agenerate_prompt`（接收 `list[PromptValue]`，返回 `LLMResult`）。
- 类型约定：
  - `LanguageModelInput`：PromptValue | str | 消息序列
  - `LanguageModelOutput`：BaseMessage | str
  - `LanguageModelLike = Runnable[LanguageModelInput, LanguageModelOutput]`
- 通用能力：cache、verbose、callbacks、token 计数（`get_token_ids` / `get_num_tokens`）等。

## BaseChatModel（chat_models.py）

- **`BaseChatModel(BaseLanguageModel[AIMessage])`**：Chat 模型基类。
- 对外接口：`invoke` / `ainvoke`（单次）、`stream` / `astream`（流式）、`generate` / `agenerate`（批量）。
- 子类必须实现：
  - **`_generate(messages, stop=..., run_manager=..., **kwargs) -> ChatResult`**
  - **`_llm_type`**（字符串标识）。
- 可选：`_stream` / `_astream`；未实现时走 `_generate` 的非流式路径。
- 内置能力：缓存、callback、rate_limiter、`bind_tools`、`with_structured_output` 等。
- **SimpleChatModel**：简化子类，只需实现 `_call` 返回 str 即可。

## BaseLLM / LLM（llms.py）

- **LLM** / **BaseLLM**：传统「字符串进、字符串出」的 LLM 接口，与 Chat 并列，供旧用法与兼容。
