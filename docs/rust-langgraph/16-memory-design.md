# 记忆设计（以 Python LangGraph 为蓝本，Rust 代码风格）

以 **Python LangGraph** 的 Checkpoint / Store 体系为蓝本，给出 **rust-langgraph** 侧的记忆设计：类型、trait、模块划分与集成方式。本文为设计文档，实现时按任务表推进。

**前置阅读**：[10-memory-deep-dive](../langgraph-agent/10-memory-deep-dive.md)（LangGraph 记忆深度）、[MEMORY_VS_LANGGRAPH_STORE.md](MEMORY_VS_LANGGRAPH_STORE.md)、[10-reducer-design.md](10-reducer-design.md)。

---

## 1. 蓝本：Python LangGraph 记忆架构

LangGraph 将记忆分为两层：

| 类型     | 机制 | 作用域 | 典型用途 |
|----------|------|--------|----------|
| **短期记忆** | Checkpointer + `thread_id` + state 的 channel（如 `messages`） | 单会话（thread） | 多轮对话历史、图状态快照、时间旅行 |
| **长期记忆** | **Store**（如 `InMemoryStore`）+ namespace | 跨 thread / 跨会话 | 用户偏好、长期档案、跨会话检索 |

- **Checkpointer**：按 `thread_id` 存图状态快照；每次 invoke 时 `config["configurable"]["thread_id"]` 指定会话；可选 `checkpoint_id` 从某历史快照恢复。
- **Store**：按 namespace（如 `(user_id, "memories")`）做 put/get/search，与 thread 解耦；可配语义索引做 `search(namespace, query=..., limit=...)`。

本文在 Rust 侧对齐上述概念，用 trait + 具体类型 + 明确模块划分表达。

---

## 2. 设计目标与原则

- **对齐 Python 语义**：thread、checkpoint、channel_values、channel_versions、Store namespace 等概念一一对应。
- **Rust 风格**：trait 抽象、泛型约束、显式错误类型、异步 API；类型一个文件、测试独立；注释英文。
- **分阶段落地**：先短期记忆（Checkpointer + 图集成），再长期记忆（Store）；序列化与 reducer 复用 [10-reducer-design](10-reducer-design.md)。**持久化**：短期记忆用 SqliteSaver（3.6），长期记忆用 SqliteStore（5.2.2），与 Python `langgraph-checkpoint-sqlite` 对齐。
- **向量/语义检索**：本方案采用**内存向量数据库**（进程内存储 embedding，余弦相似度检索），不依赖 Qdrant 等独立向量服务；适合开发、单机与小规模部署，进程退出不持久化。

---

## 3. 短期记忆：Thread、Config、Checkpoint

### 3.1 调用配置（RunnableConfig）

与 Python 的 `config["configurable"]` 对应，invoke 时由调用方传入，用于标识 thread 与可选恢复点。

```rust
/// Config for a single invoke. Identifies the thread and optional checkpoint.
/// Aligns with LangGraph's config["configurable"] (thread_id, checkpoint_id, checkpoint_ns).
#[derive(Debug, Clone, Default)]
pub struct RunnableConfig {
    /// Unique id for this conversation/thread. Required when using a checkpointer.
    pub thread_id: Option<String>,
    /// If set, load state from this checkpoint instead of the latest (time travel / branch).
    pub checkpoint_id: Option<String>,
    /// Optional namespace for checkpoints (e.g. subgraph). Default is empty.
    pub checkpoint_ns: String,
    /// Optional user id; used by Store for cross-thread memory (namespace).
    pub user_id: Option<String>,
}
```

**交互**：`CompiledStateGraph::invoke(state, config)` 在传入 checkpointer 时，用 `config.thread_id` 决定读写哪个 thread；`checkpoint_id` 指定从哪一版恢复。

### 3.2 Checkpoint 数据结构

与 Python 底层 Checkpoint（v, id, ts, channel_values, channel_versions, versions_seen）对齐。Rust 侧 state 为强类型 `S`，序列化后以字节存；元数据单独字段。

