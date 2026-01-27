# 案例：带记忆的对话 Agent

本文通过一个**旅行助手**场景，说明如何实现「能记住用户信息、在多轮对话中持续使用」的对话 Agent。

## 案例背景

**场景**：用户与旅行助手对话，希望助手记住自己的名字、偏好和此前聊过的内容，在后续对话中直接使用，无需重复说明。

**目标**：

- 会话内记忆：同一 `thread_id` 下，历史消息、提炼出的用户信息都会保留。
- 结构化记忆：除原始消息外，额外维护「用户名」「偏好」「上次话题」等字段，方便在系统提示里注入，让 LLM 自然用到。

## 状态与记忆设计

本案例在状态里增加若干“记忆字段”，在每轮对话前先更新这些字段，再让 LLM 基于「系统提示 + 记忆 + 本轮消息」生成回复。

| 字段 | 含义 | 示例 |
|------|------|------|
| `messages` | 对话历史（由 Checkpoint 持久化） | 全部 Human/AI 消息 |
| `user_name` | 从对话中识别出的用户称呼 | "小明" |
| `preferences` | 用户明确说过的偏好 | `{"座位": "靠窗", "饮食": "素食"}` |
| `last_topic` | 上一轮主要话题摘要 | "想去日本赏樱" |

## 完整实现

```python
from typing import Annotated, Sequence, TypedDict
from langchain_core.messages import BaseMessage, HumanMessage, SystemMessage, AIMessage
from langgraph.checkpoint.memory import MemorySaver
from langgraph.graph import StateGraph, START, END
from langgraph.graph.message import add_messages
from langchain_openai import ChatOpenAI
import re
import json

# ---------- 状态定义 ----------
class TravelAssistantState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]
    user_name: str
    preferences: dict  # 如 {"座位": "靠窗", "饮食": "素食"}
    last_topic: str


def extract_memory(state: TravelAssistantState) -> dict:
    """从最新一条用户消息中抽取可记忆的信息，更新状态中的记忆字段。"""
    updates = {}
    last = state["messages"][-1] if state["messages"] else None
    if not last or not isinstance(last, HumanMessage):
        return updates

    text = last.content.strip()

    # 解析用户名："我叫XX" / "你可以叫我XX"
    for pattern in [r"我叫[（(]?(\w+)[）)]?", r"你可以叫我[（(]?(\w+)[）)]?", r"我是[（(]?(\w+)[）)]"]:
        m = re.search(pattern, text)
        if m:
            updates["user_name"] = m.group(1).strip()
            break

    # 解析偏好（示例：用简单键值或「偏好：…」句式）
    prefs = dict(state.get("preferences") or {})
    if "靠窗" in text or "窗边" in text:
        prefs["座位"] = "靠窗"
    if "素食" in text or "不吃肉" in text:
        prefs["饮食"] = "素食"
    if "经济舱" in text:
        prefs["舱位"] = "经济舱"
    if prefs:
        updates["preferences"] = prefs

    # 简短话题摘要：取用户句子的前 30 字作为 last_topic（实际可用 LLM 做摘要）
    if len(text) > 5:
        updates["last_topic"] = text[:30] + ("..." if len(text) > 30 else "")

    return updates


def chat_node(state: TravelAssistantState) -> dict:
    """根据当前状态（含记忆）调用 LLM 生成回复。"""
    llm = ChatOpenAI(model="gpt-4o", temperature=0.3)

    name = state.get("user_name") or ""
    prefs = state.get("preferences") or {}
    topic = state.get("last_topic") or ""

    system_parts = [
        "你是旅行助手。请根据对话历史和用户已提供的信息回答，不要臆造用户没说过的偏好或名字。"
    ]
    if name:
        system_parts.append(f"当前用户希望被称呼为：{name}。")
    if prefs:
        system_parts.append(f"用户已表达过的偏好：{json.dumps(prefs, ensure_ascii=False)}。")
    if topic:
        system_parts.append(f"上一轮或当前话题概要：{topic}。")

    system_msg = SystemMessage(content="\n".join(system_parts))
    all_messages = [system_msg] + list(state["messages"])
    response = llm.invoke(all_messages)

    return {"messages": [response]}


def create_memory_chat_agent():
    builder = StateGraph(TravelAssistantState)

    def memory_then_chat(state: TravelAssistantState) -> dict:
        """先更新记忆，再生成回复；为简化图结构，在一个节点内完成。"""
        updates = extract_memory(state)
        # 先更新状态中的记忆字段（仅当本节点负责整轮逻辑时这样写）
        # 若拆成多节点：第一个节点只做 extract_memory 写回，第二个节点做 chat_node
        state_with_memory = {**state, **updates}
        return {**chat_node(state_with_memory), **updates}

    builder.add_node("assistant", memory_then_chat)
    builder.add_edge(START, "assistant")
    builder.add_edge("assistant", END)

    return builder.compile(checkpointer=MemorySaver())
```

