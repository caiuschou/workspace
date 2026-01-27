# 多 Agent 协作

多个 Agent 各司其职，协作完成任务。

## 架构设计

```
                    ┌──────────────┐
                    │  Supervisor  │
                    │   (主管)     │
                    │  (调度/路由)  │
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Researcher   │  │   Analyst    │  │    Writer    │
│  (研究)      │  │   (分析)     │  │   (写作)     │
└──────────────┘  └──────────────┘  └──────────────┘
```

## 状态设计

```python
class MultiAgentState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]
    current_agent: str       # 当前激活的 agent
    research_data: str       # 研究数据
    analysis_result: str     # 分析结果
    final_report: str        # 最终报告
```

## 实现代码

```python
def researcher_node(state: MultiAgentState):
    """研究 Agent：负责收集信息"""
    last_message = state["messages"][-1].content

    # 收集信息（可以调用搜索服务）
    research_data = f"关于 '{last_message}' 的研究数据..."

    return {
        "current_agent": "researcher",
        "research_data": research_data
    }

def analyst_node(state: MultiAgentState):
    """分析 Agent：负责分析数据"""
    research = state.get("research_data", "")

    # 分析数据
    analysis = f"基于研究数据的分析: {research[:50]}..."

    return {
        "current_agent": "analyst",
        "analysis_result": analysis
    }

def writer_node(state: MultiAgentState):
    """写作 Agent：负责生成报告"""
    analysis = state.get("analysis_result", "")

    # 生成报告
    report = f"完整报告: {analysis}"

    return {
        "current_agent": "writer",
        "final_report": report
    }

def supervisor(state: MultiAgentState) -> Literal["researcher", "analyst", "writer", "end"]:
    """主管：决定下一个执行的 Agent"""
    research = state.get("research_data")
    analysis = state.get("analysis_result")
    report = state.get("final_report")

    if not research:
        return "researcher"
    elif not analysis:
        return "analyst"
    elif not report:
        return "writer"
    return "end"

def create_multi_agent():
    builder = StateGraph(MultiAgentState)

    builder.add_node("researcher", researcher_node)
    builder.add_node("analyst", analyst_node)
    builder.add_node("writer", writer_node)

    builder.add_conditional_edges(START, supervisor, {
        "researcher": "researcher",
        "analyst": "analyst",
        "writer": "writer",
        "end": END
    })

    builder.add_conditional_edges("researcher", supervisor)
    builder.add_conditional_edges("analyst", supervisor)
    builder.add_conditional_edges("writer", supervisor)

    return builder.compile(checkpointer=MemorySaver())
```

## 图结构

```
┌─────────┐
│  START  │
└────┬────┘
     │
     ▼
┌─────────┐
│supervisor│───┐
└─────────┘    │
     │          │
     ├──researcher──┐
     │             │
     ├──analyst────┤
     │             │
     ├──writer─────┤
     │             │
     └──end ───────┴──► END
```