```rust
/// Metadata for a single checkpoint (source, step, created_at).
/// Aligns with LangGraph checkpoint metadata.
#[derive(Debug, Clone)]
pub struct CheckpointMetadata {
    pub source: CheckpointSource,
    pub step: u64,
    pub created_at: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub enum CheckpointSource {
    Input,
    Loop,
    Update,
    Fork,
}

/// One checkpoint: serialized state + channel versions + id/ts.
/// Stored by Checkpointer keyed by (thread_id, checkpoint_ns, checkpoint_id).
pub struct Checkpoint<S> {
    pub id: String,
    pub ts: String,
    pub channel_values: S,
    pub channel_versions: std::collections::HashMap<String, u64>,
    pub metadata: CheckpointMetadata,
}
```

- **channel_values**：此处用泛型 `S` 表示「已反序列化的图状态」；持久化时由 `Serializer` 将 `S` 转为 `Vec<u8>` 写入。
- **channel_versions**：channel 名 → 版本号，用于 reducer 与增量合并（与 [10-reducer-design](10-reducer-design.md) 的 merge 配合）。

若先做最简实现，可暂不落 `versions_seen`，仅保留 `id/ts/channel_values/metadata`。

### 3.3 Checkpointer trait

与 Python `BaseCheckpointSaver` 对齐：put、get_tuple、list。

```rust
/// Error type for checkpoint operations.
#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("thread_id required")]
    ThreadIdRequired,
    #[error("serialization: {0}")]
    Serialization(String),
    #[error("storage: {0}")]
    Storage(String),
    #[error("not found: {0}")]
    NotFound(String),
}

/// Saves and loads checkpoints by (thread_id, checkpoint_ns, checkpoint_id).
/// Aligns with LangGraph BaseCheckpointSaver (put, get_tuple, list).
#[async_trait]
pub trait Checkpointer<S>: Send + Sync
where
    S: Clone + Send + Sync + 'static,
{
    /// Persist a checkpoint for the thread and config. Returns the checkpoint id used.
    async fn put(
        &self,
        config: &RunnableConfig,
        checkpoint: &Checkpoint<S>,
    ) -> Result<String, CheckpointError>;

    /// Load the latest checkpoint for the thread (or the one given by config.checkpoint_id).
    async fn get_tuple(
        &self,
        config: &RunnableConfig,
    ) -> Result<Option<(Checkpoint<S>, CheckpointMetadata)>, CheckpointError>;

    /// List checkpoint ids for the thread (e.g. for get_state_history, time travel).
    async fn list(
        &self,
        config: &RunnableConfig,
        limit: Option<usize>,
        before: Option<&str>,
        after: Option<&str>,
    ) -> Result<Vec<CheckpointListItem>, CheckpointError>;
}

#[derive(Debug, Clone)]
pub struct CheckpointListItem {
    pub checkpoint_id: String,
    pub metadata: CheckpointMetadata,
}
```

- **get_tuple**：若 `config.checkpoint_id` 有值则取该 id，否则取该 thread 最新一条。
- **list**：用于历史列表、时间旅行 UI；before/after 可为上一 checkpoint 的 id 或时间边界，按实现而定。

### 3.4 MemorySaver（进程内实现）

与 Python `MemorySaver` / `InMemorySaver` 对应：进程内字典，不落盘。

```rust
/// In-memory checkpointer. Key: (thread_id, checkpoint_ns, checkpoint_id).
/// Aligns with LangGraph MemorySaver / InMemorySaver. Not persistent; for dev and tests.
pub struct MemorySaver<S> {
    inner: std::sync::Arc<tokio::sync::RwLock<MemorySaverInner<S>>>,
}

struct MemorySaverInner<S> {
    by_thread: std::collections::HashMap<String, Vec<(String, Checkpoint<S>)>>,
    next_id: u64,
}
```

- 键可用 `format!("{}:{}", config.thread_id.as_deref().unwrap_or(""), config.checkpoint_ns)`，同一 thread 下按 `next_id` 或时间序存一列 checkpoint；`get_tuple` 取最新或指定 `checkpoint_id`。
- `S` 需 `Serialize + DeserializeOwned`，MemorySaver 内部可存 `Checkpoint<S>` 或序列化后的 `Vec<u8>`；若存 `Checkpoint<S>` 则无需在 MemorySaver 内再调 Serializer，仅对外与「图 state」类型一致即可。