## 对话示例

下面用同一 `thread_id` 连续调用，演示记忆如何生效。

```python
agent = create_memory_chat_agent()
config = {"configurable": {"thread_id": "travel-user-001"}}

# 第 1 轮：用户介绍自己与偏好
r1 = agent.invoke(
    {"messages": [HumanMessage(content="我叫小明，下次订票请帮我选靠窗，我吃素食。")]},
    config=config,
)
print(r1["messages"][-1].content)
# 助手可能回复："好的小明，已记住：座位优先靠窗、饮食为素食。下次订票时会按这些来推荐。"

# 第 2 轮：只问“刚才你记住了什么”，不重复说名字和偏好
r2 = agent.invoke(
    {"messages": [HumanMessage(content="刚才你记住了我的哪些信息？")]},
    config=config,
)
print(r2["messages"][-1].content)
# 应提到：名字是小明、靠窗、素食等（说明记忆被正确注入到上下文中）

# 第 3 轮：基于记忆做推荐
r3 = agent.invoke(
    {"messages": [HumanMessage(content="我想下个月去日本看樱花，有适合的行程或航班推荐吗？")]},
    config=config,
)
print(r3["messages"][-1].content)
# 回答中应自然带上“小明”“靠窗”“素食”等已记忆信息，例如推荐航班时提到靠窗座位、餐饮考虑素食
```

## 流程说明

1. **入口**：每次 `invoke` 传入 `messages`（通常只追加本轮一条 `HumanMessage`）和带 `thread_id` 的 `config`。
2. **持久化**：`MemorySaver()` 按 `thread_id` 存 Checkpoint，因此 `messages` 与上一步写回的 `user_name` / `preferences` / `last_topic` 会在同一线程内累积。
3. **记忆更新**：`extract_memory(state)` 基于**当前状态里的最后一条用户消息**更新三个记忆字段，在调用 LLM 前写回状态。
4. **调用 LLM**：`chat_node` 用「系统提示（含记忆）+ 全量消息」调用模型，因此后续轮次中，模型能“看到”之前记住的名字、偏好和话题。

若要拆成多节点（例如先经过“记忆抽取”节点再进入“对话”节点），可把 `extract_memory` 与 `chat_node` 分成两个节点，中间用边连接，逻辑与上述一致，只是图更细化。

---

## 作为 Telegram Bot

上述带记忆的旅行助手可以直接接到 Telegram：**每个 Telegram 聊天对应一个 `thread_id`**，该聊天的历史与记忆（`user_name`、`preferences`、`last_topic`）都只在该会话内生效。

### 思路

| 概念 | 在 Telegram 中的对应 |
|------|----------------------|
| `thread_id` | Telegram 的 `chat_id`（或 `f"tg-{chat_id}"` 等唯一串） |
| 用户发来的一条消息 | 转为一条 `HumanMessage`，调用 `agent.invoke` |
| Agent 的回复 | 取返回状态里最后一条 AI 消息的 `content`，用 Bot API 发回该聊天 |

这样，每个用户（或群组）的对话互不干扰，且同一聊天内多轮对话会自然累积记忆。

### 依赖

```text
python-telegram-bot>=20
langgraph
langchain-openai
```

### Bot 端对接代码

沿用前文的 `create_memory_chat_agent()` 与 `TravelAssistantState`，只在「收到 Telegram 消息 → 调 Agent → 把回复发回 Telegram」这一层做对接：

