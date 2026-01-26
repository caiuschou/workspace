# API 参考

> Assistant 的完整 API 文档

## 概述

Assistant 提供 TypeScript 和 Rust 两种语言的 SDK。

| SDK | 说明 | 安装 |
|-----|------|------|
| `@assistant/sdk` | TypeScript SDK | `npm install @assistant/sdk` |
| `assistant` | Rust SDK | `cargo add assistant` |

---

## TypeScript SDK

### 构造函数

```typescript
import { Assistant } from '@assistant/sdk';

const assistant = new Assistant(options);
```

**参数**：

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `agoraUrl` | `string` | `"ws://localhost:8080/ws"` | Agora 服务器地址 |
| `agentId` | `string` | 自动生成 | Assistant 的 Agent ID |
| `llm` | `LLMConfig` | - | LLM 配置 |
| `llm.provider` | `string` | `"anthropic"` | LLM 提供商 |
| `llm.model` | `string` | `"claude-sonnet-4"` | 模型名称 |
| `llm.temperature` | `number` | `0.7` | 温度参数 |
| `llm.maxTokens` | `number` | `4096` | 最大输出令牌 |
| `profile` | `string` | `"professional"` | 默认助手类型 |
| `profiles` | `Record<string, Profile>` | 内置类型 | 自定义助手类型 |
| `memory` | `MemoryConfig` | - | 记忆配置 |

**示例**：

```typescript
const assistant = new Assistant({
  agoraUrl: 'wss://agora.example.com/ws',
  llm: {
    provider: 'openai',
    model: 'gpt-4',
    temperature: 0.5,
  },
  profile: 'casual',
  memory: {
    session: { storagePath: './data/sessions.db' },
    longTerm: { vectorStore: './data/memories.db' }
  }
});
```

### 连接管理

#### `connect()`

建立与 Agora 的连接。

```typescript
await assistant.connect(): Promise<void>
```

#### `disconnect()`

断开连接。

```typescript
await assistant.disconnect(): Promise<void>
```

#### `isConnected()`

检查连接状态。

```typescript
const connected = assistant.isConnected(): boolean
```

### 对话

#### `chat()`

发送消息并获取响应。

```typescript
const response = await assistant.chat(message, options?): Promise<ChatResponse>
```

**参数**：

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `message` | `string` | - | 用户消息 |
| `options.sessionId` | `string` | 当前会话 | 指定会话 |
| `options.profile` | `string` | 当前类型 | 覆盖助手类型 |
| `options.stream` | `boolean` | `false` | 流式响应 |
| `options.files` | `string[]` | - | 附加文件 |

**返回**：

```typescript
interface ChatResponse {
  text: string;              // 响应文本
  sessionId: string;         // 会话 ID
  messageId: string;         // 消息 ID
  agentCalls?: AgentCall[];  // 调用的 Agent
  tokens?: {                 // Token 使用
    input: number;
    output: number;
  };
}
```

**示例**：

```typescript
// 简单对话
const response = await assistant.chat('如何优化数据库查询？');
console.log(response.text);

// 流式响应
for await (const chunk of await assistant.chat('解释这段代码', { stream: true })) {
  process.stdout.write(chunk.text);
}

// 带文件的对话
const response = await assistant.chat('帮我重构这个文件', {
  files: ['./src/auth.rs']
});
```

#### `chatStream()`

流式对话的便捷方法。

```typescript
for await (const chunk of assistant.chatStream(message, options?)) {
  console.log(chunk.text);  // 增量文本
}
```

### 会话管理

#### `session.create()`

创建新会话。

```typescript
const session = await assistant.session.create(options): Promise<Session>
```

**参数**：

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `title` | `string` | - | 会话标题 |
| `profile` | `string` | 默认类型 | 助手类型 |

**返回**：

```typescript
interface Session {
  id: string;
  title: string;
  profile: string;
  createdAt: Date;
  updatedAt: Date;
  messageCount: number;
}
```

#### `session.get()`

获取会话详情。

```typescript
const session = await assistant.session.get(sessionId): Promise<Session>
```

