# ReAct Agent（推理 + 行动）

ReAct = **Reasoning + Acting**，Agent 会先"思考"再"行动"。

## 核心思路

```
用户: 帮我查询今天天气

思考: 用户想知道今天的天气
行动: 需要调用天气服务
观察: 获取到天气数据
思考: 我有足够信息回答了
最终答案: 今天晴天，25°C
```

## 状态设计

```python
from typing import Literal
from langchain_core.messages import BaseMessage
from langgraph.graph import StateGraph, START, END

class ReActState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]
    thought: str      # 当前思考
    action: str       # 计划执行的动作
    observation: str  # 执行结果的观察
```

## 实现代码

```python
def create_react_agent():
    llm = ChatOpenAI(model="gpt-4o")

    def think_node(state: ReActState):
        """思考节点：分析当前状态，决定下一步"""
        last_message = state["messages"][-1].content

        # 让 LLM 进行推理
        prompt = f"""
        用户问题: {last_message}
        当前思考: {state.get("thought", "")}

        请分析并决定：
        1. 如果已有答案，返回最终答案
        2. 如果需要更多信息，说明需要做什么
        """

        response = llm.invoke(prompt)
        return {"thought": response.content}

    def should_continue(state: ReActState) -> Literal["think", "end"]:
        """判断是否需要继续思考"""
        thought = state.get("thought", "")
        if "最终答案" in thought or "完成" in thought:
            return "end"
        return "think"

    # 构建图
    builder = StateGraph(ReActState)
    builder.add_node("think", think_node)
    builder.add_edge(START, "think")
    builder.add_conditional_edges("think", should_continue, {
        "think": "think",  # 继续思考
        "end": END
    })

    return builder.compile(checkpointer=MemorySaver())
```

## 图结构

```
┌─────────┐
│  START  │
└────┬────┘
     │
     ▼
┌─────────┐  should_continue
│  think  │───────────────┐
└────┬────┘               │
     │ think              ▼
     └────────────►     ┌─────────┐
        (循环)           │   END   │
                         └─────────┘
```
