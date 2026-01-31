# 记忆 Agent

带有记忆功能的 Agent，可以根据历史对话和存储的信息进行智能交流。

## 记忆层次

```
┌─────────────────────────────────────────────────────────┐
│                    Memory System                         │
├─────────────┬─────────────────┬─────────────────────────┤
│   短期记忆   │     长期记忆     │       语义记忆          │
│  (Session)   │    (Profile)    │      (Vector)           │
│             │                 │                         │
│  - 对话历史  │  - 用户偏好     │  - 向量嵌入             │
│  - 临时状态  │  - 个人信息     │  - 相似度搜索           │
│  - 当前上下文│  - 学习结果     │  - RAG 检索            │
└─────────────┴─────────────────┴─────────────────────────┘
```

## 核心 Trait

```rust
/// 记忆存储 trait
#[async_trait]
pub trait Memory: Send + Sync {
    /// 存储键值对
    async fn store(&self, key: &str, value: &str) -> Result<(), MemoryError>;

    /// 获取值
    async fn get(&self, key: &str) -> Result<Option<String>, MemoryError>;

    /// 搜索相关记忆
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>, MemoryError>;

    /// 删除记忆
    async fn delete(&self, key: &str) -> Result<bool, MemoryError>;
}

/// 记忆项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub embedding: Option<Vec<f32>>,
    pub timestamp: i64,
    pub importance: f32,  // 0-1，用于记忆管理
}

/// 记忆错误
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("NotFound: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

## 短期记忆 (Session Memory)

```rust
/// 会话记忆 - 存储对话历史
pub struct SessionMemory {
    user_id: String,
    session_id: String,
    messages: Arc<Mutex<Vec<Message>>>,
    max_messages: usize,
}

impl SessionMemory {
    pub fn new(user_id: String, session_id: String, max_messages: usize) -> Self {
        Self {
            user_id,
            session_id,
            messages: Arc::new(Mutex::new(Vec::with_capacity(max_messages))),
            max_messages,
        }
    }

    /// 添加消息
    pub async fn add(&self, message: Message) {
        let mut messages = self.messages.lock().await;
        messages.push(message);

        // 超过容量时压缩历史
        if messages.len() > self.max_messages {
            self.compress_history(&mut messages).await;
        }
    }

    /// 压缩历史（保留最近的消息）
    async fn compress_history(&self, messages: &mut Vec<Message>) {
        let keep = self.max_messages / 2;
        let summary = self.summarize(&messages[..messages.len() - keep]).await;

        *messages = vec![
            Message::System(format!("对话摘要: {}", summary)),
        ];
        messages.extend(messages.drain(messages.len() - keep..));
    }

