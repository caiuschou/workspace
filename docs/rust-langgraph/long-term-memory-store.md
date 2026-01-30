# rust-langgraph 长期记忆方案（Store）

## 概述

长期记忆用于**跨会话、跨 thread** 的数据：用户偏好、长期档案、可检索的记忆等。与短期记忆（Checkpointer + thread_id）解耦。

- **设计蓝本**：[16-memory-design.md](16-memory-design.md) 第 5 节；与 LangGraph 的 **BaseStore / InMemoryStore** 对齐。
- **核心结论**：长期记忆不一定要用向量。Store 的核心是 **namespace + key-value**（put / get / list）；`search` 可以是纯字符串过滤（key/value 包含、前缀等），无需 embedding。**向量与语义检索是可选项**，仅在需要「按语义相似度检索」时选用 LanceStore 或其它带向量的实现。

---

## Guideline

### 选型建议

| 场景 | 推荐 Store | 说明 |
|------|------------|------|
| 开发 / 单测 | InMemoryStore | 无持久化，进程内 HashMap，实现简单 |
| 单机生产、仅 KV 读写 | SqliteStore | 持久化、无语义检索，feature `sqlite` |
| 需要语义检索 / RAG | LanceStore | 持久化 + 向量相似度检索，feature `lance`，需 Embedder |

### 设计原则

1. **Namespace 隔离**：用 `Vec<String>` 表达多租户、多用途，例如 `[user_id, "memories"]`、`[user_id, "preferences"]`；同一 namespace 内 key 唯一。
2. **Value 形态**：统一使用 `serde_json::Value`，便于扩展字段、兼容不同业务结构。
3. **检索分层**：默认提供字符串过滤（key/value 包含、前缀）；语义检索仅在有向量实现的 Store（如 LanceStore）中提供。
4. **与图解耦**：Store 不强制绑定到图的编译流程；业务在节点内持有 `Arc<dyn Store>`，按 `user_id` 等拼 namespace 使用。

### 使用约定

- **Key 命名**：建议有前缀或类型区分，如 `pref_theme`、`mem_<id>`，便于 list/search 过滤。
- **Search 行为**：无 query 或 query 为空时，可退化为 list 或按 limit 返回；有 query 时由实现决定（字符串匹配或向量相似度）。
- **错误与一致性**：get 不存在的 key 返回 `None`；put 同一 (namespace, key) 为覆盖写；list/search 只返回当前 namespace 下数据。

### 实现与验证顺序

1. 定义 `Store` trait 与 `StoreSearchHit` 类型。
2. 实现 InMemoryStore（无 feature，用于单测与开发）。
3. 实现 SqliteStore（feature `sqlite`），补充集成测试。
4. 按需实现 LanceStore（feature `lance`），对接 Embedder 与 LanceDB。

---

## 开发计划

### 总表（细粒度任务）

