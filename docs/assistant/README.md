# Assistant

> 智能对话助手 - 用户与 AI 交互的统一入口

Assistant 是一个类似 ChatGPT 的智能对话助手，用户可以与其进行自然语言交互。它通过 **Agora** 与其他专业 Agent 协作，完成复杂任务。

## 快速开始

### 安装

```bash
# CLI 版本
npm install -g @assistant/cli

# 或使用 Rust 版本
cargo install assistant
```

### 基本使用

```bash
# 启动交互式对话
assistant

# 指定助手类型
assistant --profile casual

# 指定 Agora 服务器
assistant --agora ws://localhost:8080/ws
```

### 程序化使用

```typescript
import { Assistant } from '@assistant/sdk';

const assistant = new Assistant({
  agoraUrl: 'ws://localhost:8080/ws',
  profile: 'professional',
  llm: {
    provider: 'anthropic',
    model: 'claude-sonnet-4'
  }
});

await assistant.connect();

// 简单对话
const response = await assistant.chat('帮我分析一下这个项目的结构');
console.log(response.text);
```

## 核心特性

| 特性 | 说明 |
|------|------|
| **多助手类型** | 支持专业、轻松、创意等多种对话风格 |
| **记忆管理** | 跨会话持久化对话历史，支持长期记忆 |
| **Agent 协作** | 通过 Agora 与专业 Agent 无缝协作 |
| **上下文管理** | 智能维护对话状态和任务进度 |
| **多模型支持** | Anthropic、OpenAI、Ollama 等 |

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │   CLI    │  │   Web    │  │ Desktop  │  │   IDE    │   │
│  └─────┬────┘  └─────┬────┘  └─────┬────┘  └─────┬────┘   │
└────────┼────────────┼────────────┼────────────┼───────────┘
         │            │            │            │
         └────────────┴────────────┴────────────┘
                              │
                    ┌─────────▼─────────┐
                    │     Assistant     │
                    │       Core        │
                    │  ┌─────────────┐  │
                    │  │ Personality │  │
                    │  │   Engine    │  │
                    │  ├─────────────┤  │
                    │  │    Memory   │  │
                    │  │   Manager   │  │
                    │  ├─────────────┤  │
                    │  │   Context   │  │
                    │  │   Manager   │  │
                    │  ├─────────────┤  │
                    │  │   Router    │  │
                    │  └─────────────┘  │
                    └─────────┬─────────┘
                              │
                    ┌─────────▼─────────┐
                    │      Agora        │
                    │  Communication    │
                    │       Layer       │
                    └─────────┬─────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
    ┌────▼────┐         ┌────▼────┐         ┌────▼────┐
    │  Coder  │         │ Search  │         │ Executor│
    │  Agent  │         │  Agent  │         │  Agent  │
    └─────────┘         └─────────┘         └─────────┘
```

## 助手类型

Assistant 支持配置不同的对话风格，满足用户偏好：

| 类型 | 风格 | 适用场景 |
|------|------|----------|
| `professional` | 专业、严谨、结构化 | 工作场景、技术讨论 |
| `casual` | 友好、轻松、口语化 | 日常对话、快速问答 |
| `creative` | 富有想象力、启发性 | 创意头脑、设计讨论 |
| `concise` | 简洁、直接、高效 | 命令式交互、脚本场景 |

详见 [助手类型配置](./personality.md)。

## 与 Agent 协作

Assistant 通过 Agora 与专业 Agent 协作：

```typescript
// 用户提问
const response = await assistant.chat('帮我重构 auth 模块');

// Assistant 自动：
// 1. 识别需要代码分析能力
// 2. 通过 Agora 调用 Coder Agent
// 3. 聚合结果并用当前风格回复
```

协作模式：
- **自动路由** - 根据意图自动选择合适的 Agent
- **手动调用** - 使用 `@agent` 语法显式指定
- **并行执行** - 多个 Agent 并行工作，聚合结果

## 记忆系统

Assistant 维护多层记忆：

| 层级 | 说明 | 持久化 |
|------|------|--------|
| **短期记忆** | 当前会话上下文 | 会话期间 |
| **会话记忆** | 历史对话记录 | SQLite |
| **长期记忆** | 跨会话的知识提取 | Vector DB |

详见 [记忆管理](./memory.md)。

## 配置文件

Assistant 使用 `assistant.json` 进行配置：

```json
{
  "agora": {
    "url": "ws://localhost:8080/ws",
    "agent_id": "assistant-001"
  },
  "llm": {
    "provider": "anthropic",
    "model": "claude-sonnet-4",
    "temperature": 0.7
  },
  "profile": "professional",
  "memory": {
    "max_sessions": 100,
    "max_messages_per_session": 1000,
    "vector_store": "./data/memory.db"
  }
}
```

## 链接

- [架构设计](./architecture.md) - 详细架构和模块说明
- [助手类型](./personality.md) - 助手类型配置和自定义
- [记忆管理](./memory.md) - 记忆系统详解
- [API 参考](./api.md) - 完整 API 文档
- [Agora 协议](../agora/spec.md) - Agent 通信协议
