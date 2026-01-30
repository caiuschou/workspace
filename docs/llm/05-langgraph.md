# 与 langgraph 的关系

| 项目 | LLM 角色 |
|------|-----------|
| **thirdparty/langgraph** | 不实现 LLM；prebuilt 使用 `langchain_core` 的 `BaseChatModel` / `LanguageModelLike`，由调用方传入具体模型。 |
| **thirdparty/langchain** | 提供上述抽象与各厂商 Chat/LLM 实现，LangGraph 的 Python 侧实际调用的就是这些实现。 |
