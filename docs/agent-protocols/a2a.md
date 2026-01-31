# Agent2Agent (A2A) 协议 — 详细研究

## 1. 背景与发起

- **发起方**：Google 牵头，2025 年 4 月发布。
- **治理**：现由 **Linux Foundation** 维护（2025 年 6 月正式接纳），保持厂商中立与长期治理。
- **规范**：<https://a2a-protocol.org/latest/specification/>（当前为 Release Candidate v1.0，另有 0.x 历史版本）。

## 2. 目标与定位

A2A 是面向 **Agent↔Agent** 的开放标准，目标包括：

- **互操作**：弥合不同厂商、框架、语言的 agent 系统之间的通信鸿沟。
- **协作**：支持任务委托、上下文交换、多 agent 协同完成用户请求。
- **发现**：通过 Agent Card 动态发现并理解其他 agent 的能力。
- **灵活性**：支持同步请求/响应、流式实时更新、异步 Webhook 推送（长任务、人机协同）。
- **安全**：对齐企业级实践（认证、授权、隐私、追踪、监控）。
- **异步优先**：原生支持长任务与人机在环。
- **不透明执行**：基于声明能力与交换信息协作，不暴露内部状态、记忆或工具实现。

## 3. 技术架构（三层）

### 3.1 Layer 1：规范数据模型（Canonical Data Model）

- 以 **Protocol Buffers** 定义，为协议无关的核心结构。
- **权威定义**：`spec/a2a.proto` 为唯一规范性来源；JSON 等为派生产物。

| 对象 | 说明 |
|------|------|
| **Task** | 工作单元，含 id、contextId、status、artifacts、history、metadata。 |
| **Message** | 单轮通信，含 messageId、contextId、taskId、role、parts、metadata、extensions、referenceTaskIds。 |
| **AgentCard** | Agent 自描述清单：身份、能力、技能、端点、鉴权要求。 |
| **Part** | 消息/产出内容的最小单位：text、raw、url、data 等。 |
| **Artifact** | 任务产出（文档、图像、结构化数据），由 Part 组成。 |
| **Extension** | 扩展机制，用于在核心规范外增加能力或数据。 |

**Task 状态（TaskState）**：SUBMITTED、WORKING、COMPLETED、FAILED、CANCELED、INPUT_REQUIRED、REJECTED、AUTH_REQUIRED 等。

### 3.2 Layer 2：抽象操作（Abstract Operations）

| 操作 | 作用 |
|------|------|
| Send Message | 发起交互，返回 Task 或直接 Message。 |
| Send Streaming Message | 带流式状态/产出更新的发送。 |
| Get Task | 轮询任务状态与产出。 |
| List Tasks | 按 contextId、status、分页等筛选任务列表。 |
| Cancel Task | 取消进行中任务。 |
| Subscribe to Task | 建立流式连接接收任务更新。 |
| Create/Get/List/Delete Push Notification Config | 配置 Webhook 接收异步任务更新。 |
| Get Extended Agent Card | 鉴权后获取更详细的 Agent Card（可选能力）。 |

**多轮与上下文**：通过 `contextId` 聚合多 Task/Message；`taskId` 由服务端生成；支持 INPUT_REQUIRED 状态以请求用户补充输入。

**任务更新方式**：轮询（Get Task）、流式（Stream Message / Subscribe to Task）、推送（Webhook）。流式与推送需在 Agent Card 中声明能力。

### 3.3 Layer 3：协议绑定（Protocol Bindings）

- **JSON-RPC**、**gRPC**、**HTTP+JSON/REST** 为官方绑定；可扩展自定义绑定。
- 多绑定需功能等价、行为一致、错误与鉴权语义一致。

**方法映射示例（REST）**：

- `POST /message:send` — Send Message  
- `POST /message:stream` — Send Streaming Message  
- `GET /tasks/{id}` — Get Task  
- `GET /tasks` — List Tasks  
- `POST /tasks/{id}:cancel` — Cancel Task  
- `POST /tasks/{id}:subscribe` — Subscribe to Task  
- `GET /extendedAgentCard` — Get Extended Agent Card  

**发现**：Agent Card 可通过 `.well-known/agent.json` 等发布；含 `supportedInterfaces`（URL、protocolBinding、protocolVersion）、`capabilities`（streaming、pushNotifications、extendedAgentCard、extensions）、`skills`、`securitySchemes` 等。

### 3.4 安全与错误

- **鉴权**：支持 API Key、HTTP Auth、OAuth2、OpenID Connect、mTLS 等（OpenAPI 3.2 Security Scheme 风格）。
- **A2A 专用错误**：TaskNotFoundError、TaskNotCancelableError、PushNotificationNotSupportedError、ContentTypeNotSupportedError、VersionNotSupportedError 等，在各绑定中有明确映射（如 HTTP 404/409/400/415）。

### 3.5 版本与扩展

- 版本以 Major.Minor 标识（如 1.0）；客户端可带 `A2A-Version` 头；服务端不支持则返回 VersionNotSupportedError。
- 扩展通过 Agent Card 的 `extensions` 声明，客户端通过 `A2A-Extensions` 等声明启用；支持 Message/Artifact 等扩展点。

## 4. 生态与采用

- **背书**：50+～100+ 科技公司（Atlassian、Salesforce、SAP、ServiceNow、Workday、埃森哲、德勤、麦肯锡等）。
- **综述**：arXiv 2505.02279 将其与 MCP、ACP、ANP 并列，建议在企业协作任务阶段采用 A2A；ACP 已并入 A2A 治理。

## 5. 与其它协议关系

- **MCP**：互补。MCP 负责 Agent↔工具/上下文；A2A 负责 Agent↔Agent 协作与任务。
- **ACP**：ACP 已并入 A2A 生态，迁移指南见 agentcommunicationprotocol.dev。
- **ANP**：A2A 偏企业/中心化协作；ANP 偏去中心化网络与市场，可阶段式配合（先 A2A 后 ANP）。

## 6. 参考

- A2A 规范：<https://a2a-protocol.org/latest/specification/>
- What is A2A：<https://a2a-protocol.org/topics/what-is-a2a/>
- Google 公告：<https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability>
- Linux Foundation 新闻：<https://www.linuxfoundation.org/press/linux-foundation-launches-the-agent2agent-protocol-project>
