# 对话 Agent

最基础的 Agent，处理用户输入并返回回复。

## 完整代码

```python
from typing import Annotated, Sequence, TypedDict
from langchain_core.messages import BaseMessage, HumanMessage, SystemMessage
from langgraph.checkpoint.memory import MemorySaver
from langgraph.graph import StateGraph, START, END
from langgraph.graph.message import add_messages
from langchain_openai import ChatOpenAI

# ========== 定义状态 ==========
class ChatState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]

# ========== 创建 Agent ==========
def create_chat_agent():
    llm = ChatOpenAI(model="gpt-4o")

    def chatbot_node(state: ChatState):
        """聊天节点：处理消息并生成回复"""
        messages = state["messages"]

        # 添加系统提示
        system_msg = SystemMessage(content="你是一个友好的助手。")
        all_messages = [system_msg] + list(messages)

        response = llm.invoke(all_messages)
        return {"messages": [response]}

    # 构建图
    builder = StateGraph(ChatState)
    builder.add_node("chatbot", chatbot_node)
    builder.add_edge(START, "chatbot")
    builder.add_edge("chatbot", END)

    # 添加记忆
    return builder.compile(checkpointer=MemorySaver())

# ========== 使用 ==========
agent = create_chat_agent()
config = {"configurable": {"thread_id": "chat-session-1"}}

# 第一轮对话
result = agent.invoke(
    {"messages": [HumanMessage("你好")]},
    config=config
)
print(result["messages"][-1].content)

# 第二轮对话 - 有记忆
result = agent.invoke(
    {"messages": [HumanMessage("我刚才说了什么？")]},
    config=config
)
```

## 图结构

```
┌─────────┐
│  START  │
└────┬────┘
     │
     ▼
┌─────────┐
│ chatbot │
└────┬────┘
     │
     ▼
┌─────────┐
│   END   │
└─────────┘
```