| 序号 | 项 | 状态 | 说明 |
|------|----|------|------|
| **P1** | **Store trait 与类型定义** | | |
| P1.1 | Store 错误类型（StoreError） | 完成 | 统一错误，不暴露底层 DB |
| P1.2 | StoreSearchHit 类型定义 | 完成 | key, value, score? |
| P1.3 | Store trait 方法签名 | 完成 | put/get/list/search，async |
| P1.4 | memory 模块与 Cargo 导出 | 完成 | store.rs、mod.rs、lib.rs |
| P1.5 | Store / StoreSearchHit 文档注释 | 完成 | 英文，namespace/key/search 语义 |
| **P2** | **InMemoryStore** | | |
| P2.1 | namespace 序列化与复合 key 约定 | 完成 | 稳定字符串，唯一键 |
| P2.2 | 内部存储结构（Map + 并发） | 完成 | Arc+RwLock 或 DashMap |
| P2.3 | put / get 实现 | 完成 | 覆盖写、get 返回 Option |
| P2.4 | list 实现 | 完成 | 按 namespace 过滤，返回 key 列表 |
| P2.5 | search 实现（字符串包含） | 完成 | key/value contains，limit，score=None |
| P2.6 | InMemoryStore 单元测试 | 完成 | put/get/list/search + 隔离 |
| **P3** | **SqliteStore** | | |
| P3.1 | Cargo feature `sqlite` 与依赖 | 完成 | rusqlite/sqlx，可选 WAL |
| P3.2 | 表 store_kv 与初始化 | 完成 | CREATE TABLE IF NOT EXISTS |
| P3.3 | 连接管理与并发 | 完成 | spawn_blocking 每操作新连接 |
| P3.4 | put / get / list 实现 | 完成 | INSERT OR REPLACE，SELECT |
| P3.5 | search 实现（LIKE 参数化） | 完成 | 防注入，返回 StoreSearchHit |
| P3.6 | SqliteStore 集成测试 | 完成 | 临时 DB、持久化、隔离 |
| **P4** | **LanceStore** | | |
| P4.1 | Cargo feature `lance` 与 Embedder | 完成 | lancedb，Embedder trait |
| P4.2 | LanceDB 表结构与 (ns,key) 唯一 | 完成 | ns, key, value, vector；put 前 delete 再 add |
| P4.3 | put/get/list（put 时 embedding） | 完成 | 提取 text，写 vector |
| P4.4 | search 有 query（向量相似度） | 完成 | embed query，nearest_to，score |
| P4.5 | search 无 query 退化 | 完成 | 同 list 或按 limit 取 |
| P4.6 | LanceStore 集成测试 | 完成 | 语义检索顺序与 score |
| **P5** | **与图集成（可选）** | | |
| P5.1 | 文档：节点内持有 Store 示例 | 待办 | RunnableConfig::user_id → namespace |
| P5.2 | （可选）compile_with_store API | 待办 | 图持有 Option<Arc<dyn Store>> |
| P5.3 | （可选）config 传 user_id/namespace | 待办 | 节点从 config 取 namespace |
| P5.4 | （可选）示例或测试 | 待办 | 端到端读写 Store |

以下为每一项的详细方案与验收。

---

### P1.1：Store 错误类型（StoreError）

**目标**：为 Store 操作提供统一错误类型，调用方不依赖具体实现（如 rusqlite、lancedb）的错误。

**方案**：

- 在 `store.rs` 中定义 `StoreError`（或复用 crate 内已有 error 类型）。
- 使用 `thiserror` 或手写：变体可包含 `Serialization`、`NotFound`（若需）、`Internal(String)` 等；避免直接暴露 `rusqlite::Error` 等。
- 为常见底层错误实现 `From` 转换（如 `From<serde_json::Error>`）。
- 实现 `std::error::Error` 与 `Display`。

**验收**：`StoreError` 可在 `Store` 的 `Result` 中使用；`cargo build -p langgraph` 通过。

---

### P1.2：StoreSearchHit 类型定义

**目标**：定义 search 返回的单项结构，支持无语义（score=None）与语义检索（score=相似度）。

**方案**：

- 结构体字段：`key: String`、`value: serde_json::Value`、`score: Option<f32>`。
- 实现 `Clone`、`Debug`，必要时 `Serialize/Deserialize`。
- 文档注释（英文）：说明 key/value 为命中条目，score 仅在向量检索时存在。

**验收**：类型可被 `Store::search` 返回；文档完整。

---

### P1.3：Store trait 方法签名

**目标**：定稿 Store 的公共 API，供 InMemoryStore / SqliteStore / LanceStore 实现。

**方案**：

- trait 定义于 `store.rs`，使用 `#[async_trait]` 或标准库 async fn（视 Rust 版本）。
- 方法：
  - `async fn put(&self, namespace: &[String], key: &str, value: &serde_json::Value) -> Result<(), StoreError>`
  - `async fn get(&self, namespace: &[String], key: &str) -> Result<Option<serde_json::Value>, StoreError>`
  - `async fn list(&self, namespace: &[String]) -> Result<Vec<String>, StoreError>`
  - `async fn search(&self, namespace: &[String], query: Option<&str>, limit: Option<u32>) -> Result<Vec<StoreSearchHit>, StoreError>`
- 文档注释：说明 namespace 隔离、同一 namespace 内 key 唯一、put 覆盖写、search 的 query/limit 语义（无 query 可退化为 list 行为）。

