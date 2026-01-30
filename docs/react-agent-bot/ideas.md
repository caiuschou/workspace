# ReAct Agent Bot 扩展想法（调研整理）

基于网络调研与 MCP/ReAct 生态整理的可选扩展点，供后续迭代参考。实现优先级由产品需求决定。

---

## 1. ReAct / Agent 行为与鲁棒性

| 想法 | 说明 | 参考 |
|------|------|------|
| **max_iterations / max_retries** | 限制 Think→Act→Observe 循环次数，避免无限循环与成本失控；配合 `retry_parsing_errors` 减少 LLM 解析失败导致的提前退出 | NVIDIA Agent Toolkit、LangChain ReAct |
| **推理步骤对用户透明** | 将 thought / action / observation 作为结构化事件暴露（如 SSE 或回调），便于前端展示「正在思考」「正在调用某工具」「观察到结果」 | ReAct 模式本身、Microsoft Teams streaming bot |
| **解析失败重试** | LLM 返回的 tool_calls 格式异常时自动重试若干次再报错，降低幻觉导致的断链 | 常见 Agent 配置项 |

---

## 2. MCP 生态扩展（更多工具源）

除 Exa、Fetch 外，可选的官方/社区 MCP Server（偏「抓内容/搜索/记忆」）：

| Server | 能力 | 说明 |
|--------|------|------|
| **Brave Search MCP** | 网页/本地搜索 | 官方 [brave/brave-search-mcp-server](https://github.com/brave/brave-search-mcp-server)，与 Exa 互补 |
| **Memory MCP**（官方） | 知识图谱式持久记忆 | [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers) 内 Memory Server，可与 Store 结合或替代部分自建 Store |
| **Filesystem MCP**（官方） | 安全文件读写 | 可控目录内文件操作，适合「个人知识库」「项目上下文」 |
| **Git MCP**（官方） | 仓库读/搜/操作 | 代码库检索、diff、commit 等，适合开发者场景 |
| **Sequential Thinking MCP**（官方） | 动态问题分解 | 多步推理与子任务规划，详见 [Sequential Thinking MCP 说明](sequential-thinking-mcp.md) |
| **Time MCP**（官方） | 时间/时区 | 简单工具，便于与现有 Mock 对齐或替换 |

**实现思路**：AgentConfig 支持多组「工具源」预设（Exa / Fetch / Brave / Filesystem / …），本 crate 或 langgraph 内做 **多 ToolSource 聚合**（list_tools 合并、call_tool 按 name 路由）。

---

## 3. 记忆与 RAG

| 想法 | 说明 | 参考 |
|------|------|------|
| **长期记忆 + RAG 分工** | **LTM（Store）**：谁、之前聊过什么、偏好；**RAG**：当前权威文档/知识库。两者互补，LTM 做会话连贯，RAG 做事实增强 | AWS Bedrock、Mem0 论文 |
| **Mem0 式图记忆** | 用图结构存实体与关系，检索时按关系扩展，提升多跳推理（进阶，可选） | Mem0、图记忆论文 |
| **多用户与隐私** | Store 严格按 `user_id` + namespace 隔离；敏感内容不写入 checkpoint 的 verbose 或日志 | 常见合规需求 |

---

## 4. 流式输出与 UX

| 想法 | 说明 | 参考 |
|------|------|------|
| **流式回复** | 助手回复以 token/句子流式返回，降低首字延迟与等待感 | OpenAI/Teams streaming、Guardrails 流式校验 |
| **进度/状态反馈** | 前端可展示：「正在思考」「正在调用 web_search_exa」「正在读取网页」等，对应 Think/Act/Observe 阶段 | Microsoft Teams streaming UX、typing indicator 研究 |
| **Typing indicator** | 在未流式或网络慢时，至少显示「正在输入」类状态，提升感知响应度 | 对话 UX 最佳实践 |

实现上：图执行时通过回调或 channel 发出「阶段事件」；LLM 若支持 streaming，在 Think 节点流式输出并汇聚为最终 assistant 消息。

---

## 5. 安全与护栏（Guardrails）

| 想法 | 说明 | 参考 |
|------|------|------|
| **输入/输出校验** | 对用户输入与模型输出做格式/策略校验（如 PII 脱敏、禁止话题），流式时可逐句校验 | Guardrails AI、OpenAI Guardrails |
| **人在回路（Human-in-the-Loop）** | 敏感工具（如写文件、发邮件）执行前暂停，返回「待确认」状态，由上层确认后再继续 | OpenAI Agents SDK RunState |
| **敏感信息不落盘** | verbose / 日志中不记录完整消息体或 token，checkpoint 可配置脱敏 | 合规与隐私 |

---

## 6. 成本与性能

| 想法 | 说明 |
|------|------|
| **max_tokens / 上下文裁剪** | 单轮或总会话限制 token 数；历史消息超过窗口时用 summarization 或滑动窗口裁剪 |
| **工具结果截断** | 类似 Fetch 的 `max_length`，对 ToolResult 做长度限制再写入 messages，避免撑爆上下文 |
| **缓存** | 对相同或近似请求的 LLM 响应、工具结果做短期缓存（需考虑一致性） |

---

## 7. 小结与可执行项

- **易落地**：max_iterations / max_retries、推理步骤事件暴露、Brave Search / Filesystem / Time 等 MCP 预设、流式回复与进度反馈。
- **中期**：多 ToolSource 聚合、长期记忆与 RAG 分工、人在回路、Guardrails 集成。
- **进阶**：Mem0 式图记忆、流式 Guardrails、多租户与审计。

可在 [README.md](README.md) 任务表中按需拆成具体任务并排期。
