# LangGraph Agent 开发指南

本文档介绍如何使用 LangGraph 开发 Agent 应用，包括基础概念、架构模式和实现方案。

## 文档目录

| 文档 | 内容 |
|------|------|
| [01-concepts.md](01-concepts.md) | 核心概念 |
| [02-chat-agent.md](02-chat-agent.md) | 对话 Agent |
| [03-react-agent.md](03-react-agent.md) | ReAct Agent（推理 + 行动） |
| [04-planning-agent.md](04-planning-agent.md) | 多步骤规划 Agent |
| [05-multi-agent.md](05-multi-agent.md) | 多 Agent 协作 |
| [06-memory-agent.md](06-memory-agent.md) | 带记忆的 Agent |
| [09-memory-chat-case.md](09-memory-chat-case.md) | **案例**：带记忆的对话 Agent（含 Telegram Bot 对接） |
| [10-memory-deep-dive.md](10-memory-deep-dive.md) | **深度研究**：LangGraph 记忆与持久化（Checkpoint / Store，代码在 thridparty） |
| [11-langchain-memory.md](11-langchain-memory.md) | **深度研究**：LangChain 记忆模块（chat_history + memory，参考 thridparty 代码） |
| [13-langchain-vs-langgraph-memory.md](13-langchain-vs-langgraph-memory.md) | **深度比较**：LangChain 与 LangGraph 记忆（架构、数据模型、API、选型与迁移） |
| [12-context-compaction.md](12-context-compaction.md) | **参考**：上下文压缩（以 Claude Code Auto-Compact 为例，对应本系列摘要压缩） |
| [07-production.md](07-production.md) | 生产级实现 |
| [08-best-practices.md](08-best-practices.md) | 最佳实践 |

## Agent 类型对比

| 类型 | 复杂度 | 适用场景 | 特点 |
|------|--------|----------|------|
| 对话 Agent | 低 | 简单对话 | 直接对话，无复杂逻辑 |
| ReAct | 中 | 需要推理 | 显式思考过程，可解释性强 |
| 规划 Agent | 高 | 复杂任务 | 先规划再执行，支持多步骤 |
| 多 Agent | 高 | 复杂系统 | 专业化分工，协作完成 |

## 任务记录

| 任务 | 状态 |
|------|------|
| 深度研究 LangGraph 记忆（代码在 thridparty） | 已完成 → [10-memory-deep-dive.md](10-memory-deep-dive.md) |
| 为 LangChain 记忆模块写文档（参考 thridparty） | 已完成 → [11-langchain-memory.md](11-langchain-memory.md) |
| 深度比较 LangChain 与 LangGraph 记忆 | 已完成 → [13-langchain-vs-langgraph-memory.md](13-langchain-vs-langgraph-memory.md) |

## 参考资料

- [LangGraph 官方文档](https://langchain-ai.github.io/langgraph/)
- [Checkpoint 持久化](https://langchain-ai.github.io/langgraph/concepts/persistence/)
- [LangGraph GitHub](https://github.com/langchain-ai/langgraph)
- [thridparty/langgraph/](../thridparty/langgraph/) - LangGraph 本地代码（参考实现，需自行 clone 至该目录）
- [thridparty/langchain/](../thridparty/langchain/) - LangChain 本地代码（记忆等模块对照，需自行 clone 至该目录）
