# LangChain 记忆模块

本文对 **LangChain**（Python）的经典记忆模块做源码级梳理，**参考实现位于第三方代码**：上游 [langchain-ai/langchain](https://github.com/langchain-ai/langchain)。若在本地维护副本，可置于 `thridparty/langchain/`（项目 `.gitignore` 已忽略该目录），便于对照阅读与二次开发。

## 代码位置说明

| 层级 | 说明 |
|------|------|
| **上游仓库** | [github.com/langchain-ai/langchain](https://github.com/langchain-ai/langchain) |
| **本地 thirdparty** | 工作区根目录下 `thridparty/langchain/`（需自行 clone 或复制，与 README 中「本地代码」约定一致） |
| **Chat 历史抽象** | `libs/core/langchain_core/chat_history.py`：`BaseChatMessageHistory`、`InMemoryChatMessageHistory` |
| **Chain 用 Memory** | `libs/langchain/langchain/memory/`：`ConversationBufferMemory`、`ConversationBufferWindowMemory` 等 |
| **社区后端** | `libs/community/langchain_community/chat_message_histories/`：Redis、DynamoDB、Firestore、Streamlit 等 |

下文基于上游源码与官方文档展开，可与本地 **thridparty** 源码对照。

---

## 记忆架构总览

LangChain 的「记忆」在经典链路里主要由两部分组成：

1. **ChatMessageHistory**：负责**存储与读取**对话消息列表，是「存储层」抽象。
2. **Memory**（如 `ConversationBufferMemory`）：在 **Chain / Agent 里**把「输入/输出」与 ChatMessageHistory 打通，并提供 `load_memory_variables`、`save_context` 等接口，是「使用层」。

二者关系：Memory 内部持有一个 `BaseChatMessageHistory`（或可配置的 factory），在 `save_context` 时往 history 里追加消息，在 `load_memory_variables` 时从 history 取出消息并格式化为字符串或消息列表，供 Prompt/LLM 使用。

---

## ChatMessageHistory：存储层

### 1. BaseChatMessageHistory（langchain_core）

定义在 `langchain_core/chat_history.py`，是**抽象基类**，约定所有「聊天历史」后端的接口。

- **核心属性 / 方法**：
  - **messages**：返回当前会话的消息列表 `list[BaseMessage]`（可能触发 IO）。
  - **add_message(message)**：写入一条 `BaseMessage`。
  - **add_messages(messages)**：批量写入；默认实现逐条调 `add_message`，子类可重写以优化批量 IO。
  - **add_user_message / add_ai_message**：便捷方法，内部调 `add_message`，官方建议新代码优先用 `add_messages`。
  - **clear()**：清空当前会话历史。
  - 异步版本：`aget_messages`、`aadd_messages`、`aclear`，默认通过 `run_in_executor` 调用同步实现。

实现时的惯例：子类至少实现 `messages`、`add_messages`（或 `add_message`）、`clear`；若持久化后端支持批量写入，应重写 `add_messages` 以减少往返。

### 2. InMemoryChatMessageHistory（langchain_core）

同文件中提供的**进程内**实现：

- 使用 `list[BaseMessage]` 存消息，无落盘、无跨进程。
- 实现 `add_message`、`add_messages`、`clear`、`messages`，以及对应的 async 方法。
- 适用场景：单机开发、测试、无需持久化的对话。

在 **thridparty** 中对应文件：`libs/core/langchain_core/chat_history.py`。

### 3. langchain_community.chat_message_histories 中的后端

多种持久化/集成实现，均继承 `BaseChatMessageHistory`，例如：

| 类名 | 存储/用途 |
|------|------------|
| `ChatMessageHistory` | 常指「InMemory」或默认实现，具体以 community 包内导入为准 |
| `FileChatMessageHistory` | 按 session_id 存为本地文件 |
| `RedisChatMessageHistory` | Redis |
| `DynamoDBChatMessageHistory` | AWS DynamoDB |
| `FirestoreChatMessageHistory` | Google Firestore |
| `CosmosDBChatMessageHistory` | Azure Cosmos DB |
| `StreamlitChatMessageHistory` | Streamlit 的 session state |

使用时通过构造函数传入连接或路径等，得到可注入 Chain/Memory 的 `BaseChatMessageHistory` 实例。

---

## Memory：Chain 侧使用层

在 `langchain.memory` 中，Memory 负责在「人机多轮对话」场景下，把「当前输入/输出」与某条 ChatMessageHistory 绑定，并向 Chain 暴露两组接口：

- **load_memory_variables(**kwargs)**：根据当前会话历史，返回一个 dict（如 `{"history": "Human: ...\nAI: ..."}` 或 `{"chat_history": [BaseMessage, ...]}`），供 Prompt 或 LCEL 使用。
- **save_context(inputs, outputs)**：把本轮 `inputs` / `outputs` 中的人机内容写入背后的 ChatMessageHistory。

不同 Memory 类型的差异主要在「保留哪些、保留多少、以什么形式给 Prompt」。

### 1. ConversationBufferMemory

- **逻辑**：无界缓冲，把所有历史都放进 ChatMessageHistory，`load_memory_variables` 时全部取出，格式化为字符串（默认 `"Human: ...\nAI: ..."`）或消息列表（`return_messages=True`）。
- **特点**：实现简单，但对话一长容易超出模型 context 且 token 消耗大。
- **常见用法**：`memory = ConversationBufferMemory(chat_memory=my_chat_history, return_messages=True)`，再通过 `memory.load_memory_variables({})` 或 LCEL 的 `RunnableWithMessageHistory` 等注入到链中。

在 **thridparty** 中对应：`libs/langchain/langchain/memory/buffer.py`（或包内 `memory/` 下等价路径）。

### 2. ConversationBufferWindowMemory

- **逻辑**：只保留**最近 k 轮**人机交互；内部仍可用 ChatMessageHistory 存全量，但在 `load_memory_variables` 时只取最后 k 轮；或后端本身只保留 k 轮。
- **参数**：`k` 表示保留的「轮数」或「交互对数」。
- **特点**：在长对话中控制上下文长度与 token，适合窗口有限的模型。

在 **thridparty** 中对应：`libs/langchain/langchain/memory/buffer_window.py`。

### 3. 其他常见 Memory 类型

- **ConversationSummaryMemory**：用 LLM 对历史做摘要，`load_memory_variables` 返回摘要字符串，从而压缩上下文。
- **ConversationSummaryBufferMemory**：结合「最近 N 条原始消息 + 更早的摘要」。
- **ConversationEntityMemory**：按实体维护记忆，适合需要「记住人/物」的多轮对话。

具体类名与路径以 **thridparty** 中 `libs/langchain/langchain/memory/` 目录为准。

---

## 与 LangGraph 记忆的对比

更详细的架构、数据模型、API 与迁移对照见 **[13-langchain-vs-langgraph-memory.md](13-langchain-vs-langgraph-memory.md)**。以下为摘要：

| 维度 | LangChain（经典 Memory） | LangGraph（本系列 06/09/10） |
|------|---------------------------|-------------------------------|
| 会话粒度 | 通常由「session_id / chat_memory」区分，需在链或 Runnable 配置里传入 | 由 `thread_id` + Checkpointer 区分，每次 invoke 在 config 中传 `thread_id` |
| 存储抽象 | `BaseChatMessageHistory` + 各种后端 | Checkpointer（如 MemorySaver/Sqlite/Postgres）+ 可选 Store |
| 状态形态 | 以「消息列表」为主，Memory 再转为字符串或列表给 Prompt | 图状态（State）中的 `messages` channel + reducer（如 add_messages），以及自定义 channel |
| 长期/跨会话 | 需自行用 VectorStore、数据库或 Store 等扩展 | 原生支持 Store + namespace（如 `(user_id, "memories")`） |

若项目已用 LangGraph，会话内记忆更推荐直接使用 Checkpointer + `thread_id`；若仍在用经典 LangChain Chain/Agent，则通过 `ConversationBufferMemory` / `ConversationBufferWindowMemory` + 某一种 `BaseChatMessageHistory` 实现记忆，并与 **thridparty** 中 `langchain_core.chat_history`、`langchain.memory`、`langchain_community.chat_message_histories` 对照阅读。

---

## 与本系列文档的对应关系

| 文档 | 对应点 |
|------|--------|
| [06-memory-agent.md](06-memory-agent.md) | LangGraph 侧：会话记忆用 Checkpoint + `thread_id`；若要做「类似 LangChain Memory 的会话缓冲」，在 LangGraph 里等价于「带 add_messages 的 messages channel + Checkpointer」。 |
| [09-memory-chat-case.md](09-memory-chat-case.md) | 案例中的「会话内结构化记忆」在 LangChain 里可用「Memory + 自定义变量」或「Chain state 中额外字段」类比。 |
| [10-memory-deep-dive.md](10-memory-deep-dive.md) | LangGraph 的 Checkpoint/Store 机制；本文则专门写 **LangChain** 的 chat_history + memory 模块，可与 10 对照理解两套体系的差异。 |

---

## 小结表

| 主题 | 要点 |
|------|------|
| **代码位置** | 上游 [langchain-ai/langchain](https://github.com/langchain-ai/langchain)；本地可放到 `thridparty/langchain/` 做对照。 |
| **存储层** | `langchain_core.chat_history`：`BaseChatMessageHistory`（抽象）、`InMemoryChatMessageHistory`（内存实现）；其他后端在 `langchain_community.chat_message_histories`。 |
| **使用层** | `langchain.memory`：`ConversationBufferMemory`、`ConversationBufferWindowMemory` 等，内部依赖 ChatMessageHistory，通过 `load_memory_variables` / `save_context` 与 Chain 集成。 |
| **接口约定** | 存储层：`messages`、`add_messages`、`clear`（及 async）；使用层：`load_memory_variables`、`save_context`。 |
| **与 LangGraph** | LangChain 记忆是「ChatMessageHistory + Memory」；LangGraph 记忆是「Checkpointer + 图状态 + 可选 Store」，两者可并存于同一技术栈的不同组件中。 |

若要对照具体实现，可直接在 **thridparty** 下打开 `libs/core/langchain_core/chat_history.py`、`libs/langchain/langchain/memory/` 与 `libs/community/langchain_community/chat_message_histories/`，并结合官方 [Memory](https://python.langchain.com/docs/modules/memory/) 相关文档阅读。