#### `session.list()`

列出所有会话。

```typescript
const sessions = await assistant.session.list(options?): Promise<Session[]>
```

**参数**：

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `limit` | `number` | `20` | 返回数量 |
| `offset` | `number` | `0` | 偏移量 |

#### `session.delete()`

删除会话。

```typescript
await assistant.session.delete(sessionId): Promise<void>
```

#### `session.history()`

获取会话历史。

```typescript
const messages = await assistant.session.history(sessionId, options?): Promise<Message[]>
```

**参数**：

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `limit` | `number` | `100` | 返回数量 |

**返回**：

```typescript
interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  createdAt: Date;
}
```

### 助手类型

#### `profile.set()`

设置当前助手类型。

```typescript
await assistant.profile.set(profileId): Promise<void>
```

#### `profile.get()`

获取当前助手类型。

```typescript
const profile = assistant.profile.get(): Profile
```

#### `profile.list()`

列出所有助手类型。

```typescript
const profiles = assistant.profile.list(): Profile[]
```

#### `profile.register()`

注册自定义助手类型。

```typescript
await assistant.profile.register(id, profile): Promise<void>
```

**参数**：

```typescript
interface Profile {
  id: string;
  name: string;
  systemPrompt: string;
  temperature?: number;
  tone?: 'professional' | 'casual' | 'creative' | 'neutral';
  responseStyle?: 'structured' | 'conversational' | 'detailed' | 'brief';
}
```

### 记忆

#### `memory.search()`

搜索长期记忆。

```typescript
const memories = await assistant.memory.search(query, options?): Promise<Memory[]>
```

**参数**：

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `query` | `string` | - | 搜索查询 |
| `limit` | `number` | `10` | 返回数量 |

**返回**：

```typescript
interface Memory {
  id: string;
  content: string;
  metadata: {
    type: string;
    importance: string;
    sessionId?: string;
  };
  createdAt: Date;
  distance: number;  // 语义距离
}
```

#### `memory.store()`

手动存储记忆。

```typescript
await assistant.memory.store(content, metadata?): Promise<string>
```

**参数**：

| 选项 | 类型 | 说明 |
|------|------|------|
| `content` | `string` | 记忆内容 |
| `metadata` | `object` | 元数据 |

#### `memory.listSessions()`

列出所有会话。

```typescript
const sessions = await assistant.memory.listSessions(): Promise<Session[]>
```

### Agent 协作

#### `agent.list()`

列出可用的 Agent。

```typescript
const agents = await assistant.agent.list(): Promise<AgentInfo[]>
```

**返回**：

```typescript
interface AgentInfo {
  id: string;
  name: string;
  description?: string;
  capabilities: string[];
  status: 'online' | 'offline';
}
```

#### `agent.call()`

直接调用 Agent。

```typescript
const result = await assistant.agent.call(agentId, task): Promise<AgentResult>
```

**参数**：

| 选项 | 类型 | 说明 |
|------|------|------|
| `agentId` | `string` | Agent ID |
| `task` | `string` | 任务描述 |

**返回**：

```typescript
interface AgentResult {
  agentId: string;
  output: string;
  status: 'success' | 'error';
  error?: string;
  duration: number;
}
```

### 事件

#### `on()`

监听事件。

```typescript
assistant.on(event, handler): void
```

**事件类型**：

| 事件 | 说明 | 数据 |
|------|------|------|
| `connected` | 已连接 Agora | - |
| `disconnected` | 已断开 | - |
| `message` | 收到消息 | `Message` |
| `agent.joined` | Agent 上线 | `AgentInfo` |
| `agent.left` | Agent 离线 | `string` (agentId) |
| `error` | 错误 | `Error` |

**示例**：

```typescript
assistant.on('connected', () => {
  console.log('已连接到 Agora');
});

assistant.on('agent.joined', (agent) => {
  console.log(`${agent.name} 上线了`);
});

assistant.on('error', (error) => {
  console.error('错误:', error.message);
});
```

#### `off()`

取消监听。

```typescript
assistant.off(event, handler?): void
```