```python
import os
from langchain_core.messages import HumanMessage
from telegram import Update
from telegram.ext import Application, ContextTypes, MessageHandler, filters

# 使用前文定义的 create_memory_chat_agent
from your_agent_module import create_memory_chat_agent

agent = create_memory_chat_agent()


async def on_message(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """收到用户文字消息时：用 chat_id 当 thread_id 调 Agent，再把回复发回该聊天。"""
    chat_id = update.effective_chat.id
    user_text = update.message.text.strip()

    if not user_text:
        await update.message.reply_text("请发一段文字，我会作为旅行助手回复你。")
        return

    config = {"configurable": {"thread_id": str(chat_id)}}

    try:
        result = agent.invoke(
            {"messages": [HumanMessage(content=user_text)]},
            config=config,
        )
        reply = result["messages"][-1].content
        await update.message.reply_text(reply)
    except Exception as e:
        await update.message.reply_text(f"处理时出错：{e}")


def main() -> None:
    token = os.environ.get("TELEGRAM_BOT_TOKEN")
    if not token:
        raise RuntimeError("请设置环境变量 TELEGRAM_BOT_TOKEN")

    app = Application.builder().token(token).build()
    app.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, on_message))
    app.run_polling()


if __name__ == "__main__":
    main()
```

要点：

- **`thread_id = str(chat_id)`**：私聊时每个用户一个会话；群聊时每个群一个会话，同一群内所有人共享该会话的记忆（若要对「用户+群」隔离，可用 `f"{chat_id}-{user_id}"`）。
- **只传本轮用户消息**：`invoke` 时只放本轮的 `HumanMessage`；Checkpoint 会按 `thread_id` 自动拼上该聊天已有的消息和状态。
- **同步 `invoke` 在异步里**：若希望不阻塞事件循环，可把 `agent.invoke(...)` 丢到 `asyncio.to_thread` 或线程池中执行，再在回调里 `reply_text`。

### 可选：按「用户 + 聊天」隔离记忆

若在**群组**里希望每个用户有独立记忆，可用「聊天 + 用户」作为会话标识：

```python
chat_id = update.effective_chat.id
user_id = update.effective_user.id
thread_id = f"tg-{chat_id}-{user_id}"   # 同一群内不同用户不同记忆

config = {"configurable": {"thread_id": thread_id}}
```

私聊时 `chat_id` 已经唯一，用 `str(chat_id)` 作 `thread_id` 即可。

### 小结（Telegram 版）

| 要点 | 说明 |
|------|------|
| 会话边界 | 用 Telegram `chat_id`（或 `chat_id + user_id`）作为 `thread_id`，一个会话一份记忆。 |
| 消息流 | 用户消息 → `HumanMessage` → `agent.invoke(..., config)` → 取最后一条 AI 消息 → `reply_text`。 |
| 部署 | 同上文 Agent；生产环境可将 `MemorySaver` 换成 PostgreSQL 等持久化，重启后同一 `thread_id` 的对话与记忆仍可恢复。 |

---

## 记忆会爆炸吗？

**会。** 若不做任何控制，同一 `thread_id` 下的 `messages` 会随轮次不断追加，结构化记忆（如 `preferences`、`last_topic`）也可能只增不减，容易带来：

| 风险 | 说明 |
|------|------|
| **上下文超长** | 每轮都把**整段**历史发给 LLM，token 数线性甚至更快增长，触及模型上下文上限后被截断或直接报错。 |
| **成本与延迟** | 输入越长，调用越贵、越慢。 |
| **存储膨胀** | Checkpoint 里为每个 thread 存完整状态，会话多、轮次多时占用会明显增加。 |

因此需要**主动控制“喂给模型”和“持久化”的量**，而不是无限堆积。

### 常见应对

1. **发给 LLM 的 messages 做截断或摘要**
   - **滑动窗口**：只保留最近 N 轮（例如最后 10 条 Human + AI），再拼上系统提示与记忆字段。
   - **摘要压缩**：超出窗口的旧对话用 LLM 生成一段「此前对话摘要」，以后每轮只发「摘要 + 最近 N 条」给模型；`messages` 在 Checkpoint 里仍可多留一些用于摘要生成，但**进入当次 LLM 调用的**要限长。

2. **结构化记忆设上限**
   - `preferences` 只保留最近或按优先级保留若干条，超出则覆盖或丢弃。
   - `last_topic` 只保留最近 1～2 句摘要，或用固定长度字符串。

