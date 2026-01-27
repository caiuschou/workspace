# 最佳实践

## 1. 状态设计

```python
# ✅ 好的状态设计 - 明确类型，使用注解
class AgentState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]
    step: int
    result: str

# ❌ 不好的状态设计 - 类型不明确
class AgentState(TypedDict):
    data: Any
    info: dict
```

## 2. 节点职责单一

```python
# ✅ 好的节点设计 - 单一职责
def planning_node(state):
    """只负责规划"""
    return {"plan": [...]}

def execution_node(state):
    """只负责执行"""
    return {"result": ...}

# ❌ 不好的节点设计 - 职责混乱
def do_everything_node(state):
    """又规划又执行又总结"""
    plan = make_plan(state)
    result = execute(plan)
    summary = summarize(result)
    return {...}
```

## 3. 错误处理

```python
def agent_node(state: AgentState):
    try:
        response = llm.invoke(state["messages"])
        return {"messages": [response]}
    except Exception as e:
        # 记录错误
        logger.error(f"Agent error: {e}")
        # 返回友好错误消息
        error_msg = SystemMessage(content=f"抱歉，处理出错了: {e}")
        return {"messages": [error_msg]}
```

## 4. 环境配置

```python
# 开发环境 - 内存存储
checkpointer = MemorySaver()

# 生产环境 - PostgreSQL 持久化
checkpointer = PostgresSaver.from_conn_string(
    "postgresql://user:pass@host:port/db"
)
```

## 5. 监控日志

```python
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("agent")

def agent_node(state: AgentState):
    user_id = state.get("user_id", "unknown")
    message = state["messages"][-1].content[:50]

    logger.info(f"[{user_id}] Processing: {message}...")
    # ...
    logger.info(f"[{user_id}] Response generated")
```

## 6. 性能优化

| 优化点 | 说明 |
|--------|------|
| 流式输出 | 使用 `stream()` 而不是 `invoke()` |
| 异步处理 | 使用异步节点和异步 LLM 调用 |
| 缓存 | 缓存 LLM 响应，避免重复计算 |
| 批处理 | 批量处理多个请求 |
