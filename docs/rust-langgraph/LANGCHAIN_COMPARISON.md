# LangChain vs LangGraph 对比分析

## 核心差异

### LangChain - 组件化框架

```
┌─────────────────────────────────────────────────────────┐
│                    LangChain                           │
│                   "乐高积木" 风格                      │
├─────────────────────────────────────────────────────────┤
│  │                                                  │
│  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐         │
│  │ LLM │  │Chain│  │Tool │  │Prompt│  │Memory│       │
│  │     │  │     │  │     │  │      │  │      │       │
│  └──┬──┘  └──┬──┘  └──┬──┘  └──┬──┘  └──┬──┘         │
│     │       │       │       │       │               │
│     └───────┴───────┴───────┴───────┴───────────────►   │
│                  一系列串行操作                         │
└─────────────────────────────────────────────────────────┘
```

**特点**：
- 预构建组件，快速开发
- 适合简单聊天机器人、RAG 管道
- 一次性任务，常见 LLM 模式

### LangGraph - 状态图编排

```
┌─────────────────────────────────────────────────────────┐
│                    LangGraph                           │
│                 "交通管制" 风格                      │
├─────────────────────────────────────────────────────────┤
│                  ┌─────────────┐                       │
│              ┌──►│   START     │                       │
│              │   └──────┬──────┘                       │
│              │          │                             │
│         ┌────┴────┐     │     ┌────┴────┐                  │
│         │ Think   │◄───┘     │  Act    │                  │
│         └────┬────┘             └────┬────┘                  │
│              │                    │                         │
│              └────┬───────────────┘                         │
│                   │                                       │
│              should_continue?                             │
│                   │                                       │
│            ┌──────┴──────┐                                 │
│            ▼             ▼                                 │
│        ┌─────────┐   ┌─────────┐                         │
│        │  END   │   │ Think   │ (循环)                   │
│        └─────────┘   └─────────┘                         │
└─────────────────────────────────────────────────────────┘
```

**特点**：
- 有状态、基于图的编排
- 适合长期运行任务、多 Agent 系统
- 深度/长期多步骤任务
- 内置高级状态管理

---

## 详细对比

| 特性 | LangChain | LangGraph |
|------|-----------|-----------|
| **复杂度** | 低到中 | 中到高 |
| **学习曲线** | 容易上手 | 需要理解图结构 |
| **状态管理** | 有限 | 高级内置状态管理 |
| **控制力** | 高层抽象 | 底层细粒度控制 |
| **持久化** | 外部手动 | Checkpoint 架构 |
| **适用场景** | 原型、常见模式 | 复杂工作流、Agent |

---

## LangChain 核心概念

### Chain（链）

```python
from langchain.chains import LLMChain
from langchain_openai import ChatOpenAI

# 串行执行一系列操作
chain = LLMChain(
    llm=ChatOpenAI(),
    prompt="你是一个有用的助手，回答: {question}"
)

result = chain.invoke({"question": "什么是 Rust?"})
```

**Rust 对应实现**：

```rust
use langgraph::prelude::*;

/// Chain - 串行执行
pub struct Chain<C, T> {
    components: Vec<C>,
    _phantom: PhantomData<T>,
}

impl<C, T> Chain<C, T>
where
    C: Component<Input = T, Output = T>,
{
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn add(mut self, component: C) -> Self {
        self.components.push(component);
        self
    }

    pub async fn execute(&self, input: T) -> Result<T, ChainError> {
        let mut current = input;

        for component in &self.components {
            current = component.execute(current).await?;
        }

        Ok(current)
    }
}

trait Component {
    type Input;
    type Output;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ComponentError>;
}
```

### Agent (LangChain)

```python
from langchain.agents import AgentExecutor, create_openai_functions_agent
from langchain.tools import tool

@tool
def search(query: str) -> str:
    return "搜索结果..."

agent = create_openai_functions_agent(
    model="gpt-4o",
    tools=[search]
)

agent_executor = AgentExecutor(
    agent=agent,
    verbose=True
)

response = agent_executor.invoke({"input": "搜索 Rust"})
```