3. **Checkpoint 深度与清理**
   - 若使用支持「每个 thread 只保留最近 K 个 checkpoint」的存储，可配置 K，避免单个会话的历史无限增长。
   - 对长期不活跃的 `thread_id` 做归档或删除，释放存储。

4. **按 token 预算截断**
   - 在拼好「系统提示 + 记忆 + 历史消息」之后、调用 LLM 之前，按总 token 数从**最旧**的消息开始丢，直到不超过设定上限（如 8k、32k），再发送。这样既保留最近几轮，又绝不会超窗。

### 方案：向量库 + 最近 N 条 + 检索附加

你说的做法——**把历史对话存进向量数据库，每次只加载最近 N 条，再按语义检索“相关”的旧对话附加到上下文**——是一个很常用、也很适合长会话的方案，可以单独用，也可以和前面的滑动窗口、结构化记忆一起用。

#### 思路

| 步骤 | 做法 |
|------|------|
| **写入** | 每轮对话结束后，把本条（或本轮 Human+AI）转成文本，打成 embedding，按 `thread_id` 写入向量库（可带时间戳、轮次等元数据）。 |
| **读取** | 调用 LLM 前：① 取该 thread 的**最近 K 条消息**；② 用「当前用户消息」或「最近几条消息拼接」做 query，在向量库里检索该 thread 的**历史片段**，取 top‑N 条；③ 拼成 `[系统提示 + 记忆字段] + [检索到的相关旧对话] + [最近 K 条]` 再发给模型。 |

这样**上下文长度有上界**（由「最近 K 条」+「检索到的 N 条」+ 系统提示决定），同时还能在用户问「上次说的那个地方叫什么」「你记得我说过喜欢靠窗吗」时，把真正相关的旧对话拉回来，而不是只能看到最近几轮。

#### 优点与适用场景

- **上下文有界**：不再随轮次线性变长，成本与延迟可控。
- **按需召回**：久远但语义相关的内容仍能进入当次上下文，比纯滑动窗口更能回答“以前说过什么”。
- **适合长会话、多轮闲聊**：例如旅行助手聊了很久，用户突然问「我们最开始聊的那家酒店叫什么」，检索可以帮忙找到那一段。

#### 实现要点

1. **存什么**  
   - 按「条」存：每条 Human/AI 各一条文档，或按「轮」存：一轮对话（Human + AI）合并成一段再存。按轮存一般更连贯，按条存则更细、检索更精准，可按产品需求选。
   - 每条/段文档建议带 `thread_id`、`ts`（或 `turn_id`），写入时只写该 thread 的，检索时按 `thread_id` 过滤，避免串会话。

2. **用什么做 query**  
   - 通常用**当前用户消息**即可；若要更强一点，可用「当前用户消息 + 上一条助手回复」拼成 query，再检索。

3. **检索数量与排序**  
   - 设一个上限（如 5～10 条/段），再按相似度排序；若向量库支持，可按 `ts` 做轻度衰减，让“最近的相关对话”略优先。
   - 检索到的片段按时间顺序拼进上下文，避免顺序错乱。

4. **拼接顺序**  
   - 建议：`[系统提示 + 结构化记忆] + [检索到的相关历史（按时间从前到后）] + [最近 K 条消息]`。  
   - 这样模型先有「身份与记忆」、再看到「和当前问题相关的旧事」、最后是「刚发生的对话」，逻辑清晰。

5. **与 Checkpoint 的关系**  
   - 可以「双写」：Checkpoint 照常存 `messages`（用于最近 K 条、或做时间旅行）；向量库只存「需要被检索」的历史（例如每轮结束后异步写入）。  
   - 也可以「向量库为主」：历史只进向量库，Checkpoint 里只保留最近几轮或只保留结构化记忆，由你根据运维与产品需求定。

#### 取舍

| 方面 | 说明 |
|------|------|
| **检索质量** | 依赖 embedding 与 chunk 策略；若按轮存，一轮很长时可以考虑再拆成多段，避免单段过长。 |
| **延迟** | 多一次向量检索；若用本地/近端向量库或缓存，通常可接受。 |
| **一致性** | 新写进的对话要尽快写入向量库，否则当轮或下一轮检索不到；若异步写，要有“最近 K 条一定从 Checkpoint 取”的约定。 |
| **成本** | 向量库存储与 embedding 调用（若用远程 embedding 服务）会带来额外成本，换回的是长对话下的可控上下文与更好召回。 |

