# 核心概念

## Agent 是什么

```
Agent = LLM + 状态 + 决策循环
```

执行流程：
```
用户输入 → 决策节点 → 执行节点 → 结果 → 决策节点 → 输出
                  ↑___________________________|
                        (循环直到完成)
```

## Checkpoint（状态快照）

- **thread_id**: 会话标识，同一个 thread_id 会累积状态
- **checkpoint_id**: 状态快照 ID，支持"时间旅行"回溯
- **状态**: 包含消息历史、自定义变量等

## 状态管理

```python
from typing import Annotated, Sequence, TypedDict
from langchain_core.messages import BaseMessage
from langgraph.graph.message import add_messages

class AgentState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]
    # add_messages 自动追加新消息到历史
```

## 图结构要素

| 要素 | 说明 |
|------|------|
| **State** | 应用状态，定义数据流 |
| **Node** | 处理节点，接收/返回状态 |
| **Edge** | 连接边，定义节点间的流转 |
| **START** | 入口点，图的开始 |
| **END** | 结束点，图的终止 |
