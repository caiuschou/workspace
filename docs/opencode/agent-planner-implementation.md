# Agent 与 Planner 实现讲解

基于对 OpenCode 源码与架构文档的阅读，用**与语言无关**的方式讲解 Agent 与 Planner 的实现要点。阅读并理解本文后，读者可以自行用任意语言重新实现一个具备「能写代码」能力的 Agent，或在此基础上扩展 Planner。

- **前置**：[paper-agentic-coding-architecture.md](paper-agentic-coding-architecture.md)（架构总览）、[core-tech-coding-code-detail.md](core-tech-coding-code-detail.md)（代码级对应）
- **配置**：代理的配置与命名见 [agents.md](agents.md)；配置中的「Plan」代理指只读代理，与架构上的 Planner 无直接对应。

---

## 1. 概念区分

| 概念 | 含义 |
|------|------|
| **Agent** | 本架构中的**循环执行体**：在单次用户请求内，反复「调用 LLM → 处理输出与工具调用 → 执行工具 → 把结果写回消息 → 再调 LLM」，直到模型决定结束或发生取消/错误。对应 **ReAct**（Reasoning + Acting）范式，**不先显式生成多步计划**。 |
| **Planner** | **Plan-and-Execute** 中的「规划」模块：先产出显式多步计划（步骤列表或 DAG），再由执行模块按计划逐步调用工具或子模型。OpenCode 当前**未**实现独立 Planner，本文说明其原理与可选扩展方式。 |

---

## 2. Agent 实现（ReAct 式）

### 2.1 核心职责与持有状态

Agent 的职责可以归纳为：

1. **维护会话与历史**：会话 ID、该会话的消息列表（可含摘要截断）。
2. **维护工具注册表**：名称 → 工具实现（含描述与参数 schema）。
3. **提供单次运行入口**：给定「用户输入 + 可选附件」，运行直到结束或取消。
4. **在单次运行内驱动循环**：消息历史 + 工具列表 → LLM 流式请求 → 处理事件（文本/思考/工具调用）→ 执行工具 → 将助手消息与工具结果追加到历史 → 若结束原因为「工具调用」则继续下一轮，否则结束。

因此，实现时需要以下**核心状态**（与语言无关的抽象）：

- **会话存储**：`session_id → 该会话的消息列表`（或可替换为「消息存储」接口）。
- **工具注册表**：`List<Tool>` 或 `Map<name, Tool>`，每个 Tool 至少包含：名称、描述、参数 schema、执行函数 `(ctx, tool_call) → result`。
- **当前运行上下文**：当前会话 ID、当前请求的取消信号、可选「当前正在处理的 assistant 消息 ID」等。
- **与 LLM 的桥梁**：Provider 抽象（见下）。

### 2.2 主入口与单次运行的流程（伪代码）

以下用伪代码描述「单次用户请求」的完整流程，便于在不同语言中复现。

```
function Run(ctx, sessionId, userContent, attachments):
  1. 若该 session 已有运行中的请求 → 返回「忙」或拒绝
  2. 创建可取消的 genCtx，并登记到「当前运行」表（便于 Cancel(sessionId)）
  3. 在后台执行：
     a. 将 attachments 转为消息内容部件（如 BinaryContent）
     b. ProcessGeneration(genCtx, sessionId, userContent, attachmentParts)
     c. 结束后从「当前运行」表移除，并通知调用方「完成」或「取消/错误」
  4. 向调用方返回「事件流」或 Promise，用于推送进度（如流式文本、工具调用开始、完成等）
```

