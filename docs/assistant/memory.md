# 记忆管理

> Assistant 的多层记忆系统，实现智能的对话历史管理和知识提取

## 概述

Assistant 维护一个三层记忆系统：

```
┌─────────────────────────────────────────────────────────────────┐
│                        记忆层级                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  长期记忆 (Long-term Memory)                                     │
│  - 跨会话的知识提取和存储                                        │
│  - 向量数据库实现语义检索                                        │
│  - 用户偏好、项目知识、常见问题                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ "用户偏好使用 Rust", "项目使用 PostgreSQL",              │    │
│  │  "用户喜欢简洁的代码风格"                                 │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              ↑ 提取
                              │
┌─────────────────────────────────────────────────────────────────┐
│  会话记忆 (Session Memory)                                        │
│  - 历史对话记录的持久化存储                                      │
│  - SQLite 实现                                                  │
│  - 支持跨时间回溯                                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Session #1: "如何部署应用" → "使用 Docker"               │    │
│  │ Session #2: "数据库设计讨论"                            │    │
│  │ Session #3: "API 性能优化"                              │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              ↑ 保存
                              │
┌─────────────────────────────────────────────────────────────────┐
│  短期记忆 (Short-term Memory)                                     │
│  - 当前会话的上下文窗口                                          │
│  - 内存存储，快速访问                                            │
│  - 自动管理上下文长度                                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ [当前对话的最近 N 条消息]                                │    │
│  │ User: "那怎么优化查询速度？"                             │    │
│  │ Assistant: "可以添加索引..."                            │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## 短期记忆

当前会话的活跃上下文，存储在内存中。

```rust
pub struct ShortTermMemory {
    max_messages: usize,
    max_tokens: usize,
    messages: Vec<Message>,
    context_summary: Option<String>,
}

impl ShortTermMemory {
    pub fn new(max_messages: usize, max_tokens: usize) -> Self;

    pub fn append(&mut self, message: Message) -> Result<()> {
        self.messages.push(message);
        self.trim_if_needed();
        Ok(())
    }

    pub fn get_context(&self) -> Vec<&Message> {
        // 返回适合 LLM 的上下文
        if self.estimate_tokens() <= self.max_tokens {
            return self.messages.iter().collect();
        }

        // 超出限制，使用压缩策略
        self.get_compressed_context()
    }

    fn trim_if_needed(&mut self) {
        // 策略 1: 删除最旧的消息
        while self.estimate_tokens() > self.max_tokens {
            self.messages.remove(0);
        }

        // 策略 2: 保留最近 N 条
        if self.messages.len() > self.max_messages {
            let remove_count = self.messages.len() - self.max_messages;
            for _ in 0..remove_count {
                self.messages.remove(0);
            }
        }
    }

    fn get_compressed_context(&self) -> Vec<&Message> {
        // 保留最近的消息 + 压缩的旧消息摘要
        let mut result = Vec::new();

        if let Some(summary) = &self.context_summary {
            result.push(Message::system(summary.clone()));
        }

        let recent_count = self.max_messages / 2;
        let start = self.messages.len().saturating_sub(recent_count);
        result.extend(self.messages[start..].iter());

        result
    }
}
```

### 上下文压缩

当对话历史过长时，自动压缩旧消息：

```rust
pub struct ContextCompressor {
    llm: LLMClient,
}

impl ContextCompressor {
    pub async fn compress(&self, messages: &[Message]) -> String {
        let prompt = format!(
            "请将以下对话摘要为简短的上下文说明，保留关键信息：\n\n{}",
            messages.iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n")
        );

        self.llm.complete(&prompt).await
    }
}
```

## 会话记忆

持久化的历史对话存储。

### 数据模型

```sql
-- 会话表
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    profile TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata JSON
);

-- 消息表
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,  -- 'user' | 'assistant' | 'system'
    content TEXT NOT NULL,
    tokens INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- 向量表（长期记忆）
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    content TEXT NOT NULL,
    embedding BLOB NOT NULL,
    metadata JSON,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- 索引
CREATE INDEX idx_messages_session ON messages(session_id);
CREATE INDEX idx_messages_created ON messages(created_at);
CREATE INDEX idx_sessions_updated ON sessions(updated_at);
```

### 操作接口

```rust
pub struct SessionStore {
    db: SqlitePool,
}

impl SessionStore {
    // 创建会话
    pub async fn create_session(&self, title: String, profile: String) -> Result<Session>;

    // 获取会话
    pub async fn get_session(&self, id: &str) -> Result<Option<Session>>;

    // 列出会话
    pub async fn list_sessions(&self, limit: usize) -> Result<Vec<Session>>;

    // 添加消息
    pub async fn append_message(&self, session_id: &str, message: &Message) -> Result<()>;

    // 获取会话历史
    pub async fn get_history(&self, session_id: &str, limit: usize) -> Result<Vec<Message>>;

    // 搜索会话
    pub async fn search_sessions(&self, query: &str) -> Result<Vec<Session>>;

    // 删除会话
    pub async fn delete_session(&self, id: &str) -> Result<()>;
}
```

## 长期记忆

跨会话的知识提取和语义检索。

### 知识提取

自动从对话中提取重要信息：

```rust
pub struct KnowledgeExtractor {
    llm: LLMClient,
    extractor_prompt: String,
}

