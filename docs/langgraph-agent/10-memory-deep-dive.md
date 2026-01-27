# LangGraph 记忆深度研究

本文对 LangGraph 的持久化与记忆机制做源码级梳理，**参考实现位于第三方代码**：上游 [langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)。若在本地维护副本，可置于 `thridparty/langgraph/`（项目 `.gitignore` 已忽略该目录），便于对照阅读与二次开发。

## 代码位置说明

| 层级 | 说明 |
|------|------|
| **上游仓库** | [github.com/langchain-ai/langgraph](https://github.com/langchain-ai/langgraph) |
| **本地 thirdparty** | 工作区根目录下 `thridparty/langgraph/`（README 中「本地代码」所指路径；目录被 gitignore，需自行 clone 或复制） |
| **Checkpoint 相关** | Python: `langgraph/checkpoint/`（含 `base`、`memory`、`serde`）；独立包 `langgraph-checkpoint`、`langgraph-checkpoint-sqlite`、`langgraph-checkpoint-postgres` 等 |
| **Store（跨 thread 记忆）** | `langgraph/store/`（如 `langgraph/store/memory.py` 的 `InMemoryStore`） |

下文基于官方 Persistence 文档与 API 约定展开，可与上游或本地 thirdparty 源码对照。

---

## 记忆与持久化架构总览

LangGraph 的「记忆」由两层组成：

1. **Thread + Checkpoint**：按 `thread_id` 保存的**图状态快照**，实现会话内记忆、时间旅行、人机回环与故障恢复。
2. **Store**：**跨 thread** 的键值/检索存储，用于「在所有对话中保留同一用户的信息」（如用户偏好、长期档案）。

二者配合方式：Checkpointer 在每个 super-step 把当前状态写入当前 thread；Store 由应用按 `(user_id, "memories")` 等 namespace 读写，与 thread 解耦。编译图时同时传入 `checkpointer` 与 `store` 即可在节点中同时使用两者。

---

## Checkpoint 机制深度剖析

### 1. Thread（线程/会话）

- **含义**：每个「会话」对应一个唯一 ID，即 `thread_id`。Checkpointer 以 `thread_id` 为主键存取所有该会话的 checkpoint。
- **配置**：调用 `invoke` / `stream` / `get_state` 等时，必须在 `config["configurable"]` 里提供 `thread_id`，例如：
  ```python
  config = {"configurable": {"thread_id": "user-123"}}
  graph.invoke(input, config)
  ```
- **可选**：`checkpoint_id` 指定从某一历史快照恢复或分支；`checkpoint_ns` 用于更细的命名空间（如子图）。

同一 `thread_id` 下会形成一条由多个 checkpoint 组成的链，对应「该会话的完整执行历史」。

### 2. Checkpoint 数据结构（StateSnapshot / 底层 Checkpoint）

用户侧看到的是 **StateSnapshot**，大致包含：

- **values**：当前时刻各 **channel** 的值（即「图状态」的可读快照）。
- **config**：包含 `thread_id`、`checkpoint_id`、`checkpoint_ns` 的配置。
- **metadata**：如 `source`（`"input"` | `"loop"` | `"update"` | `"fork"`）、`step`、`writes`（本步各节点写回内容）、`created_at`、`parent_config` 等。
- **next**：下一批要执行的节点名。
- **tasks**：后续任务（如与 interrupt 相关的信息）。

底层存储的 **Checkpoint** TypedDict 通常包含：

- **v**：格式版本（如 1）。
- **id**：单调递增的 checkpoint ID，用于排序与唯一标识。
- **ts**：ISO 8601 时间戳。
- **channel_values**：channel 名 → 反序列化后的值。
- **channel_versions**：每个 channel 的版本号，用于判断「该节点是否已看过该 channel 的当前版本」，从而决定是否要重算或合并。
- **versions_seen**：节点 ID → 已见过的 channel 版本，用于增量计算与去重。

也就是说：**持久化的是「channel 名 + 序列化后的值 + 版本信息」**，还原时再通过 Serializer 反序列化成 Python 对象。

### 3. Channel、Reducer 与 add_messages

- **Channel**：状态里的每个键（如 `messages`、`user_name`）对应一个 channel。  
- **Reducer**：对某 channel 的「多次写入」如何合并。若未指定 reducer，则后一次覆盖前一次；若指定为 `add_messages`，则新消息**追加**到列表，不覆盖。  
- **add_messages**：LangGraph 内置的 reducer，专用于 `Sequence[BaseMessage]`，会把节点返回的 `messages` 与已有列表按消息顺序合并，从而自然形成「会话内消息历史」。

因此：**会话内记忆** = 使用带 `add_messages` 的 `messages` channel + Checkpointer 按 `thread_id` 持久化。每轮 invoke 只需传入**本轮的若干条新消息**，历史由 checkpoint 自动带上。

### 4. Checkpointer 接口（BaseCheckpointSaver）

所有 Checkpointer 实现都遵循 **BaseCheckpointSaver** 的接口，核心方法包括：

- **put(config, checkpoint, metadata)**：写入一个 checkpoint。
- **put_writes(config, writes, task_id)**：写入某 step 的中间 writes（用于故障恢复、pending writes）。
- **get_tuple(config)**：按 `thread_id`（及可选的 `checkpoint_id`）取回一个 checkpoint 元组（含 checkpoint 与 metadata）。
- **list(config, filter?, limit?, before?, after?)**：列出符合条件的 checkpoint，用于 `get_state_history`、时间旅行等。

异步版本命名为 `aput`、`aput_writes`、`aget_tuple`、`alist`；若用 `ainvoke`/`astream`，运行时将优先调用异步接口。

### 5. MemorySaver / InMemorySaver

- **名称**：不同文档/版本中可能称为 `MemorySaver` 或 `InMemorySaver`（例如官方 persistence 文档示例中使用 `InMemorySaver`）。若本地 thirdparty 或 `langgraph-checkpoint` 里类名为 `MemorySaver`，可视为同族实现，以本地包导入为准。
- **实现要点**：
  - 在进程内用字典结构按 `(thread_id, checkpoint_ns, checkpoint_id)` 存储序列化后的 blob 与元数据。
  - 不落盘、不跨进程，进程退出即丢失。
- **用途**：开发、单机演示、自动化测试。生产环境应使用 **SqliteSaver**、**PostgresSaver** 等持久化实现。

### 6. 序列化（Serializer）

Checkpoint 在写入存储前会把 channel 值序列化成字节。`langgraph-checkpoint` 定义 **SerializerProtocol**，默认实现通常为 **JsonPlusSerializer**（支持 ormsgpack/JSON 及 LangChain/LangGraph 常见类型）。

- **pickle 回退**：若状态中含有不支持的类型（如 Pandas DataFrame），可在构建 Checkpointer 时传入 `JsonPlusSerializer(pickle_fallback=True)`。
- **加密**：可通过 **EncryptedSerializer**（如 `from_pycryptodome_aes()`）在落盘前加密，适配生产合规需求。

---

## Store：跨 Thread 的记忆

当需要「同一用户在不同 thread（不同次对话）之间共享信息」时，仅靠 Checkpointer 不够，需要 **Store**。

### 1. 概念与用途

- **Store**：按 **namespace** 组织的键值/检索存储，与 `thread_id` 解耦。例如用 `(user_id, "memories")` 作为 namespace，则同一 `user_id` 下所有 thread 都可读写同一批记忆。
- **典型用法**：用户在多次对话中说过「我喜欢披萨」；用 Store 按 `user_id` 写入，之后任意新 thread 只要带相同 `user_id`，节点里就能从 Store 中查回这些偏好。

### 2. 基本 API（InMemoryStore 为例）

- **namespace**：元组，如 `(user_id, "memories")`，长度与含义由业务自定。
- **put(namespace, key, value)**：写入一条记忆；`key` 常取 UUID，`value` 为 dict 等可序列化结构。
- **search(namespace, query?, limit?, ...)**：在某一 namespace 内搜索。若配置了 **index**（embedding 模型 + 维度 + 要嵌入的字段），则支持按自然语言 **query** 的语义检索；否则多为按 key 或全量扫描。

节点中通过注入 **BaseStore** 与 **RunnableConfig** 使用 Store；`user_id` 等可从 `config["configurable"]` 中读取，再拼成 namespace。

### 3. 与 Checkpointer 的配合

- **编译**：`graph.compile(checkpointer=..., store=...)`
- **调用**：`config = {"configurable": {"thread_id": "...", "user_id": "..."}}`
- **节点**：  
  - 会话内历史 → 仍来自 state（由 Checkpointer 按 thread 持久化的 `messages` 等）。  
  - 跨会话记忆 → 在节点内调用 `store.search(namespace, query=..., limit=...)` 或 `store.put(...)`，再注入到系统提示或上下文里。

---

## 与本系列文档的对应关系

| 文档 | 对应点 |
|------|--------|
| [06-memory-agent.md](06-memory-agent.md) | 会话记忆用 Checkpoint + `thread_id`；结构化记忆（如 `user_name`、`user_preferences`）放在 state 的 channel 里，由 Checkpointer 一并持久化；语义记忆对应 Store + 向量检索（或自建 VectorStore）。 |
| [09-memory-chat-case.md](09-memory-chat-case.md) | 旅行助手案例：同一 `thread_id` 下用 `extract_memory` 更新 state 中的 `user_name` / `preferences` / `last_topic`，再由 `chat_node` 拼系统提示；若要做「跨会话长期偏好」，可把部分信息写入 Store，namespace 用 `(user_id, "preferences")` 等。 |

---

## 小结表

| 主题 | 要点 |
|------|------|
| **代码位置** | 上游 [langchain-ai/langgraph](https://github.com/langchain-ai/langgraph)；本地可放到 `thridparty/langgraph/` 做对照。 |
| **会话内记忆** | `thread_id` + Checkpointer + 带 `add_messages` 的 `messages` channel；每步状态写入 checkpoint，下一轮自动加载。 |
| **Checkpoint 内容** | channel_values（序列化）、channel_versions、id/ts/metadata；通过 BaseCheckpointSaver 的 put/get_tuple/list 存取。 |
| **MemorySaver/InMemorySaver** | 进程内字典实现，适合开发与测试；生产改用 SqliteSaver / PostgresSaver。 |
| **跨会话记忆** | Store + namespace（如 `(user_id, "memories")`）；可配 index 做语义检索，在节点中与 config 一起注入使用。 |
| **序列化与安全** | SerializerProtocol / JsonPlusSerializer；可选 pickle_fallback、EncryptedSerializer。 |

若要对照具体实现，可直接在 **thridparty** 下打开 `langgraph/checkpoint/` 与 `langgraph/store/` 的源码，与官方 [Persistence](https://docs.langchain.com/oss/python/langgraph/persistence) 与 [Add memory](https://docs.langchain.com/oss/python/langgraph/add-memory) 文档一起阅读。
