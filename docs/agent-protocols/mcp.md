# Model Context Protocol (MCP) — 详细研究

## 1. 背景与发起

- **发起方**：**Anthropic**，2024 年 11 月开源。
- **治理**：2025 年 12 月捐赠给 **Agentic AI Foundation**（Linux Foundation 旗下），作为创始项目。
- **规范**：<https://modelcontextprotocol.io/specification/latest>（版本 2025-11-25）。

## 2. 目标与定位

MCP 是 **Agent ↔ 工具/上下文/数据源** 的开放协议，不是 Agent↔Agent 的主战场，常与 A2A/ACP 一起构成「agent 互操作」拼图。

- **标准化**：为 LLM 应用与外部数据、工具提供统一接入方式（被比喻为「AI 的 USB-C」）。
- **场景**：AI IDE、聊天增强、自定义 AI 工作流等，让模型获得所需上下文与能力。
- **角色**：Host（发起连接的 LLM 应用）、Client（Host 内的连接器）、Server（提供上下文与能力的服务）。

## 3. 技术架构

### 3.1 基础协议

- **消息格式**：**JSON-RPC 2.0**。
- **连接**：有状态连接；服务端与客户端通过能力协商确定支持的功能。
- **传输**：支持 stdio（本地）、HTTP/SSE 等；规范定义 Transport 与生命周期。

### 3.2 服务端能力（Server Features）

| 能力 | 说明 |
|------|------|
| **Resources** | 上下文与数据，供用户或模型使用（如文件、数据库视图）。 |
| **Prompts** | 模板化消息与工作流，供用户或系统调用。 |
| **Tools** | 供模型调用的函数（如调用 API、执行计算、读写数据）。 |

每类能力均有列表、读取/调用、订阅更新等语义；Tools 含名称、参数 schema、错误与安全考量。

### 3.3 客户端能力（Client Features）

| 能力 | 说明 |
|------|------|
| **Sampling** | 服务端发起 agent 行为与递归 LLM 调用。 |
| **Roots** | 服务端发起对 URI 或文件系统边界的访问。 |
| **Elicitation** | 服务端向用户请求额外信息。 |

### 3.4 安全与信任

- **用户同意与控制**：数据访问与操作需用户明确同意；用户应能控制共享范围与执行动作。
- **数据隐私**：Host 在暴露用户数据给 Server 前需获得同意；不得在未同意下将资源数据外传。
- **工具安全**：Tools 即潜在任意执行，需显式用户同意后再调用；工具描述等除非来自可信 Server 否则视为不可信。
- **LLM Sampling**：用户需明确批准采样请求；协议限制 Server 对 prompt 的可见性。

规范在协议层无法强制上述原则，但要求实现方在应用层落实同意流、文档与访问控制。

## 4. 生态与采用

- **采用量**：约 **97M 月 SDK 下载**、**10K+ 活跃 Server**；PulseMCP 等注册表有 **5500+** Server（截至 2025）。
- **平台支持**：ChatGPT、Claude、Cursor、Gemini、Microsoft Copilot、Visual Studio Code 等一线平台原生支持 MCP。
- **厂商**：Block、Apollo、Zed、Replit、Codeium、Sourcegraph、Atlassian、Figma、Asana 等已集成或提供 Remote MCP Server。
- **SDK**：官方多语言 SDK（TypeScript、Python、Go、Kotlin、Swift、Java、C#、Ruby、Rust、PHP），支持 Server/Client、本地与远程传输。

## 5. 与其它协议关系

- **A2A**：互补。MCP 解决「模型拿什么上下文、调什么工具」；A2A 解决「agent 之间如何协作与任务编排」。综述建议先 MCP 再 A2A。
- **ACP/A2A**：ACP 已并入 A2A；与 MCP 无替代关系，可组合使用（工具用 MCP，agent 间用 A2A）。

## 6. 参考

- 规范：<https://modelcontextprotocol.io/specification/latest>
- 文档与概念：<https://modelcontextprotocol.io/docs>
- 概念 - Tools：<https://modelcontextprotocol.io/docs/concepts/tools>
- SDK：<https://modelcontextprotocol.io/docs/sdk>
- GitHub：<https://github.com/modelcontextprotocol>
- Anthropic 介绍：<https://www.anthropic.com/news/model-context-protocol>
