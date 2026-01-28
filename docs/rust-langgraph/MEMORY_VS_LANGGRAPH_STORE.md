# rust-langgraph Memory 与 LangGraph MemoryStore 对照

本文对照 **rust-langgraph** 的记忆模块与 **LangGraph (Python)** 的「记忆」与 **Store** 体系，说明概念对应关系和实现差异。

## 1. LangGraph 的记忆与存储架构

LangGraph 官方把记忆分为两层（见 [Add Memory](https://docs.langchain.com/oss/python/langgraph/add-memory)）：

| 类型 | 机制 | 作用域 | 典型用途 |
|------|------|--------|----------|
| **短期记忆** | Checkpointer + `thread_id` + state 的 `messages` channel | 单会话（thread） | 多轮对话历史、图状态快照、时间旅行 |
| **长期记忆** | **Store**（如 `InMemoryStore`）+ namespace | 跨 thread / 跨会话 | 用户偏好、长期档案、跨会话检索 |

注意命名容易混淆：

- **MemorySaver / InMemorySaver**：属于 **checkpoint** 包，实现 **BaseCheckpointSaver**，存的是 **thread 级状态快照**（含 `messages` 等 channel），不是「跨会话记忆」。
- **Store / InMemoryStore**：属于 **store** 包，实现 **BaseStore**，才是「跨 thread 记忆」的载体；文档里说的「Memory Store」通常指这一类。

因此：**LangGraph 的「MemoryStore」指的是 Store（BaseStore / InMemoryStore）**，用于长期、跨会话记忆；短期记忆由 Checkpointer + state 承担。

## 2. rust-langgraph 的记忆架构

rust-langgraph 在 `crates/langgraph/src/memory/` 下提供：

| 抽象 / 实现 | 职责 | 对应 LangGraph 的 |
|-------------|------|--------------------|
| **Memory** trait | 消息列表的增删查：`add`, `get`, `clear`, `count` | 会话内「消息列表」这一概念（由 Checkpointer 按 thread 持久化的 `messages` channel） |
| **SessionMemory** | 进程内 FIFO 消息列表，带容量上限 | 短期记忆的「进程内实现」，相当于「无持久化的 messages 列表」 |
| **SemanticMemory** trait | 按向量存储 + 相似度检索：`add(content, embedding)`, `search(query_embedding, top_k)` | Store 的 **语义检索** 能力（InMemoryStore 配 `index={"embed", "dims"}` 时的 `search`） |
| **VectorMemory** | 内存向量存储，余弦相似度 `search` | 长期记忆里「仅语义检索」那一部分的进程内实现 |

当前 rust-langgraph **没有** 与 LangGraph **BaseStore** 一一对应的「通用 Store」抽象（即带 namespace 的 put/get/search、可插语义索引的那种）。语义检索由 **SemanticMemory / VectorMemory** 专门承担。

## 3. 概念对照表

| LangGraph (Python) | rust-langgraph |
|--------------------|----------------|
| 短期记忆：Checkpointer + `thread_id` + `messages` channel | **Memory** + **SessionMemory**（当前仅进程内，无通用 Checkpointer） |
| InMemorySaver（checkpoint 进程内实现） | 若将来做 Checkpointer，可视为「图状态 + 会话」的进程内实现 |
| 长期记忆：**Store**（BaseStore） | 暂无「Store」trait；长期、跨会话语义由 **SemanticMemory / VectorMemory** 覆盖 |
| InMemoryStore | 可类比为「某种实现 SemanticMemory 的进程内存储」；命名上 LangGraph 的 Store 更通用（namespace + put/search），rust 侧是「仅语义检索」的专门抽象 |
| InMemoryStore + `index={"embed", "dims"}` 的 `search(namespace, query=..., limit=...)` | **SemanticMemory::search(query_embedding, top_k)**（无 namespace，多会话需在上层用 `user_id` 等自己分片） |
| Store 的 `put(namespace, key, value)` | 无直接对应；若只存「可嵌入文本」，可用 **SemanticMemory::add(content, embedding)**，key/namespace 需业务层维护 |

## 4. API 对照（长期记忆 / 语义部分）

- **LangGraph Store（带语义检索）**  
  - `store.put(("user_id", "memories"), key, {"text": "..."})`  
  - `store.search(("user_id", "memories"), query="...", limit=K)`  
  - namespace 在 Store 内建，embedding 由 Store 的 index 配置完成。

- **rust-langgraph SemanticMemory**  
  - `semantic_memory.add(content, embedding)`（无 namespace，key 由实现内部生成，如 VectorMemory 的自增 id）  
  - `semantic_memory.search(query_embedding, top_k)`  
  - 若要多用户/多会话，需要在业务层为每个 `(user_id, namespace)` 建不同的 `SemanticMemory` 实例，或在自己的「Store 层」里用 HashMap 等按 namespace 分片到多个 VectorMemory。

## 5. 小结与可扩展方向

- **短期记忆**：rust-langgraph 的 **Memory / SessionMemory** 与 LangGraph 的「Checkpointer + messages channel」在「会话内消息列表」这一职责上对齐；LangGraph 还多了 thread、checkpoint_id、时间旅行等，rust 侧若要做对等能力，需要引入 Checkpointer 抽象和 thread 概念。
- **长期记忆**：LangGraph 的 **MemoryStore（Store）** 是「namespace + put/search + 可选语义索引」的通用抽象；rust-langgraph 用 **SemanticMemory / VectorMemory** 覆盖了「按向量检索」这一块，没有泛化到「任意 namespace + 键值 + 语义」的 Store。
- 若要在 rust-langgraph 中逼近 LangGraph 的 MemoryStore 使用方式，可选方向包括：  
  1）在现有 **SemanticMemory** 之上再抽象一层「按 namespace 分片的 Store」；或  
  2）新增一个 **Store** trait，提供 `put(namespace, key, value)`、`search(namespace, query?, limit)`，再为「仅键值」和「带向量索引」各做一个实现，从而与 LangGraph 的 BaseStore 在概念上对齐。

上述对照基于当前（文档编写时）的 rust-langgraph 与 LangGraph 官方文档/实现；若某一侧 API 或模块划分有变更，以各自仓库与官方文档为准。