    /// 总结对话
    async fn summarize(&self, messages: &[Message]) -> String {
        let text: String = messages.iter()
            .filter_map(|m| match m {
                Message::User(s) | Message::Assistant(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<&str>>()
            .join("\n");

        // 调用 LLM 总结
        // ...
        text.chars().take(100).collect()
    }

    /// 获取历史
    pub async fn history(&self) -> Vec<Message> {
        self.messages.lock().await.clone()
    }
}

#[async_trait]
impl Memory for SessionMemory {
    async fn store(&self, key: &str, value: &str) -> Result<(), MemoryError> {
        self.add(Message::System(format!("{}: {}", key, value)));
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, MemoryError> {
        let messages = self.history().await;
        // 简单实现：在历史中搜索
        for msg in messages {
            if let Message::System(content) = msg {
                if content.starts_with(&format!("{}:", key)) {
                    return Ok(Some(content[key.len() + 1..].to_string()));
                }
            }
        }
        Ok(None)
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>, MemoryError> {
        let messages = self.history().await;
        let mut results = Vec::new();

        for msg in messages.iter().filter(|m| matches!(m, Message::User(_) | Message::Assistant(_))) {
            let content = match msg {
                Message::User(s) => s,
                Message::Assistant(s) => s,
                _ => continue,
            };

            if content.contains(query) {
                results.push(MemoryItem {
                    key: uuid::Uuid::new_v4().to_string(),
                    value: content.clone(),
                    embedding: None,
                    timestamp: chrono::Utc::now().timestamp(),
                    importance: 0.5,
                });
            }

            if results.len() >= limit {
                break;
            }
        }

        Ok(results)
    }

    async fn delete(&self, key: &str) -> Result<bool, MemoryError> {
        // 实现...
        Ok(false)
    }
}
```

## 长期记忆 (Profile Memory)

```rust
/// 用户档案 - 长期记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,

    /// 基本信息
    pub name: Option<String>,
    pub nickname: Option<String>,
    pub timezone: Option<String>,

    /// 偏好
    pub preferences: HashMap<String, String>,

    /// 学习到的信息
    pub facts: Vec<Fact>,

    /// 统计信息
    pub stats: UserStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub key: String,
    pub value: String,
    pub confidence: f32,  // 0-1
    pub last_confirmed: i64,
    pub source: FactSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactSource {
    UserProvided,
    Inferred,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub total_conversations: usize,
    pub total_messages: usize,
    pub first_contact: i64,
    pub last_contact: i64,
}

/// 长期记忆存储
pub struct ProfileMemory<P: ProfileStore> {
    store: P,
    llm: Arc<dyn LlmClient>,
}

#[async_trait]
pub trait ProfileStore: Send + Sync {
    async fn load(&self, user_id: &str) -> Result<Option<UserProfile>, StoreError>;
    async fn save(&self, profile: &UserProfile) -> Result<(), StoreError>;
    async fn update<F>(&self, user_id: &str, f: F) -> Result<(), StoreError>
    where
        F: FnOnce(&mut UserProfile) + Send;
}

impl<P: ProfileStore> ProfileMemory<P> {
    pub fn new(store: P, llm: Arc<dyn LlmClient>) -> Self {
        Self { store, llm }
    }

    /// 获取或创建用户档案
    pub async fn get_or_create(&self, user_id: &str) -> Result<UserProfile, MemoryError> {
        Ok(match self.store.load(user_id).await? {
            Some(profile) => profile,
            None => UserProfile {
                user_id: user_id.to_string(),
                name: None,
                nickname: None,
                timezone: None,
                preferences: HashMap::new(),
                facts: Vec::new(),
                stats: UserStats {
                    total_conversations: 0,
                    total_messages: 0,
                    first_contact: chrono::Utc::now().timestamp(),
                    last_contact: chrono::Utc::now().timestamp(),
                },
            },
        })
    }

    /// 保存档案
    pub async fn save(&self, profile: &UserProfile) -> Result<(), MemoryError> {
        self.store.save(profile).await.map_err(Into::into)
    }

    /// 更新档案
    pub async fn update<F>(&self, user_id: &str, f: F) -> Result<(), MemoryError>
    where
        F: FnOnce(&mut UserProfile) + Send,
    {
        self.store.update(user_id, f).await.map_err(Into::into)
    }

    /// 提取并保存信息
    pub async fn extract_and_store(
        &self,
        user_id: &str,
        text: &str,
    ) -> Result<(), MemoryError> {
        // 使用 LLM 提取结构化信息
        let facts = self.extract_facts(text).await?;

        // 更新档案
        self.update(user_id, |profile| {
            for fact in facts {
                // 更新或添加事实
                if let Some(existing) = profile.facts.iter_mut().find(|f| f.key == fact.key) {
                    if fact.confidence > existing.confidence {
                        *existing = fact;
                    }
                } else {
                    profile.facts.push(fact);
                }
            }
        }).await
    }

    /// 从文本提取事实
    async fn extract_facts(&self, text: &str) -> Result<Vec<Fact>, MemoryError> {
        let prompt = format!(
            r#"从以下对话中提取用户的结构化信息，返回 JSON:

对话: {}

返回格式:
{{
    "facts": [
        {{"key": "name", "value": "张三", "confidence": 0.95}},
        {{"key": "location", "value": "北京", "confidence": 0.8}}
    ]
}}
"#,
            text
        );

        let response = self.llm.complete(&prompt).await?;

        // 解析 JSON
        #[derive(Deserialize)]
        struct Extracted {
            facts: Vec<ExtractedFact>,
        }

        #[derive(Deserialize)]
        struct ExtractedFact {
            key: String,
            value: String,
            confidence: f32,
        }

        let parsed: Extracted = serde_json::from_str(&response)
            .map_err(|_| MemoryError::Serialization("Invalid JSON".into()))?;

        Ok(parsed.facts.into_iter().map(|f| Fact {
            key: f.key,
            value: f.value,
            confidence: f.confidence,
            last_confirmed: chrono::Utc::now().timestamp(),
            source: FactSource::Inferred,
        }).collect())
    }
}
```

## 语义记忆 (Vector Memory)

```rust
/// 向量记忆 - 基于语义相似度检索
pub struct VectorMemory<E: Embedder, S: VectorStore> {
    embedder: E,
    store: S,
}

impl<E: Embedder, S: VectorStore> VectorMemory<E, S> {
    pub fn new(embedder: E, store: S) -> Self {
        Self { embedder, store }
    }

    /// 存储记忆并自动编码
    pub async fn store(&self, value: String) -> Result<String, MemoryError> {
        let embedding = self.embedder.embed(&value).await?;
        let id = uuid::Uuid::new_v4().to_string();

        let item = MemoryItem {
            key: id.clone(),
            value,
            embedding: Some(embedding),
            timestamp: chrono::Utc::now().timestamp(),
            importance: 0.5,
        };

        self.store.insert(item).await?;
        Ok(id)
    }

    /// 语义搜索
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>, MemoryError> {
        let query_embedding = self.embedder.embed(query).await?;
        self.store.search(&query_embedding, limit).await
    }

    /// 相关记忆检索 (RAG)
    pub async fn retrieve_relevant(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<String, MemoryError> {
        let items = self.search(query, limit).await?;

        let context = items.iter()
            .map(|item| item.value.as_str())
            .collect::<Vec<&str>>()
            .join("\n\n");

        Ok(context)
    }
}

/// 嵌入器 trait
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>;
}

/// 向量存储 trait
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn insert(&self, item: MemoryItem) -> Result<(), StoreError>;
    async fn search(&self, embedding: &[f32], limit: usize) -> Result<Vec<MemoryItem>, StoreError>;
}

/// 简单实现：内存向量存储
pub struct InMemoryVectorStore {
    items: Arc<RwLock<Vec<MemoryItem>>>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn insert(&self, item: MemoryItem) -> Result<(), StoreError> {
        self.items.write().await.push(item);
        Ok(())
    }

    async fn search(&self, embedding: &[f32], limit: usize) -> Result<Vec<MemoryItem>, StoreError> {
        let items = self.items.read().await;

        let mut results: Vec<_> = items.iter()
            .filter_map(|item| {
                item.embedding.as_ref().map(|emb| {
                    (item, cosine_similarity(embedding, emb))
                })
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::ordering::Ordering::Equal));

        Ok(results.into_iter()
            .take(limit)
            .map(|(item, _score)| item.clone())
            .collect())
    }
}

/// 余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b + 1e-8)
}
```

## Memory Agent

```rust
/// 带记忆的 Agent
pub struct MemoryAgent<L, S, P, V> {
    llm: L,
    session: S,
    profile: P,
    vector: V,
    system_prompt: String,
}

impl<L, S, P, V> MemoryAgent<L, S, P, V>
where
    L: LlmClient,
    S: SessionMemory,
    P: ProfileMemory,
    V: VectorMemory,
{
    pub fn new(llm: L, session: S, profile: P, vector: V) -> Self {
        Self {
            llm,
            session,
            profile,
            vector,
            system_prompt: "你是一个有帮助的助手，拥有记忆功能。".to_string(),
        }
    }

    /// 获取完整的上下文
    async fn build_context(&self, user_id: &str, query: &str) -> Result<String, AgentError> {
        let mut context_parts = Vec::new();

        // 1. 用户档案信息
        if let Ok(profile) = self.profile.get_or_create(user_id).await {
            if let Some(name) = profile.name {
                context_parts.push(format!("用户姓名: {}", name));
            }
            if !profile.facts.is_empty() {
                let facts: Vec<String> = profile.facts.iter()
                    .filter(|f| f.confidence > 0.7)
                    .map(|f| format!("- {}: {}", f.key, f.value))
                    .collect();
                context_parts.push(format!("已知信息:\n{}", facts.join("\n")));
            }
        }

        // 2. 语义相关记忆
        if let Ok(relevant) = self.vector.retrieve_relevant(query, 3).await {
            if !relevant.is_empty() {
                context_parts.push(format!("相关记忆:\n{}", relevant));
            }
        }

        Ok(context_parts.join("\n\n"))
    }

    /// 聊天
    pub async fn chat(
        &self,
        user_id: &str,
        message: &str,
    ) -> Result<String, AgentError> {
        // 1. 构建上下文
        let memory_context = self.build_context(user_id, message).await?;

        // 2. 获取会话历史
        let history = self.session.history().await;

        // 3. 构建消息
        let mut messages = vec![
            Message::System(format!(
                "{}\n\n{}",
                self.system_prompt,
                memory_context
            )),
        ];
        messages.extend(history);
        messages.push(Message::User(message.to_string()));

        // 4. 调用 LLM
        let response = self.llm.chat(&messages).await?;

        // 5. 保存到会话记忆
        self.session.add(Message::User(message.to_string())).await;
        self.session.add(Message::Assistant(response.clone())).await;

        // 6. 提取并保存到长期记忆
        let _ = self.profile.extract_and_store(user_id, message).await;

        // 7. 保存到语义记忆
        let _ = self.vector.store(message.to_string()).await;

        Ok(response)
    }

    /// 显式记忆
    pub async fn remember(
        &self,
        user_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), MemoryError> {
        // 保存到长期记忆
        self.profile.update(user_id, |profile| {
            profile.facts.push(Fact {
                key: key.to_string(),
                value: value.to_string(),
                confidence: 1.0,
                last_confirmed: chrono::Utc::now().timestamp(),
                source: FactSource::UserProvided,
            });
        }).await?;

        // 同时保存到语义记忆
        self.vector.store(format!("{}: {}", key, value)).await?;

        Ok(())
    }

    /// 遗忘
    pub async fn forget(&self, user_id: &str, key: &str) -> Result<(), MemoryError> {
        self.profile.update(user_id, |profile| {
            profile.facts.retain(|f| f.key != key);
        }).await
    }
}
```

## 使用示例

```rust
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建组件
    let llm = Arc::new(OpenAIClient::new(LLMConfig::default()));

    // 短期记忆
    let session = SessionMemory::new("user-123".to_string(), "session-1".to_string(), 100);

    // 长期记忆 (使用 Redis)
    let profile_store = RedisProfileStore::new("redis://localhost").await?;
    let profile = ProfileMemory::new(profile_store, llm.clone());

    // 语义记忆 (使用 Qdrant)
    let embedder = OpenAIEmbedder::new(std::env::var("OPENAI_API_KEY")?);
    let vector_store = QdrantStore::new("http://localhost:6333").await?;
    let vector = VectorMemory::new(embedder, vector_store);

    // 创建 Agent
    let agent = MemoryAgent::new(llm, session, profile, vector);

    // 第一次对话
    let response = agent.chat("user-123", "我叫小明，住在北京").await?;
    println!("{}", response);  // "你好小明！很高兴认识你。"

    // Agent 记住了信息
    agent.remember("user-123", "name", "小明").await?;
    agent.remember("user-123", "location", "北京").await?;

    // 后续对话 - Agent 会使用记忆
    let response = agent.chat("user-123", "我叫什么名字？").await?;
    println!("{}", response);  // "你叫小明。"

    let response = agent.chat("user-123", "我现在在哪里？").await?;
    println!("{}", response);  // "你住在北京。"

    // 相关记忆检索
    let response = agent.chat("user-123", "介绍一下北京").await?;
    println!("{}", response);  // 会检索到之前关于北京的对话

    Ok(())
}
```

## 记忆管理策略

```rust
/// 记忆重要性评估
pub struct MemoryScorer {
    llm: Arc<dyn LlmClient>,
}

impl MemoryScorer {
    pub async fn score(&self, memory: &str) -> Result<f32, MemoryError> {
        let prompt = format!(
            "评估以下记忆的重要性 (0-1):\n{}\n\n只返回数字。",
            memory
        );

        let response = self.llm.complete(&prompt).await?;
        response.parse::<f32>()
            .map_err(|_| MemoryError::Serialization("Invalid score".into()))
    }
}

/// 记忆清理 - 删除不重要的旧记忆
pub struct MemoryCleaner<V> {
    store: V,
    scorer: MemoryScorer,
    max_age_days: i64,
    min_importance: f32,
}

impl<V: VectorStore> MemoryCleaner<V> {
    pub async fn cleanup(&self, user_id: &str) -> Result<usize, MemoryError> {
        // 获取所有记忆
        // 评估重要性
        // 删除旧且不重要的记忆
        Ok(0)
    }
}
```