**验收**：trait 可被 `impl Store for InMemoryStore` 等使用；签名与文档一致。

---

### P1.4：memory 模块与 Cargo 导出

**目标**：将 Store 相关类型纳入 langgraph crate 的公共 API，且不破坏现有构建。

**方案**：

- 在 `crates/langgraph/src/memory/` 下新增 `store.rs`（或 `store/mod.rs`），内含 `StoreError`、`StoreSearchHit`、`Store`。
- 在 `memory/mod.rs` 中：`mod store; pub use store::{Store, StoreSearchHit, StoreError};`
- 在 `lib.rs` 中确保 `pub mod memory`（或 `pub use memory::{Store, ...}`）导出。
- Cargo.toml：若 Store 依赖 `serde_json`、`async_trait` 等，确保为已有依赖或新加依赖无误。

**验收**：其他 crate 可 `use langgraph::memory::{Store, StoreSearchHit, StoreError}`；`cargo build -p langgraph` 通过。

---

### P1.5：Store / StoreSearchHit 文档注释

**目标**：为对外类型与方法提供英文文档，便于实现者与调用方理解契约。

**方案**：

- 为 `Store` trait 写模块级或 trait 级文档：长期记忆、namespace 隔离、与 Checkpointer 的区别。
- 为 `put`/`get`/`list`/`search` 各写文档：参数含义、返回值、错误情况；特别说明 `search` 的 `query: Option` 与 `limit: Option` 的语义及「无 query 时退化」。
- 为 `StoreSearchHit` 及字段写文档：`score` 仅在语义检索时有意义。

**验收**：`cargo doc -p langgraph --no-deps` 生成文档且无警告；阅读体验清晰。

---

### P2.1：namespace 序列化与复合 key 约定

**目标**：在内存实现中，将 (namespace, key) 映射为唯一字符串键，且可反向解析出 namespace 用于 list/search 过滤。

**方案**：

- 约定：`internal_key = serde_json::to_string(namespace)? + "::" + key`（或 `format!("{}::{}", serde_json::to_string(ns)?, key)`）；namespace 的 JSON 序列化需稳定（如数组排序一致）。
- 提供私有辅助函数：`fn to_internal_key(namespace: &[String], key: &str) -> Result<String, StoreError>` 与 `fn namespace_prefix(namespace: &[String]) -> Result<String, StoreError>`（用于 list 时过滤）。
- 从 internal_key 解析回「用户 key」：取 `"::"` 后的部分即可。

**验收**：同一 namespace 下不同 key 得到不同 internal_key；不同 namespace 不会碰撞；单元测试验证 to_internal_key / namespace_prefix。

---

### P2.2：内部存储结构（Map + 并发）

**目标**：InMemoryStore 内部用可并发访问的 KV 结构存储数据。

**方案**：

- 类型：`Arc<RwLock<HashMap<String, serde_json::Value>>>` 或 `Arc<DashMap<String, serde_json::Value>>`。
- `InMemoryStore::new()` 返回 `Self { inner: Arc::new(RwLock::new(HashMap::new())) }`（或 DashMap 等价）。
- 若需 `Send + Sync`，确保选型满足（RwLock/HashMap/DashMap 均满足）。

**验收**：`InMemoryStore::new()` 编译通过；结构可被 put/get 使用。

---

### P2.3：put / get 实现

**目标**：实现 Store 的 put 与 get，行为为覆盖写、get 不存在返回 None。

**方案**：

- `put`：计算 `to_internal_key(namespace, key)`，将 value 克隆后插入 map（注意 `serde_json::Value` 的 Clone）。
- `get`：计算 internal_key，从 map 取 `Option`，克隆后返回；不存在则 `Ok(None)`。
- 错误：序列化 namespace 失败时返回 `StoreError::Serialization`（或等价）。

**验收**：同一 (namespace, key) 二次 put 后 get 得到最新值；get 不存在的 key 为 `Ok(None)`；单元测试覆盖。

---

### P2.4：list 实现

