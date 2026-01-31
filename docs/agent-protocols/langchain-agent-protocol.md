# LangChain Agent Protocol — 详细研究

## 1. 背景与发起

- **发起方**：**LangChain**。
- **发布时间**：2024 年 11 月宣布。
- **仓库**：<https://github.com/langchain-ai/agent-protocol>（MIT 许可）。
- **文档**：<https://langchain-ai.github.io/agent-protocol/api.html>。

## 2. 目标与定位

- **定位**：面向「**运行/编排**」的标准化 API，让不同框架的 agent 能互操作；**不是**纯「agent 间消息协议」，而是运行与部署层面的统一接口。
- **问题**：各框架（AutoGen、OpenAI Assistant API、CrewAI、LlamaIndex 等）API 各异，难以统一编排与部署。
- **目标**：提供框架无关的通用接口，任何 agent 开发者（LangGraph 或其他框架、甚至无框架）均可实现该协议。

## 3. 技术架构

### 3.1 核心抽象

| 概念 | 说明 |
|------|------|
| **Runs** | 一次 agent 执行的抽象；对应「跑一轮 agent」。 |
| **Threads** | 多轮对话/多轮执行的会话组织；对应「一个会话」。 |
| **Store** | 长期记忆/状态存储；对应「跨会话的记忆与状态」。 |

### 3.2 API 设计

- 围绕 **Runs**、**Threads**、**Store** 提供标准化端点。
- 典型用法：创建 Thread → 在 Thread 上创建/执行 Run → 通过 Store 读写长期状态。
- 有提议如 `POST /threads/runs` 等单次操作（创建 Thread 并启动 Run），以减少往返。

### 3.3 与框架的关系

- **框架无关**：不绑定 LangGraph；可与 AutoGen、CrewAI、LlamaIndex 等配合。
- **用途**：LangGraph Studio 连接本地 agent、将 AutoGen/CrewAI 等当子 agent 使用、用 LangGraph Platform 部署其他框架的 agent，均通过该协议统一接口。

## 4. 生态与采用

- **仓库**：约 517 stars、41 forks（截至 2025）；采用处于早期，与 A2A、MCP 等并存于碎片化生态。
- **对比**：常与 A2A、MCP、AGNTCY 等一起出现在「开放 agent 标准」对比中；LangChain 将其定位为多 agent 互操作的基础设施。
- **State of AI Agents**：LangChain 的「State of AI Agents」报告显示，51% 受访者在生产中使用 agent、78% 有计划采用；Agent Protocol 作为互操作层在其中发挥作用。

## 5. 与其它协议关系

- **A2A**：A2A 偏「agent 间消息与任务」；LangChain Agent Protocol 偏「运行与会话编排」。可组合：用 A2A 做跨系统协作，用 Agent Protocol 做单系统内多框架编排。
- **MCP**：MCP 解决工具/上下文接入；Agent Protocol 解决运行/线程/存储接口，无直接替代关系。
- **ANP**：ANP 偏去中心化网络；Agent Protocol 偏中心化部署与编排，场景不同。

## 6. 参考

- 仓库：<https://github.com/langchain-ai/agent-protocol>
- 博客：<https://blog.langchain.com/agent-protocol-interoperability-for-llm-agents>
- API 文档：<https://langchain-ai.github.io/agent-protocol/api.html>
- State of AI Agents：<https://www.langchain.com/stateofaiagents>