---

## LangGraph 核心概念

### StateGraph + Checkpoint

```python
from langgraph.graph import StateGraph, START, END
from langgraph.checkpoint.memory import MemorySaver

# 定义状态
class State(TypedDict):
    messages: Annotated[list[Message], add_messages]
    thought: str
    action: str
    observation: str

# 定义节点
def think_node(state: State):
    return {"thought": "思考中..."}

def act_node(state: State):
    return {"observation": "执行结果"}

# 构建图
graph = StateGraph(State)
graph.add_node("think", think_node)
graph.add_node("act", act_node)
graph.add_edge(START, "think")
graph.add_conditional_edges(
    "think",
    should_continue,
    {"continue": "think", "end": END}
)

# Checkpoint 持久化
checkpointer = MemorySaver()
compiled_graph = graph.compile(checkpointer=checkpointer)

# 执行
result = compiled_graph.invoke(
    {"messages": [HumanMessage(content="你好")]},
    config={"configurable": {"thread_id": "session-1"}}
)
```

### Checkpoint 架构

```
┌─────────────────────────────────────────────────────────┐
│              LangGraph Checkpoint 架构                   │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │              Checkpoint (检查点)                   │    │
│  │  ┌─────────────────────────────────────────────┐  │    │
│  │  │  checkpoint: {                              │  │    │
│  │  │    state: State,                            │  │    │
│  │  │    channel_versions: Map[str, int],        │  │    │
│  │  │    versions_seen: Set[str]                 │  │    │
│  │  │  }                                         │  │    │
│  │  │  config: {                                 │  │    │
│  │  │    thread_id: str,                         │  │    │
│  │  │    checkpoint_id: str,                      │  │    │
│  │  │    checkpoint_ns: str                       │  │    │
│  │  │  }                                         │  │    │
│  │  └─────────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────┘    │
│                       ▼                                  │
│  ┌─────────────────────────────────────────────────┐    │
│  │              Checkpointer (存储)                  │    │
│  │  ┌──────────┐  ┌───────────┐  ┌─────────────┐   │    │
│  │  │ Memory   │  │  SQLite   │  │  PostgreSQL │   │    │
│  │  │  Saver    │  │   Saver    │  │    Saver    │   │    │
│  │  └──────────┘  └───────────┘  └─────────────┘   │    │
│  └─────────────────────────────────────────────────┘    │
│                                                          │
│  支持:                                                   │
│  - 时间旅行 (回溯到任意 checkpoint)                       │
│  - 分支比较                                               │
│  - 状态回滚                                               │
│  - 人机协作 (interrupt/resume)                            │
└─────────────────────────────────────────────────────────┘
```

---

## Rust 实现对应

### LangChain Chain → Rust Chain

```rust
// LangChain: 串行组件
// Rust: 对应实现

pub struct Chain<C, T> {
    components: Vec<Box<dyn Component<Input = T, Output = T>>>,
}

#[async_trait]
impl<C, T> Component for Chain<C, T>
where
    T: Send + 'static,
{
    type Input = T;
    type Output = T;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ComponentError> {
        let mut result = input;

        for component in &self.components {
            result = component.execute(result).await?;
        }

        Ok(result)
    }
}
```

### LangGraph StateGraph → Rust 状态机

```rust
// LangGraph: 状态图 + Checkpoint
// Rust: 对应实现

use std::sync::Arc;

/// 检查点对应
#[derive(Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub checkpoint_id: String,
    pub thread_id: String,
    pub state: Vec<u8>,           // 序列化状态
    pub step: usize,
    pub timestamp: i64,
}

/// 检查点存储 trait
#[async_trait]
pub trait Checkpointer: Send + Sync {
    async fn save(&self, checkpoint: &Checkpoint) -> Result<(), CheckpointError>;
    async fn load(&self, thread_id: &str) -> Result<Option<Checkpoint>, CheckpointError>;
}

/// 内存 Checkpointer (开发用)
pub struct MemorySaver {
    checkpoints: Arc<RwLock<HashMap<String, Checkpoint>>>,
}

/// PostgreSQL Checkpointer (生产用)
pub struct PostgresSaver {
    pool: PgPool,
}
```