**目标**：返回指定 namespace 下所有条目的 key（仅用户 key，不含 internal 前缀）。

**方案**：

- 使用 `namespace_prefix(namespace)` 得到前缀。
- 遍历 map（或迭代器）：筛选 key 以该前缀开头的条目；从每条 key 中截取 "::" 后的部分作为用户 key 放入 `Vec<String>`。
- 返回前可去重、排序（文档若无要求则实现决定）。

**验收**：put 若干条后 list 返回全部且仅该 namespace 的 key；不同 namespace 互不干扰；单元测试覆盖。

---

### P2.5：search 实现（字符串包含）

**目标**：在指定 namespace 内，按 key 或 value 的字符串形式包含 query 过滤，最多返回 limit 条，score 为 None。

**方案**：

- 先通过 namespace_prefix 筛选出该 namespace 的条目。
- 若 `query` 为 `None` 或空字符串：可退化为 list 行为，取前 `limit` 条，构造 `StoreSearchHit { key, value, score: None }`。
- 若 `query` 有值：对每条 entry，将 key 与 `value.to_string()`（或序列化）与 query 做 `contains` 判断，匹配的加入结果，直到达到 `limit`（若 limit 为 None 则实现约定上限或全部返回）。
- 返回 `Vec<StoreSearchHit>`，score 一律 `None`。

**验收**：search 含 query 时只返回 key 或 value 包含 query 的条目；limit 生效；无 query 时与 list 行为一致；单元测试覆盖。

---

### P2.6：InMemoryStore 单元测试

**目标**：对 InMemoryStore 做完整行为测试，保证与 Store 契约一致。

**方案**：

- 测试文件：`crates/langgraph/src/memory/in_memory_store.rs` 内 `#[cfg(test)] mod tests` 或 `crates/langgraph/tests/memory_in_memory.rs`。
- 用例建议：put 后 get 同值；同一 key 覆盖 put 后 get 为新值；get 不存在的 key 为 None；list 仅返回该 namespace 的 key；list 不同 namespace 隔离；search 含 query 时过滤正确；search 无 query 时与 list 一致；limit 生效。
- 无需 feature，`cargo test -p langgraph` 即可运行。

**验收**：`cargo test -p langgraph` 通过，且上述用例均存在并通过。

---

### P3.1：Cargo feature `sqlite` 与依赖

**目标**：在 Cargo.toml 中增加可选 SQLite 支持，不启用时不影响默认构建。

**方案**：

- `[features]` 增加 `sqlite = ["dep:rusqlite"]`（或 `sqlx/sqlite` 等）。
- `[dependencies]` 增加 `rusqlite = { version = "...", optional = true, features = ["bundled"] }`（或按需）。
- 可选：为连接启用 WAL，在打开连接后执行 `PRAGMA journal_mode=WAL;`。

**验收**：`cargo build -p langgraph`（无 feature）与 `cargo build -p langgraph --features sqlite` 均通过。

---

### P3.2：表 store_kv 与初始化

**目标**：SqliteStore 使用固定表结构，首次打开时自动建表。

**方案**：

- 表名：`store_kv`。列：`ns TEXT NOT NULL`、`key TEXT NOT NULL`、`value TEXT NOT NULL`；唯一约束 `UNIQUE(ns, key)`。
- 在 `SqliteStore::new(path)` 或 `open()` 中：打开/创建连接后执行 `CREATE TABLE IF NOT EXISTS store_kv (ns TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, UNIQUE(ns, key))`；可选创建索引 `CREATE INDEX IF NOT EXISTS idx_store_kv_ns ON store_kv(ns)`。

**验收**：首次打开新 DB 文件后表存在；重复打开不报错；表结构符合文档。

---

### P3.3：连接管理与并发

**目标**：保证多线程下对同一 SqliteStore 的访问安全。

**方案**：

- 字段：`inner: Mutex<rusqlite::Connection>` 或 `RwLock<Connection>`（rusqlite 的 Connection 非 Send，若需跨线程则用 `rusqlite::Connection::open_with_flags` 等或考虑 `sqlx` 多线程连接）。
- 若使用 `rusqlite`：在单线程内使用则 `Mutex` 足够；若需 `Send`，可查文档是否支持或改用 `sqlx::SqlitePool`。
- 每次 put/get/list/search 在锁内获取 connection 并执行语句。

