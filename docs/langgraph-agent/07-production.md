# 生产级实现

## Agent 类封装

```python
from typing import Annotated, Sequence, TypedDict, Literal
from langchain_core.messages import BaseMessage, HumanMessage
from langchain_openai import ChatOpenAI
from langgraph.checkpoint.memory import MemorySaver
from langgraph.checkpoint.postgres import PostgresSaver
from langgraph.graph import StateGraph, START, END
from langgraph.graph.message import add_messages

class AgentState(TypedDict):
    messages: Annotated[Sequence[BaseMessage], add_messages]
    user_id: str
    session_id: str

class ProductionAgent:
    """生产级 Agent"""

    def __init__(self, model: str = "gpt-4o", use_postgres: bool = False):
        self.llm = ChatOpenAI(model=model, temperature=0)
        self.app = self._build_graph(use_postgres)

    def _build_graph(self, use_postgres: bool):
        def agent_node(state: AgentState):
            response = self.llm.invoke(state["messages"])
            return {"messages": [response]}

        builder = StateGraph(AgentState)
        builder.add_node("agent", agent_node)
        builder.add_edge(START, "agent")
        builder.add_edge("agent", END)

        # 选择持久化方案
        if use_postgres:
            checkpointer = PostgresSaver.from_conn_string(
                "postgresql://user:pass@host:port/db"
            )
        else:
            checkpointer = MemorySaver()

        return builder.compile(checkpointer=checkpointer)

    def chat(self, user_id: str, message: str, session_id: str = None) -> str:
        """单轮对话"""
        if session_id is None:
            session_id = f"session-{user_id}"

        config = {"configurable": {"thread_id": session_id}}

        result = self.app.invoke(
            {
                "messages": [HumanMessage(content=message)],
                "user_id": user_id,
                "session_id": session_id
            },
            config=config
        )

        return result["messages"][-1].content

    def stream_chat(self, user_id: str, message: str, session_id: str = None):
        """流式对话 - 可以看到每一步"""
        if session_id is None:
            session_id = f"session-{user_id}"

        config = {"configurable": {"thread_id": session_id}}

        for event in self.app.stream(
            {"messages": [HumanMessage(content=message)]},
            config=config
        ):
            for node_name, node_output in event.items():
                print(f"→ 节点: {node_name}")

        final_state = self.app.get_state(config)
        return final_state.values["messages"][-1].content

    def get_history(self, session_id: str):
        """获取对话历史"""
        config = {"configurable": {"thread_id": session_id}}
        state = self.app.get_state(config)
        if state:
            return state.values.get("messages", [])
        return []

    def reset_session(self, session_id: str):
        """重置会话"""
        # 删除对应的 checkpoint
        checkpointer = self.app.checkpointer
        if hasattr(checkpointer, "delete_thread"):
            checkpointer.delete_thread(session_id)
```

## HTTP API

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

app = FastAPI(title="Agent API")
agent = ProductionAgent()

class ChatRequest(BaseModel):
    user_id: str
    message: str
    session_id: str = None

class ChatResponse(BaseModel):
    response: str

@app.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    """聊天接口"""
    try:
        response = agent.chat(
            user_id=request.user_id,
            message=request.message,
            session_id=request.session_id
        )
        return ChatResponse(response=response)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/reset")
async def reset_session(session_id: str):
    """重置会话"""
    agent.reset_session(session_id)
    return {"message": "Session reset"}

@app.get("/history/{session_id}")
async def get_history(session_id: str):
    """获取对话历史"""
    messages = agent.get_history(session_id)
    return {"messages": [m.content for m in messages]}

# 运行: uvicorn api:app --reload
```

## 部署架构

```
┌─────────────────────────────────────────────────────┐
│                    Client Layer                     │
│              (Web / Mobile / CLI)                   │
└────────────────────────┬────────────────────────────┘
                         │ HTTP/WebSocket
                         ▼
┌─────────────────────────────────────────────────────┐
│                   API Gateway                       │
│                   (FastAPI)                         │
└────────────────────────┬────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│                  Agent Layer                        │
│                (ProductionAgent)                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │  Chat    │  │  ReAct   │  │  Multi   │         │
│  │  Agent   │  │  Agent   │  │  Agent   │         │
│  └──────────┘  └──────────┘  └──────────┘         │
└────────────────────────┬────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│     LLM      │  │   Storage    │  │   Services   │
│  (OpenAI)    │  │ (Checkpoint) │  │  (外部服务)   │
└──────────────┘  └──────────────┘  └──────────────┘
```
