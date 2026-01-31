# ReAct Agent Prompt 实践

本文档整理 ReAct（Reasoning + Acting）Agent 的 **prompt 实践与最佳实践**，供 rust-langgraph 中 Think/Act/Observe 节点与 LLM 集成时参考。来源：论文 Yao et al. 2022、PromptingGuide.ai、Xaibo 等。

## 1. 核心框架

ReAct 将**语言推理轨迹**与**任务相关动作**交错执行，形成闭环：

| 步骤 | 含义 |
|------|------|
| **Thought** | 模型对「下一步做什么」进行推理与规划 |
| **Action** | 执行外部工具/API 或给出最终答案 |
| **Observation** | 处理执行结果，更新推理 |

循环直到任务完成，使回答**基于真实观察**而非仅依赖训练知识，从而减少幻觉、支持动态规划与纠错。

## 2. Prompt 结构（五类）

实践中常将 ReAct 的 prompt 拆成多类，便于按阶段定制：

| 类型 | 作用 |
|------|------|
| **system_prompt** | 定义整体 ReAct 行为与规则 |
| **thought_prompt** | 引导「下一步思考」的生成 |
| **action_prompt** | 引导「选工具 or 给最终答案」 |
| **observation_prompt** | 引导「分析工具结果、规划下一步」 |
| **error / max_iterations_prompt** | 出错或达到步数上限时的兜底说明 |

## 3. 系统提示（System Prompt）示例

**通用 ReAct 规则：**

```
You are an agent that follows the ReAct pattern.

RULES:
1. Always start with THOUGHT to analyze the request
2. Use ACTION to call tools or provide FINAL_ANSWER
3. After tool execution, use OBSERVATION to analyze results
4. Be thorough but concise in your reasoning
5. Cite sources when using tool data

AVAILABLE PHASES:
- THOUGHT: Reason about what information you need
- ACTION: Execute tools or provide final answer
- OBSERVATION: Analyze tool results and plan next steps

Always explain your reasoning clearly and use tools when they can help.
```

**研究助手场景：**

```
You are an academic research assistant using the ReAct pattern.
Always verify information from multiple sources and cite your sources.
```

## 4. Thought / Action / Observation 提示

**Thought 提示（引导推理）：**

- 用户要什么？
- 已有哪些信息？
- 还缺什么信息？
- 哪些工具能补全？
- 下一步最优动作是什么？

示例：

```
Analyze the current situation carefully:
1. What is the user asking for?
2. What information do I already have?
3. What information do I still need?
4. Which tools could help me get this information?
5. What's my next best step?

Provide your THOUGHT with clear reasoning about your approach.
```

**Action 提示（工具 vs 最终答案）：**

- 需要更多信息 → 选最相关工具、一次只调一个。
- 信息足够 → 用 `FINAL_ANSWER: [your complete response]`，并包含依据与来源。

**Observation 提示（分析工具结果）：**

- 工具返回了什么？
- 是否准确、相关？
- 是否足以回答用户问题？
- 是否还需其他工具？
- 是否可以给出最终答案？

## 5. 论文中的 Few-Shot 轨迹格式

Yao et al. 2022 在 HotpotQA 等任务中使用 **Thought–Action–Observation** 轨迹作为 few-shot 示例，例如（简化）：

```
Question What is the elevation range for the area that the eastern sector of the Colorado orogeny extends into?

Thought 1 I need to search Colorado orogeny, find the area that the eastern sector of the Colorado orogeny extends into, then find the elevation range of the area.
Action 1 Search[Colorado orogeny]
Observation 1 The Colorado orogeny was an episode of mountain building...

Thought 2 It does not mention the eastern sector. So I need to look up eastern sector.
Action 2 Lookup[eastern sector]
Observation 2 (Result 1 / 1) The eastern sector extends into the High Plains...

Thought 5 High Plains rise in elevation from around 1,800 to 7,000 ft, so the answer is 1,800 to 7,000 ft.
Action 5 Finish[1,800 to 7,000 ft]
```

要点：

- **知识密集型 QA**：多轮 Thought–Action–Observation，强调分解问题、检索、综合。
- **决策型任务**（如 ALFWorld、WebShop）：Thought 可更稀疏，侧重动作序列。

## 6. 实现侧最佳实践

| 实践 | 说明 |
|------|------|
| **max_iterations** | 设置合理上限（如 5–15），避免死循环 |
| **retry_parsing_errors** | 应对 LLM 输出格式不稳定 |
| **工具描述** | 工具名与描述清晰，便于模型正确路由 |
| **领域定制** | 按场景定制 system/thought/action/observation 文案 |
| **开发期** | 开启 verbose 或 reasoning 日志，便于调试推理链 |

## 7. 错误与步数上限提示

**error_prompt 示例：**

```
An error occurred: {error}

As a professional assistant, I should:
1. Acknowledge the issue without technical jargon
2. Explain what I was trying to accomplish
3. Provide alternative approaches or partial answers
4. Suggest next steps for the user

Provide a helpful FINAL_ANSWER despite this setback.
```

**max_iterations_prompt 示例：**

```
I've reached my analysis limit of {max_iterations} steps.

Let me provide the best answer I can based on my research so far:
1. Summarize what I've learned
2. Identify any gaps in information
3. Provide actionable recommendations
4. Suggest how the user can get additional help if needed
```

## 8. 与 rust-langgraph 的对应关系

| ReAct 概念 | rust-langgraph 实现 |
|------------|----------------------|
| Thought + Action | **ThinkNode**：`LlmClient::invoke(messages)` → content + tool_calls |
| Action 执行 | **ActNode**：`ToolSource::call_tool` → tool_results |
| Observation | **ObserveNode**：tool_results 写回 messages，清空 tool_* |
| System / 多阶段 prompt | 在构造传给 `LlmClient::invoke` 的 `messages` 时注入（如首条 System，或按轮追加） |

当前 ThinkNode 使用「固定 prompt」时，可将上述 **system_prompt / thought_prompt** 等合并进第一条系统消息或默认消息列表；若后续支持「多阶段 prompt」，可拆成 thought_prompt / action_prompt / observation_prompt 在对应阶段注入。

## 9. 参考资料

- Yao et al., 2022: [ReAct: Synergizing Reasoning and Acting in Language Models](https://arxiv.org/abs/2210.03629)
- [PromptingGuide.ai – ReAct](https://www.promptingguide.ai/techniques/react)
- [Xaibo – Customize ReAct reasoning prompts](https://xaibo.ai/how-to/orchestrator/customize-react-prompts/)
- 本仓库：[13-react-agent-design.md](13-react-agent-design.md)、[15-llm-react-agent.md](15-llm-react-agent.md)