**验收**：多线程并发调用 put/get 不 panic；无数据竞争（可加 loom 或简单并发测试）。

---

### P3.4：put / get / list 实现

**目标**：SqliteStore 的 put、get、list 与 Store 契约一致，数据落盘。

**方案**：

- `put`：`ns = serde_json::to_string(namespace)?`，`value = value.to_string()`；`INSERT OR REPLACE INTO store_kv (ns, key, value) VALUES (?, ?, ?)`。
- `get`：`SELECT value FROM store_kv WHERE ns = ? AND key = ?`，取一行，将 value 列反序列化为 `serde_json::Value`；无行则 `Ok(None)`。
- `list`：`SELECT key FROM store_kv WHERE ns = ?`，收集为 `Vec<String>`。
- 错误：序列化/反序列化失败映射为 StoreError；rusqlite 错误映射为 StoreError::Internal 或等价。

**验收**：put 后 get 得到同值；重启进程后 get 仍能读到（持久化）；list 仅返回该 ns 的 key；集成测试覆盖。

---

### P3.5：search 实现（LIKE 参数化）

**目标**：在 store_kv 内按 ns 过滤，并按 key/value 的 LIKE 匹配 query，返回 StoreSearchHit，score 为 None。

**方案**：

- SQL：`SELECT key, value FROM store_kv WHERE ns = ? AND (key LIKE ? OR value LIKE ?) LIMIT ?`。
- 将 query 参数化为 `%query%`（LIKE 通配符），避免拼接 SQL 导致注入；limit 用绑定参数（或取 limit.unwrap_or(100) 等上限）。
- 遍历结果构造 `StoreSearchHit { key, value: serde_json::from_str(row.value)?, score: None }`。

**验收**：search 返回 key 或 value 包含 query 的条目；limit 生效；无 SQL 注入；集成测试覆盖。

---

### P3.6：SqliteStore 集成测试

**目标**：在临时 DB 上验证 SqliteStore 的完整行为与持久化。

**方案**：

- 测试：创建临时路径（如 `tempfile::NamedTempFile` 或 `std::env::temp_dir()` 下随机文件名），`SqliteStore::new(path)`；执行 put/get/list/search；针对同一 path 再打开一次 Store，再次 get 验证持久化；多 namespace 写入后 list/search 验证隔离。
- 测试入口：`crates/langgraph/tests/memory_sqlite.rs` 或类似，`#[cfg(feature = "sqlite")]` 条件编译。

**验收**：`cargo test -p langgraph --features sqlite` 通过；持久化与隔离符合文档。

---

### P4.1：Cargo feature `lance` 与 Embedder

**目标**：引入 LanceDB 与 Embedder 抽象，可选编译。

**方案**：

- Cargo：`lance = ["dep:lancedb", ...]`，依赖 `lancedb`（版本按生态选择）。
- Embedder：若 crate 内已有 `Embedder` trait（如 `embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>`），则复用；否则在 `lance_store.rs` 或 `embedding.rs` 中定义最小 trait，供 LanceStore 与测试 mock 使用。

**验收**：`cargo build -p langgraph --features lance` 通过；LanceStore 可接收 `Arc<dyn Embedder>`。

---

### P4.2：LanceDB 表结构与 (ns, key) 唯一性

**目标**：确定 LanceDB 中表/数据集的 schema，并保证 (namespace, key) 唯一。

**方案**：

- Schema：`ns` (string)、`key` (string)、`value` (string)、`vector` (fixed-size list of float32)。
- 唯一性：LanceDB 若不支持唯一约束，则在应用层 put 时先按 (ns, key) 查是否存在；存在则更新该行（更新 vector 与 value），否则 append 新行。
- 表名：可与 path 同名的单表，或固定表名如 `store`；文档中写明约定。

**验收**：同一 (ns, key) 两次 put 后仅一条记录；get 返回最新 value。

---

### P4.3：put / get / list（put 时 embedding）

