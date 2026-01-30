# Agent 通信协议概览

本文档整理当前业界与学术界的 AI agent 之间沟通 / 互操作协议，便于选型与对比。

## 文档目录

| 文档 | 内容 |
|------|------|
| [README.md](README.md) | 概述、协议对比、任务记录 |

## 任务记录

| 任务 | 状态 |
|------|------|
| 整理 agent 通信协议（A2A、LACP、ACP、ANP 等） | 已完成 |
| 写入 docs/agent-protocols | 已完成 |

---

## 1. 概述

当前有多套面向「agent 之间沟通」或「agent 与工具/上下文」的协议与标准，大致分为：

- **工业标准**：A2A、ACP（已并入 A2A 生态）、MCP、LangChain Agent Protocol、ANP
- **学术协议**：LACP（南洋理工大学，NeurIPS 2025）

综述论文（arXiv 2505.02279）建议**分阶段采用**：MCP（工具接入）→ ACP（多模态消息与发现）→ A2A（协作任务）→ ANP（去中心化市场）。

---

## 2. 协议一览

| 协议 | 发起/主导 | 侧重 | 备注 |
|------|-----------|------|------|
| **A2A** | Google + 50+ 厂商，Linux Foundation | Agent↔Agent 协作、任务、企业 | 开放标准，规范 RC v1.0 |
| **ACP** | IBM 等，现归 A2A/LF | REST、多模态消息 | 已并入 A2A 治理 |
| **ANP** | 社区/开源 | 去中心化、DID、agent 网络与市场 | Apache-2.0 |
| **LangChain Agent Protocol** | LangChain | Runs/Threads/Store，框架互操作 | 面向运行与部署 |
| **MCP** | Anthropic | Agent↔工具/上下文 | 2024 提出，与 A2A 互补 |
| **LACP** | 南洋理工大学（学术） | 语义+事务+传输标准化 | NeurIPS 2025 |

---

## 3. Agent2Agent (A2A) 协议

- **发起**：Google 牵头，2025 年 4 月发布，现由 **Linux Foundation** 维护。
- **规范**：<https://a2a-protocol.org/latest/specification/>

### 目标

让不同厂商、不同框架的 AI agent 能互相发现、协作、安全交换信息，**不暴露内部状态、记忆或工具实现**。强调互操作、协作、能力发现、多种交互模式和企业级安全。

### 技术结构（三层）

| 层级 | 内容 |
|------|------|
| **Layer 1 数据模型** | Task、Message、AgentCard、Part、Artifact、Extension（规范用 Protocol Buffers 定义） |
| **Layer 2 抽象操作** | Send Message、Stream Message、Get/List/Cancel Task、Subscribe to Task、Push Notification 配置、Get Agent Card 等 |
| **Layer 3 协议绑定** | JSON-RPC、gRPC、HTTP/REST；可扩展自定义绑定 |

### 交互与发现

- **同步**：请求/响应（如 Send Message）
- **流式**：Server-Sent Events 实时推送任务状态与产出
- **异步**：Webhook 推送，适合长任务、人机协同
- **发现**：通过 **Agent Card**（如 `.well-known/agent.json`）声明身份、能力、技能、端点、鉴权方式

### 生态

50+ 科技公司参与（Atlassian、Salesforce、SAP、ServiceNow、Workday、埃森哲、德勤、麦肯锡等）。

---

## 4. LACP（LLM Agent Communication Protocol）

- **发起**：南洋理工大学（Xin Li, Mengbing Liu, Chau Yuen），**NeurIPS 2025**（AI4NextG Workshop）。
- **页面**：<https://lixin.ai/LACP/>

### 问题意识

当前 LLM agent 通信生态**碎片化**（类似早期网络“协议战争”），缺乏统一标准，导致互操作、安全与事务一致性不足。

### 设计（电信启发、三层）

