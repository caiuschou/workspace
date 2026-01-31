# Agent Communication Protocol (ACP) — 详细研究

## 1. 背景与发起

- **发起/推动**：**IBM** 等推动，面向 agent 间通信的开放标准。
- **现状**：原 GitHub 已归档（如 2025-08-27）；**已并入 Linux Foundation 下 A2A 治理**，IBM 逐步收尾 ACP 独立开发。
- **迁移**：官方迁移到 A2A 体系，指南见 <https://agentcommunicationprotocol.dev/introduction/welcome>。

## 2. 目标与定位

- **定位**：REST 风格、多模态消息的「agent 间通信」协议，常被形容为「**AI agent 的 HTTP**」。
- **用途**：多 agent 协作、动态 agent 更新、跨公司工作流、跨组织 agent 合作；偏本地优先、边缘与机器人等场景。

## 3. 技术要点（历史与迁移后）

### 3.1 架构特点

- **REST + HTTP**：明确端点，便于用 curl、Postman 等调试；与现有 Web 基础设施兼容。
- **消息**：基于 **JSON-RPC 2.0** 与 **Server-Sent Events (SSE)** 的结构化消息；支持多模态（文本、结构化数据、图像、embedding 等），常以 MIME 多部分形式组织。
- **路由**：发送/接收/路由消息的端点与语义清晰。
- **发现**：通过 **Agent Card**（如 `.well-known/agent-card.json`）做能力发现，与 A2A 的 Agent Card 概念对齐。
- **会话与异步**：同步/异步、会话管理、DID 等均有考虑。

### 3.2 与 A2A 的对应

迁移到 A2A 后，原有能力由 A2A 数据模型与操作覆盖：

- 消息 ↔ A2A Message/Part
- 任务/会话 ↔ A2A Task/contextId
- Agent Card ↔ A2A AgentCard
- REST 绑定 ↔ A2A HTTP+JSON/REST 绑定

迁移指南通常建议：依赖从 `acp_sdk` 换为 A2A 实现（如 `beeai_sdk` 中的 A2A 客户端/服务端）；元数据结构从 ACP 的 Metadata 等映射到 A2A 的对应字段；消息与任务处理逻辑按 A2A 规范改写。

## 4. 生态与采用

- **权威**：产业背书强（IBM、LF），但**单独流行度减弱**——新项目宜直接采用 A2A，既有 ACP 实现逐步迁移。
- **综述**：arXiv 2505.02279 将 ACP 列为「多模态消息与 agent 发现」阶段，并注明已并入 A2A 生态。

## 5. 与其它协议关系

- **A2A**：ACP 已并入 A2A；功能与概念由 A2A 继承与扩展。
- **MCP**：MCP 管工具/上下文，ACP（现 A2A）管 agent 间消息与任务，互补。
- **ANP**：ACP/A2A 偏中心化/企业协作；ANP 偏去中心化网络与市场。

## 6. 参考

- IBM 介绍：<https://www.ibm.com/think/topics/agent-communication-protocol>
- IBM 研究项目页：<https://research.ibm.com/projects/agent-communication-protocol>
- ACP → A2A 迁移/欢迎：<https://agentcommunicationprotocol.dev/introduction/welcome>
- IBM 教程（使用 A2A）：<https://www.ibm.com/think/tutorials/use-a2a-protocol-for-ai-agent-communication>
