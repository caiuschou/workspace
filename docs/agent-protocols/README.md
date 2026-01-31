# Agent 通信协议概览

本文档整理当前业界与学术界的 AI agent 之间沟通 / 互操作协议，便于选型与对比。

## 文档目录

| 文档 | 内容 |
|------|------|
| [README.md](README.md) | 概述、协议一览、任务记录 |
| [a2a.md](a2a.md) | A2A 协议详细研究 |
| [mcp.md](mcp.md) | MCP 协议详细研究 |
| [acp.md](acp.md) | ACP 协议详细研究 |
| [langchain-agent-protocol.md](langchain-agent-protocol.md) | LangChain Agent Protocol 详细研究 |
| [anp.md](anp.md) | ANP 协议详细研究 |
| [lacp.md](lacp.md) | LACP 协议详细研究 |

## 任务记录

| 任务 | 状态 |
|------|------|
| 整理 agent 通信协议（A2A、LACP、ACP、ANP 等） | 已完成 |
| 写入 docs/agent-protocols | 已完成 |
| 协议一览增加权威×流行度综合打分 | 已完成 |
| 一览中的内容展开详细研究（各协议独立文档） | 已完成 |

---

## 1. 概述

当前有多套面向「agent 之间沟通」或「agent 与工具/上下文」的协议与标准，大致分为：

- **工业标准**：A2A、ACP（已并入 A2A 生态）、MCP、LangChain Agent Protocol、ANP
- **学术协议**：LACP（南洋理工大学，NeurIPS 2025）

综述论文（arXiv 2505.02279）建议**分阶段采用**：MCP（工具接入）→ ACP（多模态消息与发现）→ A2A（协作任务）→ ANP（去中心化市场）。

---

## 2. 协议一览

| 协议 | 发起/主导 | 侧重 | 权威×流行度综合 | 备注 |
|------|-----------|------|-----------------|------|
| [**A2A**](a2a.md) | Google + 50+ 厂商，Linux Foundation | Agent↔Agent 协作、任务、企业 | ★★★★★ | 开放标准，LF 治理，100+ 厂商背书，企业采用快 |
| [**MCP**](mcp.md) | Anthropic | Agent↔工具/上下文 | ★★★★★ | 2024 提出，LF 承接；约 97M 月下载、10K+ 服务器，主流 AI 平台原生支持 |
| [**ACP**](acp.md) | IBM 等，现归 A2A/LF | REST、多模态消息 | ★★★★☆ | 已并入 A2A 治理，产业权威高、单独流行度渐弱 |
| [**LangChain Agent Protocol**](langchain-agent-protocol.md) | LangChain | Runs/Threads/Store，框架互操作 | ★★★☆☆ | 面向运行与部署，框架内采用，生态中等 |
| [**ANP**](anp.md) | 社区/开源 | 去中心化、DID、agent 网络与市场 | ★★★☆☆ | Apache-2.0，愿景明确，采用偏早期 |
| [**LACP**](lacp.md) | 南洋理工大学（学术） | 语义+事务+传输标准化 | ★★★☆☆ | NeurIPS 2025，学术权威高，产业采用尚早 |

**综合说明**：本列按「权威」（背书、标准组织、学术/产业地位）与「流行度」（采用量、生态、平台支持）综合打分（★ 1–5），依据公开报道与综述（arXiv 2505.02279、Linux Foundation/Anthropic 公告、行业对比文章）。MCP、A2A 当前产业采用与背书最强；ACP 并入 A2A 后权威延续；LACP 学术认可高、落地尚少。

**长连接与流式**：以下协议使用或支持长连接/流式传输。

| 协议 | 长连接/流式方式 | 说明 |
|------|-----------------|------|
| **A2A** | **SSE（Server-Sent Events）** | `Send Streaming Message`、`Subscribe to Task` 保持 HTTP 连接打开，实时推送任务状态与产出；断线可用 `tasks/resubscribe` 重连。 |
| **MCP** | **stdio + SSE** | **stdio**：客户端以子进程启动 Server，stdin/stdout 持久管道；**SSE**：远程时用 HTTP + SSE，服务端保持连接向客户端推送。 |
| **ACP** | **SSE** | 消息可经 SSE 推送；并入 A2A 后语义与 A2A 流式一致。 |
| LangChain / ANP / LACP | 未在规范中强调 | 以请求/响应或异步回调为主；是否长连接取决于具体实现与传输层。 |

---

## 3. 各协议简要（详细研究见上表链接）

- **[A2A](a2a.md)**：Google + LF，Agent↔Agent 协作与任务；三层（数据模型 / 抽象操作 / 协议绑定），Agent Card 发现，50+ 厂商背书。
- **[MCP](mcp.md)**：Anthropic → LF，Agent↔工具/上下文；JSON-RPC，Resources/Prompts/Tools，约 97M 月下载、主流平台原生支持。
- **[ACP](acp.md)**：IBM 等，现归 A2A/LF；REST、多模态消息，原仓库已归档，迁移到 A2A。
- **[LangChain Agent Protocol](langchain-agent-protocol.md)**：LangChain，Runs/Threads/Store，面向运行与编排、框架互操作。
- **[ANP](anp.md)**：社区/开源，去中心化、DID:WBA、agent 网络与市场；三层（身份/元协议/应用）。
- **[LACP](lacp.md)**：南洋理工，NeurIPS 2025；语义（PLAN/ACT/OBSERVE）+ 事务（JWS、两阶段提交）+ 传输，学术权威高、产业采用尚早。

---

## 4. 分阶段采用建议（综述）

arXiv **2505.02279**（2025 年 5 月）的综述 *A Survey of Agent Interoperability Protocols: MCP, ACP, A2A, and ANP* 建议按用途分阶段采用：

| 阶段 | 协议 | 主要用途 |
|------|------|----------|
| 1 | **MCP** | 工具与上下文接入 |
| 2 | **ACP** | 结构化、多模态消息与 agent 发现（现并入 A2A 生态） |
| 3 | **A2A** | 协作式任务执行、企业工作流 |
| 4 | **ANP** | 去中心化、开放网络与 agent 市场 |

---

## 5. 参考

- A2A 规范：<https://a2a-protocol.org/latest/specification/>
- A2A 介绍（Google）：<https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability>
- LACP：<https://lixin.ai/LACP/>
- ACP 介绍（IBM）：<https://www.ibm.com/think/topics/agent-communication-protocol>
- ANP：<https://agentnetworkprotocol.com/en/docs>
- LangChain Agent Protocol：<https://github.com/langchain-ai/agent-protocol>
- 综述：arXiv 2505.02279 — *A Survey of Agent Interoperability Protocols: MCP, ACP, A2A, and ANP*