**目标**：实现 LanceStore 的 put/get/list；put 时从 value 提取文本并写入向量。

**方案**：

- **put**：从 value 中取可嵌入文本（如 `value.get("text").and_then(|v| v.as_str())`，无则用 `value.to_string()`）；调用 `embedder.embed([text])`，得到 `Vec<f32>`；将 ns（如 JSON 序列化）、key、value（JSON 字符串）、vector 写入 LanceDB；若 (ns, key) 已存在则更新。
- **get**：按 (ns, key) 查表，返回 value 列反序列化；不读 vector。
- **list**：按 ns 过滤，返回 key 列表。

**验收**：put 后 get/list 与 InMemoryStore 行为一致；向量被正确写入（可查表或后续 search 验证）。

---

### P4.4：search 有 query（向量相似度）

**目标**：当 search 的 query 为 Some 时，对 query 做 embedding，用向量相似度检索并返回 score。

**方案**：

- 调用 `embedder.embed([query])` 得到 query 向量。
- 使用 LanceDB 的向量检索 API：在表上按 ns 过滤后（或先过滤再检索），`nearest_to(query_vector).limit(limit)`，得到按相似度排序的结果。
- 将每条结果的 key、value、相似度 score 填入 `StoreSearchHit`；score 为相似度值（如余弦或 L2，与 LanceDB 一致）。

**验收**：语义相近的 query 能命中对应条目且 score 较高；集成测试用固定 embedder 验证顺序。

---

### P4.5：search 无 query 退化

**目标**：当 query 为 None 或空时，行为与 list 或「按 limit 取前若干条」一致，score 为 None。

**方案**：

- 实现：仅按 ns 过滤，不调用 embedder；按 key 或插入顺序取前 limit 条，构造 `StoreSearchHit { key, value, score: None }`。
- 与文档一致：无 query 时退化为 list 或 limit 列表。

**验收**：search(namespace, None, limit) 返回条数不超过 limit，且 score 均为 None；与 list 语义一致或文档明确差异。

---

### P4.6：LanceStore 集成测试

**目标**：端到端验证 LanceStore 的 put/get/list/search 与语义检索。

**方案**：

- 使用本地临时目录作为 LanceDB 路径；Embedder 可用 mock（固定返回某向量）或真实小模型。
- 用例：put 若干条带 `text` 的 value；get/list 正确；search 带 query 时返回顺序与 score 合理；search 无 query 时与 list 一致；重启后 get 仍有效（持久化）。

**验收**：`cargo test -p langgraph --features lance` 通过；语义检索与文档描述一致。

---

### P5.1：文档：节点内持有 Store 示例

**目标**：在文档中明确「当前推荐用法」：节点/Agent 内持有 `Arc<dyn Store>`，从 RunnableConfig 取 user_id 拼 namespace。

**方案**：

- 在「与图的配合方式」或独立「用法示例」中增加代码片段：创建 `InMemoryStore`/`SqliteStore`，包装为 `Arc<dyn Store>`；在节点函数或 state 中持有该 Arc；在调用处从 `config.get("user_id")` 或类似取 user_id，拼 `namespace = vec![user_id, "memories"]`；调用 `store.put`/`store.get`/`store.search`。
- 说明 namespace 的推荐结构（如 `[user_id, "memories"]`、`[user_id, "preferences"]`）。

**验收**：文档中存在可复制的示例；读者能据此在节点内使用 Store。

---

### P5.2：（可选）compile_with_store API

**目标**：若需要图级别持有 Store，则设计 `compile_with_store(store)` API，图内部保存 `Option<Arc<dyn Store>>`。

**方案**：

- 在 `CompiledGraph` 或 builder 上增加方法，例如 `fn with_store(self, store: Arc<dyn Store>) -> Self` 或 `fn compile_with_store(store: Arc<dyn Store>) -> CompiledGraph`（视现有 API 风格而定）。
- 图结构体增加字段 `store: Option<Arc<dyn Store>>`；run 时若存在则通过 config 或其它方式供节点使用（见 P5.3）。

**验收**：编译后的图能持有 Store；不破坏现有无 Store 的用法。

