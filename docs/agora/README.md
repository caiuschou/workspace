# Agora

> Agent 通信空间 - 让运行在不同位置的 Agent 可以相互协作

## 快速开始

### 安装

```bash
# 服务端 (Rust)
cargo install agora-server

# 客户端 SDK (TypeScript)
npm install @opencode/agora-sdk
```

### 服务端

```bash
agora-server --port 8080
```

### 客户端

```typescript
import { AgoraClient } from '@opencode/agora-sdk';

const client = new AgoraClient({
  url: 'ws://localhost:8080/ws',
  agentId: 'agent-001'
});

await client.connect();

// 加入协作空间
await client.space.join('general');

// 发布消息
await client.space.publish('general', { text: 'Hello' });

// 监听消息
client.on('space.event', (event) => {
  console.log(event.data);
});
```

## 什么是 Agora？

**Agora（集市广场）** 是一个基于 WebSocket 的 Agent 通信层。

Agent 可能运行在：
- 独立进程
- 线程
- 分布式节点

Agora 让这些 Agent 能够：
- **发现彼此** - 注册、心跳、状态同步
- **组建 Space** - 加入/创建通信空间
- **Pub/Sub** - 发布消息，订阅感兴趣的内容

## 核心概念

### Agent

具有唯一 ID 和能力的代理实体。

### Space

统一的通信空间，通过命名约定区分用途：

| 模式 | 用途 |
|------|------|
| `general` | 群组协作 |
| `agent.status` | 系统事件 |
| `task.123` | 任务空间 |
| `file.changed` | 文件事件 |

## 使用场景

```typescript
// 场景 1: 多 Agent 协作
await client.space.join('project-discussion');
await client.space.publish('project-discussion', {
  type: 'suggestion',
  content: '建议使用 Redis 缓存'
});

// 场景 2: 系统状态监控
await client.space.join('agent.status');
await client.space.join('agent.error');
// 自动接收所有 Agent 的状态和错误

// 场景 3: 任务协同
await client.space.join('task.456');
await client.space.publish('task.456', {
  type: 'progress',
  step: 'analyze',
  status: 'done'
});
```

## 链接

- [协议规范](./spec.md) - 事件协议和消息格式
- [开发文档](./dev.md) - 架构设计和实现指南
