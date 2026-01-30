# thirdparty/langchain LLM 代码结构

| 位置 | 作用 |
|------|------|
| **libs/core/langchain_core/language_models/** | 抽象与类型（chat/llm 基类、LanguageModelInput/Output/Like） |
| **libs/langchain/langchain_classic/chat_models/** | 各厂商 Chat 模型实现（约 35 个文件） |
| **libs/langchain/langchain_classic/llms/** | 各厂商传统 LLM 实现（string in/out，约 80+ 个文件） |
| **libs/partners/** | 独立包实现（如 langchain-openai、langchain-anthropic 等） |