### 3.5 Serializer

与 Python SerializerProtocol / JsonPlusSerializer 对应：state → bytes，bytes → state。

```rust
/// Serializes and deserializes state for checkpoint storage.
pub trait Serializer<S>: Send + Sync
where
    S: Clone + Send + Sync + 'static,
{
    fn serialize(&self, state: &S) -> Result<Vec<u8>, CheckpointError>;
    fn deserialize(&self, bytes: &[u8]) -> Result<S, CheckpointError>;
}
```

- 默认实现可用 `serde_json` 或 `bincode`；若 state 含不可序列化类型，可预留 `Serializer` 注入（或后续加 EncryptedSerializer）。
- Checkpointer 实现可持有 `Arc<dyn Serializer<S>>`，put 时先 serialize 再存；get 时取 bytes 再 deserialize 成 `Checkpoint<S>`。

### 3.6 SqliteSaver（短期记忆持久化）

与 Python `langgraph-checkpoint-sqlite` 的 SqliteSaver 对应：将 checkpoint 写入本地 SQLite，进程重启后仍可恢复；适合单机部署与开发调试。

**表结构**：单表 `checkpoints`，主键为 `(thread_id, checkpoint_ns, checkpoint_id)`，state 以 BLOB 存序列化结果，元数据单独列以便 list/排序。

```rust
/// SQLite schema (logical). One row per checkpoint.
///   checkpoints(thread_id, checkpoint_ns, checkpoint_id, ts, parent_id,
///              payload BLOB, metadata_source TEXT, metadata_step INT, metadata_created_at INT)
///   - payload: serialized Checkpoint (channel_values + channel_versions via Serializer)
///   - metadata_*: CheckpointMetadata fields for list() and get_tuple
```

- **连接**：构造时传入数据库路径（如 `./data/checkpoints.db`）或 `:memory:` 做测试；使用 `sqlx::SqlitePool` 或 `rusqlite` 异步封装，保证 `Send + Sync`。
- **put**：`INSERT OR REPLACE` 或按 thread 维护「当前」行并 INSERT 新行；生成 `checkpoint_id`（如 UUID 或 `thread_id::step`），将 `Checkpoint<S>` 经 `Serializer` 得到 `Vec<u8>` 写入 `payload`。
- **get_tuple**：若 `config.checkpoint_id` 有值则 `SELECT ... WHERE thread_id = ? AND checkpoint_id = ?`；否则 `SELECT ... WHERE thread_id = ? ORDER BY metadata_created_at DESC LIMIT 1`。取出的 `payload` 经 `Serializer::deserialize` 得到 `Checkpoint<S>`。
- **list**：`SELECT checkpoint_id, metadata_* FROM checkpoints WHERE thread_id = ? AND checkpoint_ns = ?`，按 `before`/`after`（checkpoint_id 或时间）过滤，`LIMIT` 控制条数，返回 `Vec<CheckpointListItem>`。

**依赖**：可选 `sqlx`（async）或 `rusqlite`；若用 `rusqlite`，需配合 `tokio::task::spawn_blocking` 在 invoke 中调用，避免阻塞运行时。迁移（建表/升级）可用 `sqlx::migrate!` 或手写 `CREATE TABLE IF NOT EXISTS`。

---

## 4. 图与 Checkpointer 的集成

### 4.1 编译时注入 Checkpointer

与 Python `graph.compile(checkpointer=checkpointer)` 一致。

```rust
// StateGraph
pub fn compile_with_checkpointer(
    self,
    checkpointer: Arc<dyn Checkpointer<S>>,
) -> Result<CompiledStateGraph<S>, CompilationError>;

// CompiledStateGraph
pub async fn invoke(
    &self,
    state: S,
    config: Option<RunnableConfig>,
) -> Result<S, AgentError>;
```

