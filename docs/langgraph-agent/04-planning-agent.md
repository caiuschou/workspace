# 多步骤规划 Agent

处理需要多步骤完成的复杂任务，支持任务规划和分步执行。

## 状态设计

```python
from typing import Annotated, Sequence, TypedDict
from langchain_core.messages import BaseMessage
from langgraph.graph import StateGraph, START, END

class PlanningState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]
    plan: list[str]              # 任务计划
    current_step: int            # 当前执行到哪一步
    completed_steps: list[str]   # 已完成的步骤
    final_result: str            # 最终结果
```

## 实现代码

```python
def create_planning_agent():
    llm = ChatOpenAI(model="gpt-4o")

    def planner_node(state: PlanningState):
        """规划节点：制定任务计划"""
        task = state["messages"][-1].content

        prompt = f"""
        任务: {task}

        请将这个任务分解为具体步骤，用 JSON 数组格式返回:
        ["步骤1", "步骤2", "步骤3"]
        """

        response = llm.invoke(prompt)

        # 解析计划
        import json
        try:
            plan = json.loads(response.content)
        except:
            plan = ["分析任务", "执行任务", "总结结果"]

        return {
            "plan": plan,
            "current_step": 0,
            "completed_steps": []
        }

    def execute_node(state: PlanningState):
        """执行节点：执行当前步骤"""
        step_index = state["current_step"]
        current_step = state["plan"][step_index]

        # 执行当前步骤（这里可以调用具体服务）
        result = f"已完成: {current_step}"

        return {
            "current_step": step_index + 1,
            "completed_steps": state["completed_steps"] + [current_step]
        }

    def should_continue(state: PlanningState) -> Literal["continue", "finish"]:
        """判断是否继续执行"""
        if state["current_step"] < len(state["plan"]):
            return "continue"
        return "finish"

    def finalize_node(state: PlanningState):
        """汇总节点：生成最终结果"""
        summary = f"已完成所有步骤: {', '.join(state['completed_steps'])}"
        return {"final_result": summary}

    # 构建图
    builder = StateGraph(PlanningState)
    builder.add_node("planner", planner_node)
    builder.add_node("execute", execute_node)
    builder.add_node("finalize", finalize_node)

    builder.add_edge(START, "planner")
    builder.add_conditional_edges("planner", should_continue, {
        "continue": "execute",
        "finish": "finalize"
    })
    builder.add_conditional_edges("execute", should_continue, {
        "continue": "execute",
        "finish": "finalize"
    })
    builder.add_edge("finalize", END)

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
│ planner  │───────┐
└─────────┘       │
     │ continue    ▼           │
     ▼        ┌─────────┐      │
┌─────────┐  │ execute  │◄─────┘
│finalize │  └─────────┘
└────┬────┘       │ continue
     │           └──────┘ (循环)
     ▼
┌─────────┐
│   END   │
└─────────┘
```
