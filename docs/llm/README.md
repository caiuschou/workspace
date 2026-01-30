# LLM 实现文档

本文档基于对工作区 `thirdparty/langchain` 的阅读，整理 LangChain 侧 LLM（含 Chat 模型）的抽象与实现结构，以及与 thirdparty/langgraph 的关系。

## 文档目录

| 文档 | 内容 |
|------|------|
| [README.md](README.md) | 概述、文档索引、任务记录 |
| [01-structure.md](01-structure.md) | thirdparty/langchain LLM 代码结构 |
| [02-abstractions.md](02-abstractions.md) | 核心抽象（BaseLanguageModel、BaseChatModel、BaseLLM） |
| [03-implementations.md](03-implementations.md) | 具体实现（classic chat_models、partners、classic llms） |
| [04-dataflow.md](04-dataflow.md) | Chat 模型数据流 |
| [05-langgraph.md](05-langgraph.md) | 与 langgraph 的关系 |

## 概述

- **抽象与接口**：在 `langchain_core` 的 `language_models` 中定义（Chat 模型基类 `BaseChatModel`、传统 LLM 基类 `BaseLLM`、输入输出类型等）。
- **具体实现**：分布在 `libs/langchain/langchain_classic` 的 `chat_models/`、`llms/`，以及 `libs/partners/` 下各厂商包（如 `langchain-openai`）。
- **thirdparty/langgraph**：不实现 LLM，prebuilt 的 `chat_agent_executor` 等依赖 `langchain_core` 的 `BaseChatModel` / `LanguageModelLike`，由调用方传入具体 Chat 模型（如 `ChatOpenAI`）。

## 任务记录

| 任务 | 状态 |
|------|------|
| 阅读 thirdparty/langchain LLM 实现并整理结构 | 已完成 → 本目录各文档 |
| 将 LLM 文档写入 docs/llm | 已完成 → README.md |
| 细化文档颗粒度（按结构/抽象/实现/数据流/关系拆分） | 已完成 → 01–05 |