#### 在本文案例中的示意

在旅行助手场景下，可以在「记忆抽取 + 对话」节点之前加一层「加载上下文」的逻辑：从 Checkpoint 取最近 K 条，从向量库按 `thread_id` + 当前用户消息检索 top‑N 段，再拼成 `all_messages` 传入 `chat_node`。伪代码示意：

```python
# 伪代码：向量库 + 最近 N 条 + 检索附加
def build_context(state, user_message: str, vector_store, thread_id: str):
    recent = list(state["messages"])[-10:]  # 最近 10 条
    docs = vector_store.similarity_search(
        user_message, k=5, filter={"thread_id": thread_id}
    )
    # 检索到的旧对话按时间排序后转成 message 列表
    past_messages = [doc_to_message(d) for d in sorted(docs, key=lambda d: d.metadata["ts"])]
    return past_messages + recent  # 或 [system_msg] + past_messages + recent

def chat_node(state):
    # ...
    context = build_context(state, state["messages"][-1].content, vector_store, thread_id)
    all_messages = [system_msg] + context
    response = llm.invoke(all_messages)
    return {"messages": [response]}
```

实际实现时，`vector_store` 和 `thread_id` 可以从 `config["configurable"]` 或依赖注入传入；每轮在写回 Checkpoint 之后，再把本轮的 Human+AI 文本写入向量库（例如在 Bot 的 `on_message` 里、或在图的一个「持久化」节点里做）。

整体上，**向量库 + 最近 N 条 + 检索附加**在长会话、需要“记得很久以前说过什么”的场景里，是一个兼顾长度可控和召回质量的成熟做法，可以当作「记忆会爆炸」的一种推荐方案写进文档。

### 在本文案例中的落地方式

在 `chat_node` 里「拼好 `all_messages` 之后、`llm.invoke` 之前」加一层限长，例如只取最近 20 条消息，或先算 token 再截断：

```python
def chat_node(state: TravelAssistantState) -> dict:
    # ... 前面拼 system_parts、system_msg 不变 ...

    # 只把「最近 N 条」喂给 LLM，避免上下文爆炸
    max_turns = 20  # 最近 20 条消息（10 轮对话）
    all_messages = [system_msg] + list(state["messages"])[-max_turns:]
    response = llm.invoke(all_messages)
    return {"messages": [response]}
```

Checkpoint 里仍然可以存完整 `messages`（由 `add_messages` 与 MemorySaver 负责），但**每次调用模型时**只用最近一段，即可在「记忆有用」和「不会爆炸」之间取得平衡。若希望更省存储，可以在写入 checkpoint 前对 `state["messages"]` 做滑动窗口或摘要后再存，依需求取舍。

---

## 小结

| 要点 | 说明 |
|------|------|
| 会话记忆 | 使用同一 `thread_id` + Checkpoint（如 `MemorySaver`），消息与状态在会话内累积。 |
| 结构化记忆 | 在状态中增加 `user_name`、`preferences`、`last_topic` 等字段，由 `extract_memory` 从最新用户输入里抽取并写回。 |
| 注入方式 | 在每轮调用 LLM 前，用这些字段拼成系统提示，让模型在回复时自然运用已记住的信息。 |
| 容量风险 | 消息与状态会随轮次增长，需对**喂给 LLM 的历史**做截断或摘要，并对 Checkpoint/结构化记忆设上限，见「[记忆会爆炸吗？](#记忆会爆炸吗)」。 |
| 扩展方向 | 偏好解析可改为用 LLM 做信息抽取；`last_topic` 可改为用 LLM 写简短摘要；生产环境可将 `MemorySaver` 换成 PostgreSQL 等持久化 Checkpoint；长会话可把历史存入**向量库**，只加载最近 N 条并**检索相关对话**附加到上下文，见「[方案：向量库 + 最近 N 条 + 检索附加](#方案向量库--最近-n-条--检索附加)」。 |

相关实现细节可对照 [06-memory-agent.md](06-memory-agent.md) 中的记忆层次与通用模式。