```
function ProcessGeneration(ctx, sessionId, userContent, attachmentParts):
  1. 加载消息历史
     history = Messages.List(ctx, sessionId)
     若该会话存在「摘要消息」：
       仅保留该摘要及之后的消息，并将摘要视为一条「用户消息」作为上下文起点

  2. 创建并持久化「用户消息」
     userMsg = Message(role=User, parts=[userContent, ...attachmentParts])
     Messages.Append(sessionId, userMsg)
     msgHistory = append(history, userMsg)

  3. 循环（直到退出）：
     a. 若 ctx 已取消 → 将当前轮标记为「已取消」，持久化并返回
     b. (assistantMsg, toolResults, err) = StreamAndHandleEvents(ctx, sessionId, msgHistory)
     c. 若 err == 取消 → 同上
     d. 若 assistantMsg.FinishReason == ToolUse 且 toolResults != nil：
        msgHistory = append(msgHistory, assistantMsg, toolResults)
        继续下一轮（回到 3a）
     e. 否则：本轮结束，返回最终 assistantMsg（及可选 toolResults）
```

要点：**只有**当「结束原因 = 工具调用」且**已成功得到工具结果**时，才把 assistant 消息与工具结果追加到历史并继续循环；否则（结束回合、达到上限、错误、权限拒绝等）直接结束。

### 2.3 单轮：StreamAndHandleEvents（流式 LLM + 工具执行）

这是 Agent 与 LLM、工具打交道的核心。用伪代码描述：

```
function StreamAndHandleEvents(ctx, sessionId, msgHistory):
  1. 调用 LLM（流式）
     eventStream = Provider.StreamResponse(ctx, msgHistory, tools)
     // Provider 内部将 tools 转为该后端 API 的 function/tool 定义

  2. 创建本轮的「助手消息」（用于累积本轮的文本、思考、工具调用）
     assistantMsg = Message(role=Assistant, parts=[])
     将 sessionId、assistantMsg.id 写入 ctx，供工具内使用（如权限、历史）

  3. 消费 eventStream，按事件类型更新 assistantMsg 并持久化
     for event in eventStream:
       switch event.Type:
         case ThinkingDelta:   assistantMsg.AppendReasoning(event.Content)
         case ContentDelta:    assistantMsg.AppendContent(event.Content)
         case ToolUseStart:    assistantMsg.AddToolCall(event.ToolCall)   // id, name, 初始参数
         case ToolUseDelta:    assistantMsg.AppendToolCallInput(event.ToolCallId, event.Delta)
         case ToolUseStop:     assistantMsg.FinishToolCall(event.ToolCallId)
         case Complete:        assistantMsg.SetToolCalls(event.ToolCalls)
                               assistantMsg.SetFinishReason(event.FinishReason)
                               // 可选：记录用量 event.Usage
         case Error:           if event.IsCanceled then return (..., Canceled)
                               else return (..., event.Error)

  4. 若 assistantMsg 无工具调用 → 返回 (assistantMsg, nil, nil)

  5. 按顺序执行每个工具调用
     toolResults = []
     for tc in assistantMsg.ToolCalls():
       if ctx.Done(): 将剩余项填为「用户取消」，break
       tool = tools.Find(tc.Name)
       if tool == nil: toolResults.append(ToolResult(tc.Id, "Tool not found", IsError=true)); continue
       result, err = tool.Run(ctx, ToolCall{Id: tc.Id, Name: tc.Name, Input: tc.Input})
       if err == PermissionDenied:
         将当前及后续 tool 结果填为「权限拒绝」
         assistantMsg.SetFinishReason(PermissionDenied)
         break
       toolResults.append(ToolResult(tc.Id, result.Content, result.Metadata, result.IsError))

  6. 构造「工具消息」
     toolMsg = Message(role=Tool, parts=toolResults)
  7. 返回 (assistantMsg, toolMsg, nil)
```

实现时需注意：

- **流式**：文本/思考增量应边收边写入 assistant 消息并持久化，便于 TUI 或 UI 实时展示。
- **工具调用与结果一一对应**：每条 `ToolResult` 带 `ToolCallID`，与助手消息中的 `ToolCall.ID` 对应；工具消息中结果顺序与助手消息中调用顺序一致。
- **权限拒绝**：若某个工具返回「权限拒绝」，应中止后续工具执行，并将本轮的 FinishReason 设为 PermissionDenied，循环在 ProcessGeneration 中不再继续。

