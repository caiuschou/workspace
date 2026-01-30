# OpenCode 能写代码的核心技术说明

基于对 `thirdparty/opencode` 代码库的分析，总结其「能写代码」的核心技术栈与流程。**代码级逐模块说明**见 [core-tech-coding-code-detail.md](core-tech-coding-code-detail.md)；**与编程语言无关的论文体描述**见 [paper-agentic-coding-architecture.md](paper-agentic-coding-architecture.md)。

## 1. 整体架构：Agent + LLM + Tools

OpenCode 能写代码的核心是 **Agent 循环**：大模型做决策、调用工具、根据工具结果继续推理，直到任务完成或用户取消。

```
用户输入 → Agent.Run() → 消息历史 + 工具列表 → LLM (StreamResponse)
                ↑                                    ↓
                └──── 工具结果作为新消息 ──────────── 模型输出 (文本 / Tool Use)
```

- **Agent**：`internal/llm/agent/agent.go`，维护会话、消息历史、工具列表，驱动「调用 LLM → 处理 Tool Use → 把结果写回消息 → 再调 LLM」的循环。
- **Provider**：`internal/llm/provider/`，封装各家 LLM API（OpenAI、Anthropic、Gemini、Copilot 等），统一 `StreamResponse(ctx, messages, tools)`，支持流式输出和 **Tool Use（函数调用）**。
- **Tools**：`internal/llm/tools/`，实现具体能力；**写代码**主要依赖 `edit`、`patch`、`write`，辅以 `bash`、`view`、`grep`、`glob` 等读与执行能力。

因此，**能写代码** = **LLM 的 Tool Use 能力 + 一套「改文件」的工具实现 + Agent 循环把「模型决策」和「工具执行」串起来**。

## 2. 写代码依赖的三大工具

| 工具 | 作用 | 实现要点 |
|------|------|----------|
| **edit** | 按「old_string → new_string」做单处替换（或新建/删内容） | 精确定位、唯一匹配、可请求用户授权；可接 LSP、diff、history |
| **patch** | 一次应用多文件、多处的统一 diff | 自定义块格式（`*** Begin Patch` / `*** Update File` / `*** End Patch`），原子应用 |
| **write** | 整文件覆写 | 适合新建文件或大段重写 |

- **edit**：`internal/llm/tools/edit.go`  
  - 参数：`file_path`（绝对路径）、`old_string`、`new_string`。  
  - 新建：`old_string` 空；删内容：`new_string` 空。  
  - 要求 `old_string` 在文件内**唯一**（带足够上下文），避免误改。  
  - 内部用 `internal/diff` 做差异计算，可选 LSP 通知、history 记录。

- **patch**：`internal/llm/tools/patch.go`  
  - 参数：`patch_text`，多文件、多块编辑。  
  - 格式约定：`*** Update File` / `*** Add File` / `*** Delete File` + 类 unified diff 的 `@@` 上下文与 `-`/`+` 行。  
  - 一次解析、统一应用，适合「改多个文件、多处」的重构或需求。

- **write**：`internal/llm/tools/write.go`  
  - 参数：`file_path`、`content`，直接写整文件。

三者都通过 **permission.Service** 在写盘前可请求用户确认（符合「安全、可控」的设计）。

## 3. Agent 循环如何驱动「写代码」

核心逻辑在 `internal/llm/agent/agent.go` 的 `processGeneration`：

1. **组消息**：当前会话的历史消息 + 本条用户消息（含可选的附件）。
2. **调 LLM**：`streamAndHandleEvents` → `provider.StreamResponse(ctx, msgHistory, a.tools)`，把**工具定义**（名称、描述、参数 schema）一并传给模型。
3. **流式处理**：边收 token 边处理事件；若模型返回 **Tool Use**（如 `edit`/`patch`/`write`），则解析参数、加入 assistant 消息的 `ToolCalls`。
4. **执行工具**：根据 `ToolCalls` 在本地依次执行对应工具（如 `edit`、`patch`、`bash`），得到 `ToolResponse`。
5. **写回对话**：将「assistant 消息（含 tool_calls）」+「tool 结果消息」追加到 `msgHistory`，再回到步骤 2。
6. **结束条件**：当模型不再返回 Tool Use（或返回纯文本回答）时，本轮生成结束，把最终回复给用户。

