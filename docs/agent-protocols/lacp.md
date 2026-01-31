# LACP（LLM Agent Communication Protocol）— 详细研究

## 1. 背景与发起

- **发起方**：南洋理工大学（**Xin Li, Mengbing Liu, Chau Yuen**）。
- **发表**：**NeurIPS 2025**（AI4NextG Workshop）。
- **页面**：<https://lixin.ai/LACP/>；论文/材料见站点与 OpenReview/arXiv。

## 2. 目标与定位

- **问题意识**：当前 LLM agent 通信生态**碎片化**（类似早期网络「协议战争」），缺乏统一标准，导致互操作、安全与**事务一致性**不足；现有方案（OpenAI Function Calling、LangChain、Anthropic MCP 等）多为厂商或框架专属。
- **定位**：面向 **NextG/6G** 等场景下**安全、可扩展**的多 agent 系统；强调**语义统一、事务可靠、传输安全**，适合高价值、需可验证与可审计通信的场景。
- **设计启发**：借鉴**电信**分层与可靠性思路（语义层、事务层、传输层）。

## 3. 技术架构（三层）

### 3.1 语义层（Semantic Layer）

- **统一消息类型**：用统一消息类型表达意图，减少歧义与适配成本。
- **核心类型**：**PLAN**、**ACT**、**OBSERVE**——分别表示规划、执行动作、观察结果，便于多 agent 协同与推理链清晰化。

### 3.2 事务层（Transactional Layer）

- **可靠性**：消息签名（**JWS**）、序号、事务 ID、**两阶段提交** 等机制，保证消息可验证、可审计与一致性。
- **目标**：避免「发了算」的不可靠通信，适合金融、关键任务等需要可追溯与一致性的场景。

### 3.3 传输层（Transport Layer）

- **安全、高效传输**：在语义与事务之上，负责安全、高效地把消息从一端传到另一端。

## 4. 性能与设计原则（论文数据）

- **延迟开销**：约 **3.5%**。
- **消息大小开销**：约 **30%**。
- **原则**：分层抽象、安全内建、最小核心+可扩展、内容无关（content agnostic）。

## 5. 生态与采用

- **权威**：**学术认可高**（NeurIPS 2025 Workshop）；产业采用尚早，无大规模商用报道。
- **场景**：NextG、6G、高价值多 agent 协作、需可验证与可审计通信的系统。

## 6. 与其它协议关系

- **A2A**：A2A 偏产业标准与企业协作；LACP 偏学术与电信/NextG 场景；二者在「语义+事务+传输」分层思路上有可比性，但 LACP 更强调事务与可验证性。
- **MCP**：MCP 解决工具/上下文；LACP 解决 agent 间通信的语义与事务，互补。
- **LACP 与现有碎片**：LACP 论文将 OpenAI Function Calling、LangChain、MCP 等视为碎片化现状的一部分，LACP 旨在提供统一、可验证的替代或补充。

## 7. 参考

- 官网：<https://lixin.ai/LACP/>
- NeurIPS 2025 AI4NextG Workshop 材料（见站点链接）
- arXiv：如 2510.13821 等（以站点与论文列表为准）
- OpenReview：<https://openreview.net/pdf?id=o3M9ibtZWV> 等（以实际论文链接为准）