### 2.4 消息与工具调用协议（可复现的数据结构）

要在任意语言中复现，需要统一以下抽象（与具体 API 的 JSON 可互相转换）。

**消息角色**

- `User`：用户输入，可含文本与附件（如文件内容、图片）。
- `Assistant`：模型输出，可含文本、思考（若支持）、若干**工具调用**，以及结束标记。
- `System`：系统提示词，通常由 Provider 在请求时注入，不单独持久化为会话内消息。
- `Tool`：单条消息可包含多个**工具结果**，与上一条 Assistant 消息中的工具调用一一对应（通过调用 ID 关联）。

**内容部件（Content Part）**

- 纯文本、思考内容、图片/二进制附件。
- **ToolCall**：`Id`（调用标识）、`Name`（工具名）、`Input`（参数字符串，通常为 JSON）、`Finished`（是否已收齐参数）。
- **ToolResult**：`ToolCallID`、`Name`、`Content`、`Metadata`、`IsError`。
- **Finish**：结束原因、时间戳等。

**结束原因（FinishReason）**

- `EndTurn`：模型决定结束回合。
- `MaxTokens`：达到最大 token。
- **`ToolUse`**：模型希望执行工具，需继续 Agent 循环。
- `Canceled`、`Error`、`PermissionDenied` 等。

**工具定义（供 Provider 转为各 API 的 schema）**

- 每个工具：`Name`、`Description`、`Parameters`（参数名、类型、是否必填）。
- 执行接口：`Run(ctx, ToolCall) → (ToolResponse, error)`，其中 `ToolResponse` 含 `Content`、`Metadata`、`IsError`。

按上述协议实现「消息存储 + 工具注册表 + StreamAndHandleEvents + ProcessGeneration」，即可在任意语言中复现 ReAct 式 Agent 循环。

### 2.5 取消与并发

- 每个会话的「当前运行」绑定一个可取消的 context；对外提供 `Cancel(sessionId)`，在 ProcessGeneration 中检查 `ctx.Done()` 并提前退出。
- 同一会话同一时刻只允许一个运行中的请求（忙则拒绝新请求）。

---

## 3. Planner 实现（Plan-and-Execute）

### 3.1 与 Agent 的差异

| 维度 | Agent（ReAct） | Planner（Plan-and-Execute） |
|------|----------------|-----------------------------|
| 规划形式 | 隐式，每轮由 LLM 即时决定「下一步」 | 显式，先产出多步计划（步骤列表或 DAG） |
| 执行方式 | 每轮：LLM 输出 → 执行工具 → 结果写回 → 再调 LLM | 先「规划节点」生成计划，再「执行节点」按步执行（每步可调工具或子模型） |
| 环境反馈 | 每步都依赖上一步的工具/观察结果 | 规划阶段可少依赖环境；执行阶段才密集依赖 |
| 成本/延迟 | 步数 × 单次 LLM 调用 | 规划一次 + 执行多次（可复用或轻量执行器） |
| 适应性 | 高，可随时根据结果改策略 | 依赖重规划或 Reflection，否则适应性较低 |

### 3.2 两阶段抽象（与语言无关）

**阶段一：规划**

- 输入：用户请求、可选上下文（如当前会话摘要、当前工作目录）。
- 输出：**计划**（Plan）。计划可以是：
  - 步骤列表：`[step1, step2, ...]`，每步为自然语言描述或结构化目标；
  - 或 DAG：节点为子目标，边为依赖关系。
- 实现方式：通常由一次（或少量）LLM 调用完成，提示词要求「将任务分解为可执行的步骤，不执行具体工具」。

**阶段二：执行**