---

## 功能对比表

| 功能 | LangChain | LangGraph | Rust 实现难度 |
|------|-----------|-----------|-------------|
| 基础对话 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | 简单 |
| RAG | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | 中等 |
| 工具调用 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 简单 |
| 多步骤推理 | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 中等 |
| 状态持久化 | ⭐⭐ | ⭐⭐⭐⭐⭐ | 复杂 |
| 时间旅行 | ❌ | ⭐⭐⭐⭐⭐ | 复杂 |
| 多 Agent 协作 | ⭐⭐ | ⭐⭐⭐⭐⭐ | 复杂 |
| 人机协作 | ⭐⭐ | ⭐⭐⭐⭐ | 复杂 |

---

## Rust 版本的实现建议

### 1. 分层设计

```rust
// 高层 API (类似 LangChain)
mod high_level {
    pub struct Chain;
    pub struct SimpleAgent;
    pub struct RAGPipeline;
}

// 底层 API (类似 LangGraph)
mod low_level {
    pub struct StateGraph<S>;
    pub trait Checkpointer;
    pub trait Node<S>;
}
```

### 2. 类型安全的状态

```rust
// Python: 动态类型
class State(TypedDict):
    messages: list[Message]
    thought: str  # 可能缺失，运行时才知道

// Rust: 编译时保证
pub enum AgentState {
    Thinking { messages: Vec<Message>, step: usize },
    Acting { tool_calls: Vec<ToolCall> },
    Observing { results: Vec<ToolResult> },
    Done { answer: String },
}

// 没有遗漏，编译时检查完整
```

### 3. Checkpoint 实现

```rust
// 核心 trait
#[async_trait]
pub trait Checkpointer: Send + Sync {
    async fn save(&self, id: &CheckpointId, state: &[u8]) -> Result<(), CheckpointError>;
    async fn load(&self, id: &CheckpointId) -> Result<Option<Vec<u8>>, CheckpointError>;
    async fn list(&self, thread_id: &str) -> Result<Vec<CheckpointId>, CheckpointError>;
}

// 时间旅行：回溯到指定步骤
pub async fn rewind(
    &self,
    thread_id: &str,
    to_step: usize,
) -> Result<(), CheckpointError> {
    let checkpoints = self.checkpointer.list(thread_id).await?;
    let target = checkpoints.into_iter()
        .find(|cp| cp.step <= to_step)
        .ok_or(CheckpointError::NotFound)?;

    // 回滚状态
    // ...
    Ok(())
}
```

---

## 选择建议

### 使用 Rust 实现的优势

| 方面 | Python | Rust |
|------|--------|------|
| **类型安全** | 运行时错误 | 编译时捕获 |
| **性能** | GIL 限制 | 真并行 |
| **部署** | 需要 Python 环境 | 单一二进制 |
| **内存** | 高占用 | 零成本抽象 |

### Rust 实现策略

1. **两套 API**：
   - 简单场景用 `Chain`、`SimpleAgent`
   - 复杂场景用 `StateGraph`、`Checkpoint`

2. **渐进式**：
   - 先实现 LangChain 对等物
   - 再实现 LangGraph 核心功能

3. **互操作性**：
   - 支持 Python 协议（如果需要）
   - FFI 接口集成现有 Python 库

---

## 参考资料

- [LangChain vs LangGraph：2025年該選擇哪個框架？](https://sider.ai/zh-TW/blog/ai-tools/langgraph-vs-langchain-which-framework-should-you-use-in-2025)
- [全面测评LangChain vs LangGraph：谁是agent落地最优解](https://zilliz.com.cn/blog/LangChain-vs-LangGraph-Agent-Deployment-Showdown)
- [LangGraph 官方文档](https://langchain-ai.github.io/langgraph/)
- [LangChain 官方文档](https://python.langchain.com/)