- 若 `config` 为 `None` 或 `config.thread_id` 为 `None`：行为与当前一致，不读写 checkpoint，仅 `invoke(state)` 一次执行。
- 若提供 `config.thread_id` 且编译时注入了 checkpointer：  
  - 开始时：`get_tuple(config)` 得到上一 checkpoint，若有则以其 `channel_values` 作为初始 state 与本次传入的 `state` 做合并（或按策略：本次输入为 partial，与上一 state merge，见 [10-reducer-design](10-reducer-design.md)）；  
  - 结束时（或每步后）：将当前 state 写入 `put(config, checkpoint)`。

具体策略（每步写 vs 仅结束时写）可在文档中约定，实现时先采用「invoke 结束时写一次」亦可。

### 4.2 add_messages 与 channel 持久化

会话内消息历史 = state 的 `messages` channel 使用 `add_messages` reducer + Checkpointer 持久化。与 [10-reducer-design](10-reducer-design.md) 一致：节点返回 partial（如只更新 `messages`），运行时 merge；Checkpoint 存的是合并后的完整 state（或可只存各 channel 的序列化结果）。Rust 侧若 state 为单一结构体，则 checkpoint 即该结构体的序列化；若未来拆成多 channel，则 `channel_values` 可变为 `HashMap<String, Vec<u8>>`，每 channel 单独序列化。

---

## 5. 长期记忆：Store

### 5.1 Store trait

与 Python `BaseStore` 对齐：namespace、put、get、search（可选语义）。

```rust
/// Namespace for Store: e.g. (user_id, "memories") or (user_id, "preferences").
pub type Namespace = Vec<String>;

/// Error for store operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("serialization: {0}")]
    Serialization(String),
    #[error("storage: {0}")]
    Storage(String),
    #[error("not found")]
    NotFound,
}

/// Cross-thread key-value and optional semantic search. Aligns with LangGraph BaseStore.
#[async_trait]
pub trait Store: Send + Sync {
    /// Put a value under namespace and key. Value should be serializable (e.g. serde_json::Value).
    async fn put(
        &self,
        namespace: &Namespace,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), StoreError>;

    /// Get value by namespace and key.
    async fn get(
        &self,
        namespace: &Namespace,
        key: &str,
    ) -> Result<Option<serde_json::Value>, StoreError>;

    /// List keys in namespace (optional; not all backends support).
    async fn list(&self, namespace: &Namespace) -> Result<Vec<String>, StoreError>;

    /// Search in namespace. If the backend supports semantic index, query is natural language.
    /// Otherwise can be key prefix or full scan. Limit caps the number of results.
    async fn search(
        &self,
        namespace: &Namespace,
        query: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<StoreSearchHit>, StoreError>;
}

#[derive(Debug, Clone)]
pub struct StoreSearchHit {
    pub key: String,
    pub value: serde_json::Value,
    pub score: Option<f64>,
}
```

- **Namespace**：用 `Vec<String>` 表示元组，如 `vec![user_id.clone(), "memories".into()]`；与 Python `(user_id, "memories")` 对应。
- **search**：无语义索引的实现可做 prefix 或全量过滤；带语义的实现可接 Embedder + 向量检索，score 为相似度。

### 5.2 InMemoryStore

与 Python `InMemoryStore` 对应：进程内 `HashMap<(Namespace, key), value>`；search 可先做全量 list 再按 query 过滤，**语义检索采用内存向量数据库**（见 5.2.1）。

```rust
/// In-memory Store. Aligns with LangGraph InMemoryStore. Not persistent.
pub struct InMemoryStore {
    inner: std::sync::Arc<tokio::sync::RwLock<InMemoryStoreInner>>,
}

struct InMemoryStoreInner {
    by_key: std::collections::HashMap<(Namespace, String), serde_json::Value>,
}
```

- **put/get/list**：直接对 `by_key` 操作。
- **search**：若无语义索引，可 `list(namespace)` 后按 `query` 做字符串包含或键前缀过滤，返回前 `limit` 条；**带语义时使用内存向量存储**（5.2.1）。

#### 5.2.1 语义检索：内存向量存储

