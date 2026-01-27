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

## 小结

| 要点 | 说明 |
|------|------|
| 会话记忆 | 使用同一 `thread_id` + Checkpoint（如 `MemorySaver`），消息与状态在会话内累积。 |
| 结构化记忆 | 在状态中增加 `user_name`、`preferences`、`last_topic` 等字段，由 `extract_memory` 从最新用户输入里抽取并写回。 |
| 注入方式 | 在每轮调用 LLM 前，用这些字段拼成系统提示，让模型在回复时自然运用已记住的信息。 |
| 扩展方向 | 偏好解析可改为用 LLM 做信息抽取；`last_topic` 可改为用 LLM 写简短摘要；生产环境可将 `MemorySaver` 换成 PostgreSQL 等持久化 Checkpoint。 |

相关实现细节可对照 [06-memory-agent.md](06-memory-agent.md) 中的记忆层次与通用模式。
