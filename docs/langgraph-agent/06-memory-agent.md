# 带记忆的 Agent

结合 Checkpoint 实现能记住上下文的 Agent。

## 记忆层次

| 类型 | 实现 | 生命周期 | 说明 |
|------|------|----------|------|
| 会话记忆 | Checkpoint | thread_id 范围 | 同一会话内的记忆 |
| 长期记忆 | Store | 跨会话永久 | 用户偏好、历史数据 |
| 语义记忆 | VectorStore | 可检索 | 基于向量相似度检索 |

## 实现代码

```python
from typing import Annotated, Sequence, TypedDict
from langchain_core.messages import BaseMessage, SystemMessage
from langgraph.checkpoint.memory import MemorySaver
from langgraph.graph import StateGraph, START, END
from langgraph.graph.message import add_messages

class MemoryState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]
    user_name: str        # 记住用户名字
    user_preferences: dict # 用户偏好
    context_summary: str  # 对话摘要

def create_agent_with_memory():
    llm = ChatOpenAI(model="gpt-4o")

    def agent_node(state: MemoryState):
        messages = state["messages"]
        user_name = state.get("user_name", "")
        preferences = state.get("user_preferences", {})

        # 动态构建系统提示，包含记忆信息
        system_content = "你是一个友好的助手。"
        if user_name:
            system_content += f"\n用户名字: {user_name}"
        if preferences:
            system_content += f"\n用户偏好: {preferences}"

        messages = [SystemMessage(content=system_content)] + list(messages)
        response = llm.invoke(messages)

        # 提取并保存新的用户信息（实际可用另一个 LLM 调用来提取）
        # ...

        return {"messages": [response]}

    def extract_info_node(state: MemoryState):
        """信息提取节点：从对话中提取用户信息"""
        last_message = state["messages"][-1].content

        # 简单示例：提取名字
        import re
        if "我叫" in last_message:
            match = re.search(r"我叫(\w+)", last_message)
            if match:
                return {"user_name": match.group(1)}

        return {}

    builder = StateGraph(MemoryState)
    builder.add_node("extract_info", extract_info_node)
    builder.add_node("agent", agent_node)
    builder.add_edge(START, "extract_info")
    builder.add_edge("extract_info", "agent")
    builder.add_edge("agent", END)

    return builder.compile(checkpointer=MemorySaver())
```

## 使用示例

```python
agent = create_agent_with_memory()
config = {"configurable": {"thread_id": "user-123"}}

# 第一轮对话
agent.invoke({"messages": ["我叫小明"]}, config)

# 第二轮对话 - 记得用户名字
agent.invoke({"messages": ["我叫什么名字？"]}, config)
# 返回: "你叫小明"
```