---

## Rust SDK

### 构造函数

```rust
use assistant::{Assistant, AssistantConfig, LLMConfig};

let config = AssistantConfig {
    agora_url: "ws://localhost:8080/ws".to_string(),
    llm: LLMConfig {
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(4096),
    },
    profile: "professional".to_string(),
    ..Default::default()
};

let assistant = Assistant::new(config).await?;
```

### 对话

```rust
use assistant::{ChatOptions, ChatResponse};

let response = assistant.chat(
    "如何优化数据库查询？",
    ChatOptions::default()
).await?;

println!("{}", response.text);
```

### 流式对话

```rust
use futures::StreamExt;

let mut stream = assistant.chat_stream(
    "解释这段代码",
    ChatOptions::default()
).await?;

while let Some(chunk) = stream.next().await {
    print!("{}", chunk?.text);
}
```

### 会话管理

```rust
// 创建会话
let session = assistant.session()
    .create("代码审查".to_string())
    .await?;

// 获取会话
let session = assistant.session()
    .get(&session_id)
    .await?;

// 列出会话
let sessions = assistant.session()
    .list(20)
    .await?;

// 删除会话
assistant.session()
    .delete(&session_id)
    .await?;
```

### 类型定义

```rust
pub struct Assistant {
    config: AssistantConfig,
    agora: AgoraClient,
    memory: MemoryManager,
    profile_manager: ProfileManager,
}

pub struct ChatResponse {
    pub text: String,
    pub session_id: String,
    pub message_id: String,
    pub agent_calls: Vec<AgentCall>,
    pub tokens: Option<TokenUsage>,
}

pub struct Session {
    pub id: String,
    pub title: String,
    pub profile: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}

pub struct Memory {
    pub id: String,
    pub content: String,
    pub metadata: Metadata,
    pub distance: f32,
}

pub enum AssistantError {
    AgoraConnectionFailed,
    LLMError(String),
    SessionNotFound(String),
    MemoryError(String),
}
```

---

## CLI API

### 命令行接口

```bash
# 启动交互式对话
assistant [OPTIONS]

# 选项：
#   --profile <TYPE>      助手类型 [default: professional]
#   --agora <URL>         Agora 服务器 [default: ws://localhost:8080/ws]
#   --model <MODEL>       LLM 模型
#   --session <ID>        指定会话
#   --history <PATH>      历史存储路径
#   -h, --help            显示帮助
```

### 交互式命令

在交互模式中：

| 命令 | 说明 |
|------|------|
| `/profile <type>` | 切换助手类型 |
| `/session new <title>` | 创建新会话 |
| `/session list` | 列出会话 |
| `/session switch <id>` | 切换会话 |
| `/agent list` | 列出可用 Agent |
| `/memory search <query>` | 搜索记忆 |
| `/clear` | 清空当前会话上下文 |
| `/exit` | 退出 |

### 非交互式使用

```bash
# 单次查询
echo "如何优化代码？" | assistant

# 从文件读取
cat question.txt | assistant --output answer.txt

# 管道使用
cat code.rs | assistant "审查这段代码"
```

---

## REST API

Assistant Server 提供 REST API（如果启用）：

### POST /chat

发送消息。

```bash
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "如何优化数据库？",
    "session_id": "sess_123",
    "profile": "professional"
  }'
```

### GET /sessions

列出会话。

```bash
curl http://localhost:3000/api/sessions?limit=20
```

### POST /sessions

创建会话。

```bash
curl -X POST http://localhost:3000/api/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "title": "代码审查",
    "profile": "professional"
  }'
```

### GET /sessions/:id

获取会话详情。

```bash
curl http://localhost:3000/api/sessions/sess_123
```

### DELETE /sessions/:id

删除会话。

```bash
curl -X DELETE http://localhost:3000/api/sessions/sess_123
```

### GET /agents

列出可用 Agent。

```bash
curl http://localhost:3000/api/agents
```

### GET /memory/search

搜索记忆。

```bash
curl "http://localhost:3000/api/memory/search?q=数据库&limit=5"
```
