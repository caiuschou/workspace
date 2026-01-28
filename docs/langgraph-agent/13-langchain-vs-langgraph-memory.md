# LangChain 与 LangGraph 记忆深度比较

本文对 **LangChain** 与 **LangGraph** 的「记忆」机制做深度对比，从架构、数据模型、API、生命周期到选型与迁移逐一展开。  
前置阅读：[10-memory-deep-dive.md](10-memory-deep-dive.md)（LangGraph）、[11-langchain-memory.md](11-langchain-memory.md)（LangChain）。

---

## 1. 架构总览对比

### 1.1 LangChain：存储层 + 使用层

LangChain 的记忆分为两层，**职责分离**：

| 层级 | 抽象 | 职责 |
|------|------|------|
| **存储层** | `BaseChatMessageHistory` | 消息的增删查清（`messages`、`add_messages`、`clear`），可换多种后端（内存、Redis、DynamoDB 等） |
| **使用层** | Memory（如 `ConversationBufferMemory`） | 在 Chain/Agent 中把「输入/输出」与 ChatMessageHistory 打通，通过 `load_memory_variables`、`save_context` 向 Prompt/LLM 提供「历史」 |

关系：Memory 内部持有（或通过 factory 获得）一个 `BaseChatMessageHistory`，在 **save_context** 时把本轮人机内容写入 history，在 **load_memory_variables** 时从 history 取出并格式化为字符串或消息列表，供 LCEL/Prompt 使用。  
**会话粒度**由应用在构造 Memory 或 Runnable 时传入的 **session_id / chat_memory 实例** 决定，不同 session 对应不同的 ChatMessageHistory 实例或配置。

### 1.2 LangGraph：图状态持久化 + 可选跨会话存储

LangGraph 的记忆同样分为两层，但**绑定在图与执行模型**上：

| 层级 | 抽象 | 职责 |
|------|------|------|
| **会话内** | Checkpointer + 图状态（State） | 按 `thread_id` 持久化**整图状态快照**（含各 channel 的值）；每次 invoke/stream 在 config 中传 `thread_id`，历史状态自动加载与回写 |
| **跨会话** | Store（可选） | 按 namespace（如 `(user_id, "memories")`）做键值/语义检索，与 thread 解耦，用于长期用户信息 |

关系：**会话内记忆** = 图状态里的 channel（尤其是带 `add_messages` 的 `messages`）+ Checkpointer。不单独存在「ChatMessageHistory」抽象，消息列表是 State 的一个 channel，由 Checkpointer 按 thread 一并持久化。**会话粒度**由每次调用时的 **thread_id**（在 `config["configurable"]["thread_id"]`）唯一决定。

### 1.3 对照小结

| 维度 | LangChain | LangGraph |
|------|-----------|-----------|
| 核心抽象 | ChatMessageHistory + Memory | Checkpointer + State channels + 可选 Store |
| 会话粒度 | session_id / chat_memory 实例 | thread_id（invoke 时必传） |
| 持久化单位 | 「消息列表」为主 | 「图状态快照」含所有 channel |
| 与执行模型关系 | 需在 Chain 中显式 load/save | 与图的 invoke/stream 深度集成，自动加载/写回 |

---

## 2. 数据模型对比

### 2.1 存什么

- **LangChain**  
  - 存储层存的是 **消息列表** `list[BaseMessage]`（Human/AI/System 等）。  
  - Memory 的 `load_memory_variables` 可返回字符串（如 `"Human: ...\nAI: ..."`）或 `return_messages=True` 时的消息列表。  
  - 不持久化「Chain 的其它状态」；若要做「用户偏好、上一次话题」等，需在 Chain 自己的 state 或额外 Memory/变量里维护，并自行与 ChatMessageHistory 或其它存储对接。

- **LangGraph**  
  - Checkpointer 存的是 **整图状态的快照**：每个 **channel**（如 `messages`、`user_name`、`preferences`）的当前值、版本号、元数据等。  
  - 因此「消息列表」只是其中一个 channel（通常用 `add_messages` 做追加）；其它 channel 如 `user_name`、`last_topic` 会一并持久化在同一 thread 下。  
  - 一次 checkpoint = 该 thread 在某一步的完整状态，便于时间旅行、分支、回放。