impl KnowledgeExtractor {
    pub async fn extract(&self, messages: &[Message]) -> Vec<Memory> {
        let prompt = format!(
            "从以下对话中提取值得长期记忆的信息：\n\n{}\n\n\
            提取：用户偏好、项目信息、重要决策、常见问题等。",
            self.format_messages(messages)
        );

        let response = self.llm.complete(&prompt).await;
        self.parse_memories(response)
    }

    fn parse_memories(&self, response: String) -> Vec<Memory> {
        // 解析 LLM 返回的结构化数据
        // 格式示例：
        // - 用户偏好使用 Rust 进行后端开发
        // - 项目使用 PostgreSQL 作为主数据库
        // ...
    }
}
```

### 向量存储

使用向量数据库实现语义检索：

```rust
pub struct VectorStore {
    db: SqlitePool,
    embedding_model: EmbeddingModel,
}

impl VectorStore {
    // 存储记忆
    pub async fn store(&self, content: String, metadata: Metadata) -> Result<String> {
        let embedding = self.embedding_model.embed(&content).await?;
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO memories (id, content, embedding, metadata) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&content)
        .bind(serialize_embedding(&embedding))
        .bind(serialize_metadata(&metadata))
        .execute(&self.db)
        .await?;

        Ok(id)
    }

    // 语义搜索
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let query_embedding = self.embedding_model.embed(query).await?;

        let memories = sqlx::query_as::<_, Memory>(
            "SELECT id, content, metadata,
                    vector_distance(embedding, ?) as distance
             FROM memories
             ORDER BY distance ASC
             LIMIT ?"
        )
        .bind(serialize_embedding(&query_embedding))
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        Ok(memories)
    }

    // 获取相关上下文
    pub async fn get_context(&self, query: &str, limit: usize) -> Result<String> {
        let memories = self.search(query, limit).await?;

        Ok(memories
            .iter()
            .map(|m| format!("- {}", m.content))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}
```

### 记忆类型

| 类型 | 说明 | 示例 |
|------|------|------|
| **用户偏好** | 用户的习惯和偏好 | "用户偏好使用 Rust" |
| **项目知识** | 项目相关的技术决策 | "使用 PostgreSQL 作为主数据库" |
| **常见问题** | 频繁出现的问题和解答 | "部署流程使用 Docker" |
| **代码模式** | 项目中常见的代码模式 | "错误处理使用 Result 类型" |
| **重要决策** | 重要的架构或设计决策 | "选择 WebSocket 而非轮询" |

### 自动记忆管理

```rust
pub struct MemoryManager {
    short_term: ShortTermMemory,
    session_store: SessionStore,
    long_term: VectorStore,
    extractor: KnowledgeExtractor,
    config: MemoryConfig,
}

impl MemoryManager {
    pub async fn on_message(&mut self, message: &Message) -> Result<()> {
        // 1. 添加到短期记忆
        self.short_term.append(message.clone())?;

        // 2. 持久化到会话存储
        self.session_store.append_message(&self.session_id, message).await?;

        // 3. 检查是否需要提取长期记忆
        if self.should_extract() {
            self.extract_and_store().await?;
        }

        Ok(())
    }

    fn should_extract(&self) -> bool {
        // 每 N 条消息提取一次
        // 或者检测到"重要"内容时提取
        self.short_term.message_count() % self.config.extract_interval == 0
    }

    async fn extract_and_store(&mut self) -> Result<()> {
        let history = self.short_term.get_messages();
        let memories = self.extractor.extract(&history).await?;

        for memory in memories {
            self.long_term.store(memory.content, memory.metadata).await?;
        }

        Ok(())
    }

    pub async fn get_relevant_context(&self, query: &str) -> Result<String> {
        self.long_term.get_context(query, 5).await
    }
}
```

## 配置

```json
{
  "memory": {
    "short_term": {
      "max_messages": 50,
      "max_tokens": 8000,
      "compress_threshold": 0.8
    },
    "session": {
      "storage_path": "./data/sessions.db",
      "max_sessions": 1000,
      "max_messages_per_session": 10000
    },
    "long_term": {
      "vector_store": "./data/memories.db",
      "embedding_model": "text-embedding-3-small",
      "extract_interval": 20,
      "max_memories": 10000,
      "search_threshold": 0.7
    }
  }
}
```

## API 使用

```typescript
import { Assistant } from '@assistant/sdk';

const assistant = new Assistant({
  memory: {
    shortTerm: { maxMessages: 50, maxTokens: 8000 },
    session: { storagePath: './data/sessions.db' },
    longTerm: { vectorStore: './data/memories.db' }
  }
});

// 检索相关记忆
const context = await assistant.memory.search('用户喜欢的编程语言');
console.log(context);
// ["用户偏好使用 Rust 进行后端开发", "用户也熟悉 TypeScript"]

// 手动添加记忆
await assistant.memory.store('项目使用 PostgreSQL 作为主数据库', {
  type: 'project_knowledge',
  importance: 'high'
});

// 查看会话历史
const sessions = await assistant.memory.listSessions();
for (const session of sessions) {
  console.log(`${session.title}: ${session.message_count} 条消息`);
}
```