---

### P5.3：（可选）config 传 user_id / namespace

**目标**：节点执行时从 RunnableConfig（或自定义 config）读取 user_id 或完整 namespace，用于调用 Store。

**方案**：

- 约定 config 中 key：如 `configurable["user_id"]` 或 `configurable["store_namespace"]`；若为 user_id，则默认 namespace 为 `[user_id, "memories"]`，或由业务在 config 中传 `store_namespace` 覆盖。
- 在节点执行路径中：若图持有 Store，则从 config 取上述字段，拼 namespace，调用 store 的 put/get/search；文档说明 config 的 schema。

**验收**：示例或测试中能从 config 取到 user_id/namespace 并完成一次 store 读写。

---

### P5.4：（可选）示例或测试

**目标**：提供端到端示例或集成测试，演示「图 + Store」的完整流程。

**方案**：

- 示例：最小图（如单节点），节点内使用 Store 写入/读取一条记忆；run 时传入 `configurable: { user_id: "u1" }`；断言 get 得到写入值。
- 或集成测试：编译图时 `with_store(Arc::new(InMemoryStore::new()))`，执行一轮后检查 Store 中是否有预期 key/value。

**验收**：示例可运行或测试通过；与 P5.1～P5.3 文档一致。

---

## 1. 抽象：Store trait

位置：`crates/langgraph/src/memory/store.rs`。

| 能力 | 方法 | 说明 |
|------|------|------|
| 写入 | `put(namespace, key, value)` | 按 namespace + key 存 JSON 值 |
| 读取 | `get(namespace, key)` | 按 namespace + key 取 |
| 列举 | `list(namespace)` | 返回该 namespace 下所有 key |
| 检索 | `search(namespace, query?, limit?)` | 在 namespace 内按 query 过滤（或语义检索），返回 `StoreSearchHit[]` |

- **Namespace**：`Vec<String>`，例如 `vec![user_id, "memories"]` 或 `vec![user_id, "preferences"]`，用于多用户、多用途隔离。
- **Value**：`serde_json::Value`，任意可 JSON 序列化的结构。
- **StoreSearchHit**：`{ key, value, score? }`；无语义时 `score` 为 `None`。

---

## 2. 实现

### 2.1 InMemoryStore（不持久化）

- **文件**：`memory/in_memory_store.rs`
- **存储**：进程内 `HashMap`，key = namespace 与 key 的拼接。
- **search**：按 key/value 字符串包含过滤，无向量检索。
- **用途**：单机开发、测试；进程退出数据丢失。

```rust
use langgraph::memory::{InMemoryStore, Store};

let store = InMemoryStore::new();
let ns = vec!["user-1".into(), "memories".into()];
store.put(&ns, "k1", &serde_json::json!({"text": "hello"})).await?;
let v = store.get(&ns, "k1").await?;
let keys = store.list(&ns).await?;
let hits = store.search(&ns, Some("hello"), Some(10)).await?;
```

### 2.2 SqliteStore（持久化，无语义）

- **文件**：`memory/sqlite_store.rs`
- **特性**：需启用 **feature `sqlite`**。
- **存储**：SQLite 表 `store_kv(ns, key, value)`，`ns` 为 namespace 的 JSON 序列化，`value` 为 JSON 文本。
- **search**：当前为 key/value 字符串过滤，无向量索引；设计上可后续接 SQLite 向量扩展做语义 search。
- **用途**：单机生产、跨进程共享、进程重启后数据保留。

```rust
use langgraph::memory::{SqliteStore, Store};

let store = SqliteStore::new("data/store.db")?;
let ns = vec!["user-1".into(), "memories".into()];
store.put(&ns, "pref_theme", &serde_json::json!("dark")).await?;
let theme = store.get(&ns, "pref_theme").await?;
```

### 2.3 LanceStore（Lance 持久化 + 语义检索）