### 2.2 数据形态对照

| 项目 | LangChain | LangGraph |
|------|-----------|-----------|
| 最小存储单位 | 单条消息（append 到 history） | 整图状态（所有 channel 一起写一个 checkpoint） |
| 消息在其中的角色 | history 的唯一/主内容 | state 中一个 channel 的值，可与其它 channel 并列 |
| 结构化记忆（名字、偏好等） | 需自建（如额外变量、VectorStore） | 可直接作为 state 的 channel，由 Checkpointer 一起持久化 |
| 版本与历史 | 无内置「版本」概念，仅「消息顺序」 | checkpoint 有 id/ts、channel_versions、list() 可做历史与回溯 |

---

## 3. 会话内记忆：能力与等价关系

### 3.1 LangChain 的几种 Memory

| 类型 | 行为 | 典型用法 |
|------|------|----------|
| **ConversationBufferMemory** | 无界缓冲，全部历史给 Prompt | 对话短、或做简单 demo |
| **ConversationBufferWindowMemory** | 只保留最近 k 轮 | 控制 token、长对话 |
| **ConversationSummaryMemory** | 用 LLM 把历史压成摘要 | 压缩上下文 |
| **ConversationSummaryBufferMemory** | 最近 N 条原始 + 更早的摘要 | 折中长度与细节 |
| **ConversationEntityMemory** | 按实体维护记忆 | 多轮中要「记住谁/什么」 |

共同点：底层都是 **ChatMessageHistory**；差异在「保留哪些、保留多少、以什么形式通过 load_memory_variables 给 Prompt」。

### 3.2 LangGraph 中的等价与超越

- **「等价于 Buffer」**：  
  使用带 **add_messages** 的 `messages` channel + Checkpointer。每轮只往 state 里追加本轮消息，历史完全由上一 checkpoint 带来，等价于「无界缓冲」。  
  若需要「只给 LLM 最近 k 条」，可在节点里对 `state["messages"]` 做切片再传入 LLM，或在 state 外接一层「滑动窗口」逻辑。

- **「等价于 Summary / SummaryBuffer」**：  
  可在图里加「摘要节点」：读 `state["messages"]`，调 LLM 写摘要，把结果写入例如 `state["context_summary"]` 或单独 channel；下一节点用「摘要 + 最近 N 条」拼上下文。  
  摘要本身也作为 state 一部分被 Checkpointer 持久化，无需单独存储。

- **「等价于 Entity / 结构化记忆」**：  
  如 [06-memory-agent.md](06-memory-agent.md)、[09-memory-chat-case.md](09-memory-chat-case.md) 所示，直接把 `user_name`、`preferences`、`last_topic` 等定义为 state 的 channel，由 extract 节点更新、由 chat 节点注入系统提示；Checkpointer 会一并持久化，无需自建「Entity Memory」组件。

- **超越 LangChain 之处**：  
  - 时间旅行、从任意 checkpoint 恢复或分支；  
  - 人机回环、中断与继续；  
  - 一次持久化包含「消息 + 所有结构化记忆」，语义一致、实现简单。

### 3.3 对照表（会话内）

| 能力 | LangChain | LangGraph |
|------|-----------|-----------|
| 无界消息历史 | ConversationBufferMemory + ChatMessageHistory | messages channel + add_messages + Checkpointer |
| 最近 k 轮 | ConversationBufferWindowMemory | 节点内对 state["messages"] 切片或自建窗口 |
| 摘要压缩 | ConversationSummaryMemory / SummaryBufferMemory | 图中摘要节点 + 新 channel，由 Checkpointer 持久化 |
| 实体/结构化记忆 | ConversationEntityMemory 或自建 | state 多 channel（user_name、preferences 等）+ Checkpointer |
| 时间旅行 / 从某步恢复 | 无 | get_state / list + checkpoint_id |
| 与图执行耦合 | 需在链中显式 load/save | 与 invoke/stream 一体，config 传 thread_id 即可 |

---

## 4. 长期 / 跨会话记忆对比

### 4.1 LangChain

- 标准 Memory 只覆盖「**单会话内的消息与变量**」。  
- **跨会话、长期**（如「该用户在所有对话里的偏好」）需要自己扩展：  
  - 另接 VectorStore、关系型库或缓存，在 Chain 里读写；  
  - 或使用 LangChain 生态里的「记忆」相关扩展（若后有 Store 类抽象，也属额外组件）。  