因此，**能写代码**在实现层 = 模型会**选择并正确调用** `edit`/`patch`/`write`（以及 `view`/`grep` 等），而 OpenCode 负责把这类调用**安全地执行到真实文件系统**。

## 4. 模型侧：Coder 系统提示词

写代码的行为强烈依赖 **Coder 系统提示词**，在 `internal/llm/prompt/coder.go`：

- **角色**：OpenCode CLI、agentic coding assistant，要**持续执行直到任务解决**再结束回合。
- **规范**：  
  - 需要改代码时，先读文件（FileRead）、确认路径（LS），再 edit/patch/write。  
  - 使用绝对路径；一次 edit 只改一处；多处以多次 edit 或一个 patch 完成。  
  - 改完后可要求用 `git status` / `git diff` 自检，不自动 git commit。
- **风格**：简洁、少废话、少 preamble/postamble，适合 TUI 展示。

不同 Provider（OpenAI / Anthropic）用略有差别的 base prompt（`baseOpenAICoderPrompt` / `baseAnthropicCoderPrompt`），但都强调「用工具去读、改、验证代码」。

## 5. 辅助能力：为何改得「准」

- **LSP**：`internal/lsp` 与 `diagnostics` 工具，可把诊断（错误、告警）提供给模型，便于「改完再检查」或「按错误修」。  
- **diff**：`internal/diff` 用于生成/应用补丁，保证 edit/patch 的变更可计算、可展示。  
- **permission**：写文件前可弹授权，避免误改敏感或无关文件。  
- **history**：可选记录文件变更，便于追溯或回滚。

## 6. Agent 与 Planner 的区分

本文中的 **Agent** 指驱动「写代码」的**循环执行体**：在单次会话内不断「调用 LLM → 处理 Tool Use → 执行工具 → 结果写回消息 → 再调 LLM」，直到任务结束。这是 **ReAct 式**（Reasoning + Acting）的 reactive 模式：**不先显式制定多步计划**，而是由模型每轮决定下一步（思考或调用工具）。

与另一种常见架构 **Plan-and-Execute（Planner）** 的对比：

| 维度 | OpenCode Agent（本文） | Plan-and-Execute Planner |
|------|------------------------|---------------------------|
| **流程** | 每轮 LLM 直接决定：输出文本或 Tool Use，执行后继续 | 先由「规划节点」产出多步计划，再由「执行节点」按步执行 |
| **计划** | 隐式在对话与 Tool Use 序列中 | 显式结构（如步骤列表、DAG） |
| **典型用途** | 交互式写代码、边看结果边改 | 多步流水线、子任务可并行或交给小模型 |

OpenCode 当前采用的是 **单一 Agent 循环**，没有独立的 planner 节点；规划能力依赖 **Coder 系统提示词** 和模型的 Tool Use 行为（先读再改、最小改动等）。

注意：OpenCode 配置里的 **「Plan」代理**（如 `agents.md` 中的 Plan）是**另一种含义**——指「只读」的 agent 配置（用于代码探索、分析，不开放 edit/write/bash），与架构上的「规划器（Planner）」无直接对应关系。

## 7. 小结（核心结论）

| 层次 | 技术点 |
|------|--------|
| **架构** | Agent 循环：消息历史 + LLM（StreamResponse + Tool Use）+ 工具执行 → 结果写回消息 → 再调 LLM |
| **写代码的工具** | `edit`（单处替换/新建/删）、`patch`（多文件 diff）、`write`（整文件覆写） |
| **模型能力** | 多模型接入（OpenAI/Anthropic/Gemini/Copilot/...），统一通过 Provider 的 Tool Use 接口 |
| **行为与安全** | Coder 系统提示词约束「先读再改、绝对路径、最小改动」；permission + LSP + diff 保证可验证、可回滚 |

**一句话**：OpenCode 能写代码，是因为它把「大模型的函数调用（Tool Use）」和「本地 edit/patch/write + bash 等工具」通过 Agent 循环结合起来，并用系统提示词和权限/LSP/diff 让模型按「读→改→查」的方式安全地改代码。