- **目标**：长期记忆的**持久化 + 语义检索**，使用 [LanceDB](https://lancedb.com/)（基于 Lance 格式的向量库）存储 embedding 与元数据，支持按 namespace 隔离的 put/get/list 与向量相似度 search。
- **特性**：需启用 **feature `lance`**（可选依赖 `lancedb`）。
- **存储**：
  - LanceDB 本地路径（如 `data/lance-store`）或 S3/GS 等对象存储路径。
  - 表结构建议：`ns`（namespace JSON）、`key`、`value`（JSON 文本）、`vector`（FixedSizeList\<Float32\>，由 Embedder 生成）。主键逻辑为 `(ns, key)`；put 时对 value 中可嵌入字段（如 `text`）做 embedding 写入 `vector`。
- **search**：
  - 有 `query` 时：对 query 做 embedding，调用 LanceDB `nearest_to(vector).limit(limit)`，按 namespace 过滤，返回 `StoreSearchHit { key, value, score }`（score 为相似度）。
  - 无 `query` 时：退化为 list 或按 key/value 过滤，与 SqliteStore 行为一致。
- **Embedder**：LanceStore 构造时接收 `Arc<dyn Embedder>`（或使用 LanceDB 的 `EmbeddingFunction` 对接 OpenAI/HuggingFace 等），用于写入时对 value 生成向量、检索时对 query 生成向量。
- **用途**：单机或云上持久化、语义检索、RAG/长期记忆检索；进程重启后数据保留，无需单独向量服务。

```rust
// 设计示例（待实现）
use langgraph::memory::{LanceStore, Store};

let embedder = Arc::new(OpenAIEmbedder::default()); // 或业务提供的 Embedder
let store = LanceStore::new("data/lance-store", embedder, 1536).await?;
let ns = vec!["user-1".into(), "memories".into()];
store.put(&ns, "m1", &serde_json::json!({"text": "用户喜欢深色主题"})).await?;
let hits = store.search(&ns, Some("主题偏好"), Some(5)).await?; // 语义检索，返回 score
```

- **与 16-memory-design 的关系**：替代或补充「5.2.1 内存向量存储」的持久化方案；语义检索持久化采用 Lance，不再依赖「SQLite 向量扩展」或仅内存向量。

---

## 3. 与图的配合方式

- **当前**：图没有 `compile_with_store`，Store 不自动注入编译后的图。
- **用法**：在业务侧创建 `Arc<dyn Store>`（InMemoryStore 或 SqliteStore），**在 Agent/节点内持有该 Store**，在 `run()` 里根据 `RunnableConfig::user_id`（或业务自己的用户标识）拼 namespace，调用 `store.put` / `store.get` / `store.list` / `store.search`。
- **设计目标**（见 16-memory-design §5.4）：后续可支持 `compile_with_store(store)`，图持有 `Option<Arc<dyn Store>>`，节点通过 config 拿到 user_id 拼 namespace 做读写。

---

## 4. 小结

| 项目 | 说明 |
|------|------|
| **抽象** | `Store` trait：put / get / list / search，按 namespace 隔离 |
| **不持久化** | `InMemoryStore`，进程内 HashMap，search 为 key/value 过滤 |
| **持久化（KV）** | `SqliteStore`（feature `sqlite`），SQLite 表 `store_kv`，search 为 key/value 过滤 |
| **持久化（语义）** | `LanceStore`（feature `lance`），LanceDB 存 vector + 元数据，search 为向量相似度检索 |
| **检索** | 默认：字符串过滤（key/value 包含）；可选：LanceStore 向量相似度（需 Embedder） |
| **向量** | 非必需；仅 LanceStore 等「语义检索」实现需要向量 |
| **集成** | 节点内持有 `Arc<dyn Store>`，用 `user_id` 等拼 namespace 做跨会话长期记忆 |

**相关文档**：[16-memory-design.md](16-memory-design.md)、[MEMORY_VS_LANGGRAPH_STORE.md](MEMORY_VS_LANGGRAPH_STORE.md)。

**开发计划**：见上文「开发计划」总表（P1.1～P5.4 细粒度任务）及每项详细方案与验收；完成一项可将表中对应「状态」更新为「完成」。

**测试**：`cargo test -p langgraph --features sqlite --test memory_sqlite`（SqliteStore）；LanceStore 实现后补充 `--features lance` 测试。