- 没有内置的「namespace + 跨 thread 键值/检索」抽象，会话边界和长期存储的边界都由应用自己约定与实现。

### 4.2 LangGraph

- 提供 **Store**（如 `InMemoryStore`），按 **namespace**（如 `(user_id, "memories")`）做 **put/search**。  
- 可与 **index** 配合做语义检索（embedding + 向量搜索），适合「长期记忆 + 按语义召回」。  
- 与 thread 解耦：同一 `user_id` 下多个 thread 都可读写同一 namespace，实现「跨会话长期记忆」。  
- 编译时传入 `store=...`，节点内通过 `InjectedState` 或传入的 `BaseStore` + `config` 使用。

### 4.3 对照表（长期/跨会话）

| 维度 | LangChain | LangGraph |
|------|-----------|-----------|
| 是否有原生「跨会话」抽象 | 无，需自建 | 有，Store + namespace |
| 典型实现方式 | VectorStore、自建 DB、缓存 | Store（InMemoryStore / 持久化实现）+ 可选 index |
| 与「会话」的关系 | 应用自己用 user_id/session_id 区分 | namespace 常含 user_id，与 thread_id 分离 |
| 语义检索 | 自接 VectorStore | Store 的 index + search(namespace, query=..., limit=...) |

---

## 5. API 与调用流程对比

### 5.1 LangChain：load → 调用链 → save

1. 链或 Runnable 在**执行前**调用 Memory 的 **load_memory_variables**（通常传入当前 session 等），得到 `{"history": "..."}` 或 `{"chat_history": [BaseMessage, ...]}`。  
2. 将该结果注入 Prompt 或 LCEL 的输入。  
3. 链执行完毕，调用 Memory 的 **save_context(inputs, outputs)**，把本轮人机内容写入背后的 ChatMessageHistory。  
4. 会话标识（session_id、user_id 等）需要由调用方在 **RunnableConfig** 或 Memory 的工厂里传入，以选择对应的 ChatMessageHistory。

### 5.2 LangGraph：config 驱动，自动加载与写回

1. 调用方在 **config["configurable"]** 中提供 **thread_id**（以及可选的 `checkpoint_id`、`user_id` 等）。  
2. **invoke(input, config)** 或 **stream(...)** 时，运行时会：  
   - 用 Checkpointer 根据 thread_id 加载该 thread 的**最新状态**（或指定 checkpoint）；  
   - 把本轮 **input** 与已有 state 合并（例如新消息经 add_messages 追到 messages）；  
   - 执行图，结束时把新 state 写回 Checkpointer。  
3. **不需要**在节点里显式「从某处 load 历史、再写回」：历史就是 state，state 由框架按 thread 自动持久化。  
4. 若用 Store，节点内从 **config** 取 `user_id` 等，拼 namespace，再调 **store.put / store.search**。

### 5.3 对照表（API 与流程）

| 项目 | LangChain | LangGraph |
|------|-----------|-----------|
| 会话标识传入方式 | session_id 等通过 config 或 Memory 工厂 | config["configurable"]["thread_id"] 必传 |
| 历史如何进入本次调用 | load_memory_variables 返回，再由开发者注入 Prompt | 上一 checkpoint 的 state 自动作为本次输入的一部分 |
| 本轮结果如何持久化 | save_context(inputs, outputs) 写回 ChatMessageHistory | 图结束时的 state 由 Checkpointer 自动写回 |
| 是否必须显式 load/save | 是 | 否（仅 Store 需在节点里显式 put/search） |

---

## 6. 生命周期与持久化后端

### 6.1 LangChain

- **ChatMessageHistory** 的生命周期由**后端**决定：  
  - InMemory：进程内，进程结束即丢；  
  - Redis/DynamoDB/Firestore/File 等：按各自配置持久化，会话范围由 session_id 等区分。  
- **Memory** 本身无独立「持久化实现」，只是使用层的接口，真实持久化在 ChatMessageHistory 的实现里。

### 6.2 LangGraph

- **Checkpointer** 决定会话内状态的持久化：  
  - MemorySaver / InMemorySaver：进程内，重启即丢；  
  - SqliteSaver / PostgresSaver：落盘，支持多进程，适合生产。  