本方案中 Store 的语义 search 使用**内存向量数据库**实现：

- **存储**：进程内维护 `(namespace, key) -> (value, embedding)`，embedding 由 Embedder 生成后存入内存（如 `Vec<(Namespace, key, value, Vec<f32>)>` 或按 namespace 分片的 `VectorMemory`）。
- **检索**：对 query 做 embedding，在进程内用余弦相似度做 top-k 检索，不依赖 Qdrant、Milvus 等独立服务。
- **特点**：无额外部署、延迟低；进程退出后数据不持久化，适合开发与单机/小规模；若需持久化可后续接 Lance 或 SQLite+向量扩展等。

#### 5.2.2 SqliteStore（长期记忆持久化）

与 InMemoryStore 同实现 Store trait，但将 key-value 写入本地 SQLite，进程重启后数据保留；适合单机生产或需要跨进程共享配置的场景。

**表结构**：单表 `store_kv`，主键为 `(namespace, key)`，value 存 JSON 文本；可选第二张表存向量用于语义 search（见下）。

```rust
/// SQLite schema (logical).
///   store_kv(ns_0, ns_1, ..., key, value TEXT)  -- namespace 用多列或单列 JSON 存储
/// 简化：namespace 序列化为单列，如 ns TEXT = json_encode(namespace), key TEXT, value TEXT.
///   PRIMARY KEY (ns, key)
```

- **put/get/list**：`INSERT OR REPLACE` / `SELECT value WHERE ns = ? AND key = ?` / `SELECT key FROM store_kv WHERE ns = ?`。value 为 `serde_json::Value` 的 `to_string()` 结果。
- **search**：初版可做「无语义」检索：`list(namespace)` 后在内存中按 `query` 做 key 前缀或 value 包含过滤，返回前 `limit` 条；与 InMemoryStore 的 fallback 一致。后续可加 `store_vectors(ns, key, embedding BLOB)` 与 SQLite 向量扩展（如 sqlite-vec）做语义 search。
- **连接与迁移**：构造时传入 DB 路径；可与 SqliteSaver 共用同一 DB（不同表）或单独文件（如 `./data/store.db`），按部署需求选择。

### 5.4 图与 Store 的集成

- 编译：`compile_with_store(store: Arc<dyn Store>)` 或 `compile(checkpointer, store)`，图持有 `Option<Arc<dyn Store>>`。
- 节点内使用：通过 `RunnableConfig::user_id` 拼 namespace，在节点中调用 `store.search(namespace, query, limit)`，将结果注入系统提示或上下文。Store 与 state 解耦，不写入 checkpoint，仅在图节点中读写。

---

## 6. 模块与文件划分（Rust 风格）

建议按「一个类型/一个职责一个文件」组织，测试独立。

```
crates/langgraph/src/
├── memory/
│   ├── mod.rs           // 导出 config、checkpoint、checkpointer、store、errors
│   ├── config.rs        // RunnableConfig
│   ├── checkpoint.rs    // Checkpoint, CheckpointMetadata, CheckpointSource, CheckpointListItem
│   ├── checkpointer.rs   // Checkpointer trait, CheckpointError
│   ├── memory_saver.rs   // MemorySaver<S>
│   ├── sqlite_saver.rs   // SqliteSaver<S>（持久化 Checkpointer，见 3.6）
│   ├── serializer.rs    // Serializer trait, JsonSerializer or default impl
│   ├── store.rs          // Store trait, StoreError, StoreSearchHit
│   ├── in_memory_store.rs // InMemoryStore
│   └── sqlite_store.rs   // SqliteStore（持久化 Store，见 5.2.2）
├── graph/
│   ├── compiled.rs      // 增加 invoke(state, config)、compile_with_checkpointer
│   └── ...
```

- **memory** 不依赖 **graph**；**graph** 依赖 **memory**（config、checkpointer、serializer）。Store 可选依赖，由编译或节点注入。
- **SqliteSaver / SqliteStore**：可选依赖 `sqlx` 或 `rusqlite`；若以 feature 开关（如 `sqlite`）提供，未开启时可不编译 sqlite_saver / sqlite_store。

