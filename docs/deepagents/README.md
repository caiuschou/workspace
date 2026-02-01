# Deep Agents 文档

本文档介绍 [LangChain Deep Agents](https://github.com/langchain-ai/deepagents) 的架构、设计与实现，供 Rust OpenCode SDK 及 Agent 开发参考。

> 代码位置：`thirdparty/deepagents`（需自行 clone，不提交）

## 文档目录

| 文档 | 内容 |
|------|------|
| [01-overview.md](01-overview.md) | 概述、项目结构、核心能力 |
| [02-architecture.md](02-architecture.md) | 架构：中间件链、工具、子 Agent |
| [03-backend-protocol.md](03-backend-protocol.md) | 后端协议与实现（State/Filesystem/Store/Composite） |
| [04-middleware-deep-dive.md](04-middleware-deep-dive.md) | **深度**：中间件实现细节（生命周期、eviction、PatchToolCalls） |
| [05-summarization-memory-skills.md](05-summarization-memory-skills.md) | **深度**：摘要、记忆、技能的实现与持久化 |
| [06-backend-implementation.md](06-backend-implementation.md) | **深度**：各后端实现细节（State/Filesystem/Composite/Sandbox） |
| [07-design-decisions.md](07-design-decisions.md) | 设计决策、数据流、安全考量、Rust 移植要点 |

## 核心能力概览

| 能力 | 说明 |
|------|------|
| Planning | `write_todos` / `read_todos` 任务拆解与进度跟踪 |
| Filesystem | `ls`, `read_file`, `write_file`, `edit_file`, `glob`, `grep` |
| Shell | `execute` 执行命令（需 SandboxBackendProtocol） |
| Sub-agents | `task` 委托任务，隔离上下文窗口 |
| Skills | 从 `skills/` 目录加载 SKILL.md |
| Memory | 从 `AGENTS.md` 加载长期记忆 |

## 与 OpenCode SDK 的参考关系

| Deep Agents | OpenCode SDK |
|-------------|--------------|
| BackendProtocol | Workspace / File 抽象 |
| StateBackend (invoke files={...}) | Session 传入的 workspace 文件 |
| SubAgent / task | 多 Agent 协作（langgraph-rust） |
| Skills (SKILL.md) | Agent Skill 体系 |

## 参考资料

- [Deep Agents GitHub](https://github.com/langchain-ai/deepagents)
- [Deep Agents 官方文档](https://docs.langchain.com/oss/python/deepagents/overview)
- [thirdparty/deepagents/](../../thirdparty/deepagents/) - 本地代码
