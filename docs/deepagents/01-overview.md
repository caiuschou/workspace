# Deep Agents 概述

## 是什么

Deep Agents 是 LangChain 官方的 **开箱即用 Agent Harness**。通过中间件组合，提供规划、文件操作、子 Agent、上下文管理等能力，无需自行拼装 Prompt、工具和上下文逻辑。

> 基于 LangChain + LangGraph，返回 compiled LangGraph graph，支持 streaming、checkpoint、Studio 等。

## 项目结构

```
deepagents/
├── libs/
│   ├── deepagents/     # 核心 SDK
│   │   ├── graph.py           # create_deep_agent 入口
│   │   ├── backends/          # 后端协议与实现
│   │   └── middleware/        # 中间件
│   ├── cli/            # CLI 工具（会话恢复、Web 搜索、远程沙箱等）
│   ├── acp/            # Agent Context Protocol 支持
│   └── harbor/         # 评测 / 基准框架
├── examples/
│   ├── content-builder-agent/  # 内容生成（skills + subagents）
│   ├── deep_research/          # 深度研究 Agent
│   ├── text-to-sql-agent/      # Text-to-SQL
│   └── ralph_mode/             # Ralph 模式示例
└── README.md
```

## 快速开始

```python
from deepagents import create_deep_agent

agent = create_deep_agent()
result = agent.invoke({"messages": [{"role": "user", "content": "Research LangGraph and write a summary"}]})
```

## 默认能力

- **Planning**：`write_todos` / `read_todos` 任务拆解与进度跟踪
- **Filesystem**：`read_file`, `write_file`, `edit_file`, `ls`, `glob`, `grep` 读写上下文
- **Shell**：`execute` 执行命令（需 SandboxBackendProtocol）
- **Sub-agents**：`task` 委托任务，隔离上下文
- **Context**：长对话自动摘要，大结果写入文件

## 定制化示例

```python
from langchain.chat_models import init_chat_model

agent = create_deep_agent(
    model=init_chat_model("openai:gpt-4o"),
    tools=[my_custom_tool],
    system_prompt="You are a research assistant.",
    memory=["/memory/AGENTS.md"],
    skills=["/skills/user/", "/skills/project/"],
    subagents=[...],
    backend=FilesystemBackend(root_dir="./workspace"),
)
```

## 依赖管理

- 使用 **uv** 管理依赖
- 每个 lib 有独立 `pyproject.toml` 和 `uv.lock`
- 本地开发用 editable install