| 层级 | 作用 |
|------|------|
| **语义层** | 用统一消息类型表达意图：**PLAN**、**ACT**、**OBSERVE** |
| **事务层** | 可靠性：消息签名（JWS）、序号、事务 ID、**两阶段提交** |
| **传输层** | 安全、高效传输 |

### 性能（论文数据）

- 延迟开销约 **3.5%**
- 消息大小开销约 **30%**

面向 NextG/6G 等场景下安全、可扩展的多 agent 系统。

---

## 5. Agent Communication Protocol (ACP)

- **发起/推动**：IBM 等推动，现由 **Linux Foundation** 下 A2A 治理。
- **定位**：REST 风格、多模态消息的「agent 间通信」协议，常被形容为「AI agent 的 HTTP」。

### 特点

- REST + 明确 HTTP 端点，支持发送/接收/路由消息
- MIME 多部分消息（文本、结构化数据、图像、embedding 等）
- 同步/异步、会话管理、消息路由，支持 DIDs

### 现状

原 GitHub 已归档，官方迁移到 **A2A** 体系，有迁移指南（如 <https://agentcommunicationprotocol.dev/introduction/welcome>）。

---

## 6. Agent Network Protocol (ANP)

- **定位**：面向「agent 网络/去中心化」的开放协议，自称「agentic web 时代的 HTTP」。
- **资源**：<https://agentnetworkprotocol.com/en/docs>，GitHub: `agent-network-protocol/AgentNetworkProtocol`（Apache-2.0）

### 三层架构

| 层级 | 内容 |
|------|------|
| **身份层** | 基于 W3C DID（含 DID:WBA）的去中心化认证与端到端加密 |
| **元协议层** | 动态协议协商、自组织 agent 网络 |
| **应用层** | 基于语义的能力描述与协议管理 |

支持开放网络中的 agent 发现与安全协作，适合**去中心化 agent 市场**。

---

## 7. LangChain Agent Protocol

- **发起**：**LangChain**
- **仓库**：`langchain-ai/agent-protocol`

### 定位

面向「运行/编排」的标准化 API，让不同框架的 agent 能互操作（不是纯「agent 间消息协议」）。

### 核心抽象

- **Runs**：执行一次 agent 运行
- **Threads**：多轮对话/多轮执行的会话
- **Store**：长期记忆/状态存储

### 用途

LangGraph Studio 连本地 agent、把 AutoGen/CrewAI 等当子 agent 用、用 LangGraph Platform 部署其他框架。

---

## 8. Model Context Protocol (MCP)

- **发起**：**Anthropic**（2024）
- **定位**：**Agent ↔ 工具/上下文/数据源**，不是 agent↔agent 的主战场，常和 A2A/ACP 一起出现在「agent 互操作」讨论里。

---

## 9. 分阶段采用建议（综述）

arXiv **2505.02279**（2025 年 5 月）的综述 *A Survey of Agent Interoperability Protocols: MCP, ACP, A2A, and ANP* 建议按用途分阶段采用：

| 阶段 | 协议 | 主要用途 |
|------|------|----------|
| 1 | **MCP** | 工具与上下文接入 |
| 2 | **ACP** | 结构化、多模态消息与 agent 发现（现并入 A2A 生态） |
| 3 | **A2A** | 协作式任务执行、企业工作流 |
| 4 | **ANP** | 去中心化、开放网络与 agent 市场 |

---

## 10. 参考

- A2A 规范：<https://a2a-protocol.org/latest/specification/>
- A2A 介绍（Google）：<https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability>
- LACP：<https://lixin.ai/LACP/>
- ACP 介绍（IBM）：<https://www.ibm.com/think/topics/agent-communication-protocol>
- ANP：<https://agentnetworkprotocol.com/en/docs>
- LangChain Agent Protocol：<https://github.com/langchain-ai/agent-protocol>
- 综述：arXiv 2505.02279 — *A Survey of Agent Interoperability Protocols: MCP, ACP, A2A, and ANP*