- 输入：计划、当前环境（如消息历史、工作目录）。
- 行为：按步骤顺序（或按 DAG 拓扑）执行每一步。每一步可以是：
  - 调用工具（如 read、edit、bash），将结果 append 到「执行历史」；
  - 或调用子模型/Agent：在「当前步骤描述 + 已有执行历史」下跑一轮 ReAct，直到该步完成或失败。
- 输出：步骤结果序列；可选「未完成/失败」标记，用于重规划或 Reflection。

### 3.3 最小可复现的 Planner 伪代码

```
function PlanAndExecute(ctx, userRequest):
  plan = Plan(ctx, userRequest)           // 一次 LLM 调用，产出步骤列表
  results = []
  for step in plan.Steps:
    if ctx.Done(): break
    outcome = ExecuteStep(ctx, step, results)  // 执行单步：可调工具或子 Agent
    results.append(outcome)
    if outcome.Failed && plan.AllowReplan:
      plan = Replan(ctx, userRequest, results)
  return results
```

其中 `ExecuteStep` 可以是：直接根据 step 描述调用固定工具（如「读文件 X」→ 调用 read），或把 step 描述 + 已有 results 作为输入，交给一个 **ReAct 式 Agent** 跑一轮（推荐：这样单步内仍可「读→改→查」）。后者即「上层 Planner + 下层 Agent」的融合。

### 3.4 OpenCode 为何先不做独立 Planner

- 自动编程场景中，「读→改→查」强依赖**当前文件状态与 LSP 诊断**，每步决策需要紧跟最新反馈；先验多步计划容易因代码库复杂和用户意图模糊而失效。
- 实现成本：纯 ReAct 只需一个循环 + 消息/工具协议；Plan-and-Execute 需要定义计划表示、执行器状态机、可选重规划与 Reflection。
- 因此 OpenCode 以**纯 ReAct 式 Agent** 为基线；若将来需要「多步结构 + 成本/控制流」优势，可在**不改变核心消息与工具协议**的前提下，在上层增加可选 Planner（见下节）。

---

## 4. Agent 与 Planner 的融合方式（可选扩展）

在已有 Agent 循环与消息/工具协议的基础上，可以这样扩展 Planner 能力，而不重写核心。

1. **分层融合（上层 Planner + 下层 Agent）**  
   Planner 产出高层步骤（如「1) 读需求 2) 改 A 文件 3) 改 B 文件 4) 跑测试」）；每一步由现有 ReAct Agent 在「当前子目标 + 当前环境」下执行，直到该步完成或失败再进入下一步。计划只提供结构，具体「读哪份文件、改哪几处」仍由 Agent 每轮决定。

2. **轻量规划 + 反应执行**  
   在主循环前做**一次**轻量 LLM 调用，只生成 3–5 条高层步骤或检查点，不展开到具体工具调用；主循环仍是 ReAct，把该计划作为「路标」注入系统提示或上下文，Agent 可偏离路标以应对反馈。

3. **按需重规划**  
   默认纯 Agent；当检测到失败、死循环或用户触发时，插入一次规划阶段：根据当前状态与目标重新生成计划，再继续以 Agent 按新计划（或更新后的步骤描述）执行。

4. **统一循环内的「计划状态」**  
   同一 Agent 循环内，LLM 既可输出工具调用，也可输出对「当前计划」的增删改（如「将步骤 3 提前」「增加一步：先读 X」）；计划作为可变内部状态随执行演进，执行仍按「当前计划 + 当前消息」决定是否调用工具。这样规划与执行在同一循环中交替进行。

若你自实现时先完成 **Agent 循环 + Provider + 工具协议**，再按上述之一增加「计划」层，即可在保持接口一致的前提下支持 Planner 能力。

---

## 5. 自实现清单（按顺序可实现）

读者可按下列清单在任意语言中从零实现一个「能写代码」的 Agent，并可选扩展 Planner。

### 5.1 消息与存储