- **Store** 独立于 Checkpointer：InMemoryStore 为进程内；若有持久化 Store 实现，则长期、跨会话由 Store 负责。  
- 注意命名：**MemorySaver 属于 checkpoint 体系**，存的是「图状态快照」，不是「跨会话记忆」；**Store（如 InMemoryStore）** 才是「跨 thread 记忆」的载体。

### 6.3 对照表（后端与生命周期）

| 项目 | LangChain | LangGraph |
|------|-----------|-----------|
| 会话内持久化 | ChatMessageHistory 的后端（Redis/DB/File 等） | Checkpointer 实现（MemorySaver / Sqlite / Postgres） |
| 跨会话持久化 | 自建（DB、VectorStore 等） | Store 实现（InMemoryStore 或持久化 Store） |
| 进程内开发用 | InMemoryChatMessageHistory | MemorySaver + InMemoryStore |

---

## 7. 选型与迁移建议

### 7.1 何时选 LangChain Memory

- 仍在使用 **LangChain 的 Chain / 旧版 Agent**，且不打算全面迁到 LangGraph。  
- 只需要「单会话内消息历史 + 简单变量」，不需要时间旅行、人机回环、复杂状态。  
- 希望沿用现有 **ChatMessageHistory 后端**（如已有 Redis、DynamoDB 集成）。

### 7.2 何时选 LangGraph 记忆

- 已采用或打算采用 **LangGraph** 做 Agent/对话。  
- 需要 **会话内结构化记忆**（用户名、偏好、摘要等）与消息历史**一起**持久化、一起回溯。  
- 需要 **时间旅行、从某步恢复、分支** 等能力。  
- 希望 **跨会话长期记忆**有统一抽象，少做一层「自建 Store」的胶水代码。

### 7.3 从 LangChain Memory 迁到 LangGraph 的记忆

1. **会话边界**：将原来的 session_id（或等价物）映射为 **thread_id**，在所有 invoke/stream 的 config 里传 `thread_id`。  
2. **消息历史**：  
   - 用带 **add_messages** 的 **messages** channel 表示「聊天历史」；  
   - 选一个 Checkpointer（如 SqliteSaver）替代原来的 ChatMessageHistory 后端；  
   - 不再调用 load_memory_variables / save_context，历史完全由 state + Checkpointer 维护。  
3. **BufferWindow / Summary / Entity**：  
   - 窗口：在节点内对 `state["messages"]` 取最近 k 条；  
   - 摘要：图中加摘要节点，写出到 `context_summary` 等 channel；  
   - 实体/结构化：用多个 state channel + extract 节点实现，与 [06-memory-agent.md](06-memory-agent.md)、[09-memory-chat-case.md](09-memory-chat-case.md) 一致。  
4. **长期记忆**：  
   - 若原来自建 VectorStore/DB，可逐步收口到 LangGraph 的 **Store** + namespace；  
   - user_id 从 config 取，namespace 用 `(user_id, "memories")` 或类似，与现有「按用户存长期信息」的模型对齐。

---

## 8. 小结表

| 主题 | LangChain | LangGraph |
|------|-----------|-----------|
| **核心抽象** | ChatMessageHistory（存）+ Memory（用） | Checkpointer（会话内）+ Store（跨会话，可选） |
| **会话粒度** | session_id / chat_memory | thread_id |
| **持久化单位** | 消息列表 | 图状态快照（所有 channel） |
| **结构化记忆** | 需自建或用 EntityMemory 等 | state 多 channel，与消息一起由 Checkpointer 持久化 |
| **长期/跨会话** | 需自建 VectorStore/DB | Store + namespace，可配语义检索 |
| **API 风格** | load_memory_variables + save_context，显式注入 | 以 thread_id 驱动，state 自动加载与写回 |
| **适用场景** | 经典 Chain/Agent、简单会话记忆 | LangGraph 图、需状态/回溯/长期记忆的 Agent |

两者可并存于同一技术栈（例如部分服务用 LangChain Memory，部分用 LangGraph）；若新功能以 LangGraph 为主，建议会话内记忆统一到 Checkpointer + state，长期记忆统一到 Store，以减少概念与实现上的分裂。
