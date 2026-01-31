# Agent Network Protocol (ANP) — 详细研究

## 1. 背景与发起

- **发起方**：社区/开源。
- **许可**：**Apache-2.0**。
- **资源**：<https://agentnetworkprotocol.com/en/docs>；GitHub: `agent-network-protocol/AgentNetworkProtocol`。
- **定位**：自称「**agentic web 时代的 HTTP**」，面向开放、去中心化的 agent 网络与市场。

## 2. 目标与定位

- **愿景**：定义 agent 如何互联与协作，构建「开放、安全、高效」的协作网络，支持海量智能 agent。
- **问题意识**：现有互联网基础设施主要为人类设计，存在数据孤岛、上下文局限、协作成本高等问题；ANP 面向 **AI 原生** 设计，支持标准化、直接的 agent↔agent 交互，不模拟人类行为、不依赖平台专属 API。
- **场景**：去中心化 agent 发现、安全协作、开放网络中的 agent 市场。

## 3. 技术架构（三层）

### 3.1 身份与加密通信层（Identity and Encrypted Communication Layer）

- **身份**：基于 **W3C DID**（Decentralized Identifier）的去中心化身份；agent 拥有可跨服务使用的「数字护照」，无需在各平台单独注册。
- **DID:WBA**：**DID:WBA**（Web-Based Agent Decentralized Identifier）方法规范，定义 Web 端 agent 在 ANP 生态内的标识与鉴权，解决身份碎片化。
- **安全与隐私**：多种 DID 隐私策略、人/agent 授权区分、最小信息披露、**端到端加密** 通信。

### 3.2 元协议层（Meta-Protocol Layer）

- **动态协议协商**：agent 可协商使用的应用层协议与能力。
- **自组织 agent 网络**：支持 agent 自组织成网络，不依赖单一中心目录。

### 3.3 应用层（Application Layer）

- **能力描述**：基于语义的 agent 能力描述（如 **Agent Description Protocol, ADP**）。
- **协议管理**：应用层协议与互操作标准的定义与演进。

## 4. 生态与采用

- **阶段**：愿景明确、规范与白皮书公开，**采用偏早期**；常与 MCP、ACP、A2A 一起出现在综述与对比中。
- **综述**：arXiv 2505.02279 建议分阶段采用，将 ANP 放在「去中心化、开放网络与 agent 市场」阶段（MCP → ACP → A2A → ANP）。

## 5. 与其它协议关系

- **A2A**：A2A 偏企业/中心化协作与任务；ANP 偏去中心化网络与市场。可阶段配合：先 A2A 再 ANP。
- **MCP**：MCP 解决工具/上下文；ANP 解决身份与网络层，互补。
- **ACP/A2A**：ACP 已并入 A2A；ANP 与 A2A 在「中心化 vs 去中心化」上形成对照与衔接。

## 6. 参考

- 文档：<https://agentnetworkprotocol.com/en/docs>
- 技术白皮书：<https://agentnetworkprotocol.com/en/specs/01-agentnetworkprotocol-technical-white-paper>；另见 <https://www.agent-network-protocol.com/specs/white-paper.html>
- DID:WBA 方法规范：<https://agentnetworkprotocol.com/en/specs/03-did-wba-method-specification>
- GitHub：<https://github.com/agent-network-protocol/AgentNetworkProtocol>
- MCP vs ANP 对比：<https://agentnetworkprotocol.com/en/blog/mcp-anp-comparison/>