---

## 7. 与现有设计的关系

| 现有 | 记忆设计 |
|------|----------|
| [09-minimal-agent-design](09-minimal-agent-design.md) | 无 checkpointer 时仍为「调用方持 state」；有 checkpointer 时由 Checkpointer 持 state，调用方只传 config。 |
| [10-reducer-design](10-reducer-design.md) | checkpoint 中存的是 merge 后的 state；add_messages 等 reducer 在节点/运行时合并，合并结果写入 checkpoint。 |
| [11-state-graph-design](11-state-graph-design.md) | 扩展：`compile_with_checkpointer`、`invoke(state, config)`；条件边、partial 不变。 |
| [MEMORY_VS_LANGGRAPH_STORE](MEMORY_VS_LANGGRAPH_STORE.md) | Checkpointer = 短期记忆；Store = 长期记忆；SemanticMemory/VectorMemory 可作 Store 的语义检索实现或上层封装。 |

---

## 8. 小结与任务表

- **短期记忆**：RunnableConfig（thread_id, checkpoint_id, checkpoint_ns）+ Checkpoint\<S\> + Checkpointer trait + MemorySaver + **SqliteSaver** + Serializer；图 compile 时注入 checkpointer，invoke 时传 config，自动读/写 checkpoint。
- **长期记忆**：Store trait（namespace + put/get/list/search）+ InMemoryStore + **SqliteStore**；图或节点持有 Store，按 user_id 等拼 namespace 做跨会话读写与检索；**语义检索采用内存向量数据库**（进程内 embedding + 余弦相似度），不依赖外部向量服务；持久化可选 SqliteStore，语义 search 可后续接 SQLite 向量扩展。
- **Rust 风格**：trait 与类型分文件、错误类型显式、异步 API；注释英文；测试独立。

| 序号 | 任务 | 交付物 / 子项 | 状态 | 说明 |
|------|------|----------------|------|------|
| 1 | 设计文档 | 本文档 16-memory-design.md | 已完成 | 以 Python LangGraph 为蓝本，Rust 风格；含 SQLite 持久化方案（3.6、5.2.2） |
| 2 | RunnableConfig | memory/config.rs | 已完成 | thread_id, checkpoint_id, checkpoint_ns, user_id |
| 3 | Checkpoint / Metadata | memory/checkpoint.rs | 已完成 | Checkpoint\<S\>, CheckpointMetadata, CheckpointSource, CheckpointListItem |
| 4 | CheckpointError | memory/checkpointer.rs | 已完成 | CheckpointError 变体 |
| 5 | Checkpointer trait | memory/checkpointer.rs | 已完成 | put, get_tuple, list |
| 6 | Serializer trait | memory/serializer.rs | 已完成 | serialize, deserialize；JsonSerializer |
| 7 | MemorySaver | memory/memory_saver.rs | 已完成 | 进程内 HashMap，存 Checkpoint\<S\> |
| 8 | 图集成 checkpointer | graph/compiled.rs, state_graph.rs | 已完成 | compile_with_checkpointer, invoke(state, config)；结束时写 checkpoint |
| 9 | Store trait + StoreError | memory/store.rs | 已完成 | put, get, list, search；Namespace, StoreSearchHit |
| 10 | InMemoryStore | memory/in_memory_store.rs | 已完成 | 进程内 HashMap；语义 search 用内存向量存储（embedding + 余弦相似度） |
| 11 | 文档与 README | README.md、MEMORY_VS_LANGGRAPH_STORE.md | 已完成 | 增加 16-memory-design 链接与对照说明 |
| 12 | SqliteSaver | memory/sqlite_saver.rs | 已完成 | 持久化 Checkpointer；SQLite 表 checkpoints；可选 feature `sqlite` |
| 13 | SqliteStore | memory/sqlite_store.rs | 已完成 | 持久化 Store；SQLite 表 store_kv；put/get/list；search 初版为 key/value 过滤 |

实现时按上表推进，每项完成后在「状态」列改为「已完成」，并在代码处补充注释引用本文（如 `16-memory-design.md`）。