- [ ] 定义**消息角色**：User / Assistant / System / Tool。
- [ ] 定义**内容部件**：文本、思考、ToolCall（Id, Name, Input, Finished）、ToolResult（ToolCallID, Content, Metadata, IsError）、Finish（FinishReason）。
- [ ] 定义**结束原因**：EndTurn、MaxTokens、ToolUse、Canceled、Error、PermissionDenied。
- [ ] 实现**消息存储**：按 session 持久化/加载消息列表；支持「摘要截断」（仅保留摘要及之后的消息）。

### 5.2 工具

- [ ] 定义**工具接口**：Name、Description、Parameters、`Run(ctx, ToolCall) → (ToolResponse, error)`。
- [ ] 实现**写代码三件套**（或至少其一）：  
  - **edit**：单处替换/新建/删除；参数 file_path, old_string, new_string；约束「先读再改」、old_string 唯一、写前权限请求。  
  - **patch**：多文件统一补丁；参数 patch_text；约束先读、模糊度阈值、权限。  
  - **write**：整文件覆写；参数 file_path, content；约束先读再改、权限。
- [ ] 实现若干**读与执行类**工具：如 read/view、list/ls、grep、glob、bash（写前权限可选）。
- [ ] 实现**权限服务**：写盘/删文件前请求用户授权，拒绝时返回 PermissionDenied。

### 5.3 Provider（LLM 抽象）

- [ ] 定义 **Provider 接口**：`StreamResponse(ctx, messages, tools) → eventStream`；事件至少包含：ContentDelta、ThinkingDelta、ToolUseStart/Delta/Stop、Complete（含 ToolCalls、FinishReason、Usage）、Error。
- [ ] 将**工具定义**转换为目标 API 的 function/tool schema；将 API 返回的 tool_calls 解析为统一的 ToolCall 列表。
- [ ] 实现至少一个后端（如 OpenAI 或 Anthropic）的流式 + Tool Use。

### 5.4 Agent 循环

- [ ] 实现 **Run(sessionId, userContent, attachments)**：校验忙、创建可取消 ctx、调用 ProcessGeneration、对外暴露事件流或 Promise。
- [ ] 实现 **ProcessGeneration**：加载历史、摘要截断、创建用户消息、循环调用 StreamAndHandleEvents；仅当 FinishReason==ToolUse 且 toolResults 非空时追加历史并继续循环。
- [ ] 实现 **StreamAndHandleEvents**：调用 Provider.StreamResponse、创建 assistant 消息、消费事件流更新 assistant 消息、按序执行工具、组装 Tool 消息并返回。
- [ ] 实现 **Cancel(sessionId)** 与「同一会话单运行」的并发控制。
- [ ] 系统提示词：约束「先读再改、绝对路径、最小改动、不自动提交」等（见 paper 第 8 节）。

### 5.5 可选增强

- [ ] **Diff**：写前生成可读 diff，用于权限展示与工具返回元数据。
- [ ] **LSP**：写盘后取诊断信息附加到工具返回，供模型下一轮修正。
- [ ] **文件变更历史**：按会话记录版本，便于追溯或回滚。
- [ ] **Planner 扩展**：在现有 Agent 之上增加「规划阶段」或「计划状态」（见第 4 节），不改变消息与工具协议。

完成 5.1–5.4 即具备「能写代码」的 ReAct Agent；5.5 可按需添加。若需 Plan-and-Execute，再按第 3 节实现规划节点与执行节点，并与 5.4 中的 Agent 组合（例如每步由 Agent 执行）。

---

## 6. 参考文档

- [paper-agentic-coding-architecture.md](paper-agentic-coding-architecture.md) — 架构总览、消息与工具协议、Agent 与 Planner 学术对照
- [core-tech-coding-code-detail.md](core-tech-coding-code-detail.md) — 与 OpenCode 代码路径的逐模块对应
- [core-tech-coding.md](core-tech-coding.md) — 核心技术栈摘要
- [agents.md](agents.md) — 代理配置与「Plan」代理的命名说明
