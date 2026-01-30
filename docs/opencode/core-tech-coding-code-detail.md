# OpenCode 能写代码：代码详细说明

基于对 `thirdparty/opencode` 源码的阅读，对「能写代码」相关核心模块做逐文件、逐流程的代码级说明。与 [core-tech-coding.md](core-tech-coding.md) 配合阅读。

---

## 1. 目录与模块关系

```
internal/
├── llm/
│   ├── agent/          # Agent 循环：会话、消息、工具驱动
│   │   ├── agent.go    # Run / processGeneration / streamAndHandleEvents
│   │   └── tools.go    # CoderAgentTools / TaskAgentTools
│   ├── provider/       # LLM 统一接口：StreamResponse + Tool Use
│   │   └── provider.go # Provider 接口、事件类型、多厂商封装
│   ├── prompt/
│   │   └── coder.go    # Coder 系统提示词（OpenAI/Anthropic）
│   └── tools/          # 具体工具实现
│       ├── tools.go    # BaseTool 接口、ToolCall/ToolResponse
│       ├── edit.go     # 单处替换 / 新建 / 删内容
│       ├── patch.go    # 多文件统一 patch
│       └── write.go    # 整文件覆写
├── message/            # 消息与内容部件（ContentPart、ToolCall、ToolResult）
├── diff/               # 差异计算与 patch 解析/应用
├── permission/         # 写盘前授权
├── history/            # 文件变更历史（可选）
└── lsp/                # 诊断反馈（可选）
```

---

## 2. Agent 循环：`internal/llm/agent/agent.go`

### 2.1 类型与接口

- **`Service`**：对外接口。核心方法：
  - `Run(ctx, sessionID, content, attachments...)` → 返回 `<-chan AgentEvent`，异步执行一轮「用户输入 → 多轮 LLM + 工具」直到结束或取消。
  - `Cancel(sessionID)`、`IsSessionBusy`、`IsBusy`：取消与忙状态。
  - `Update(agentName, modelID)`：切换模型；`Summarize(ctx, sessionID)`：会话摘要。
- **`agent`**（私有）：实现 `Service`。持有 `sessions`、`messages`、`tools []tools.BaseTool`、`provider provider.Provider`，以及可选的 `titleProvider`、`summarizeProvider`。通过 `pubsub.Broker[AgentEvent]` 发布事件；通过 `activeRequests sync.Map` 存 sessionID → cancel 函数以支持取消。

### 2.2 入口：`Run`

1. 若有附件且当前模型不支持附件，则清空附件。
2. 若该 session 正在处理请求则返回 `ErrSessionBusy`。
3. 创建可取消的 `genCtx`，将 cancel 存入 `activeRequests.Store(sessionID, cancel)`。
4. 启动 goroutine：
   - 将用户附件转为 `message.ContentPart`（如 `BinaryContent`）。
   - 调用 **`processGeneration(genCtx, sessionID, content, attachmentParts)`**。
   - 结束后从 `activeRequests` 删除、cancel、发布 `AgentEvent` 并关闭 events channel。

### 2.3 核心循环：`processGeneration`

1. **加载消息**：`a.messages.List(ctx, sessionID)` 得到历史 `msgs`。若为首条用户消息，异步调用 `generateTitle` 生成会话标题。
2. **长会话摘要**：若 session 有 `SummaryMessageID`，则从该条消息之后截断，并把该条改为 User 角色，作为「摘要后的上下文起点」。
3. **创建用户消息**：`createUserMessage` → `a.messages.Create(..., User, parts)`，得到 `userMsg`。`msgHistory := append(msgs, userMsg)`。
4. **循环**：
   - 若 `ctx.Done()` 则返回取消错误。
   - 调用 **`streamAndHandleEvents(ctx, sessionID, msgHistory)`**，得到 `agentMessage`、`toolResults`、`err`。
   - 若 err 为 `context.Canceled`，给 agent 消息加上 `FinishReasonCanceled` 并持久化，返回 `ErrRequestCancelled`。
   - 若 `agentMessage.FinishReason() == message.FinishReasonToolUse` 且 `toolResults != nil`：  
     `msgHistory = append(msgHistory, agentMessage, *toolResults)`，**continue**（下一轮继续调 LLM）。
   - 否则本轮结束，返回 `AgentEvent{Type: Response, Message: agentMessage, Done: true}`。

因此「能写代码」在 agent 层体现为：**只要模型返回 Tool Use 且工具执行成功，就把 assistant 消息 + tool 结果追加到历史并再次调用 LLM**，直到模型不再发起工具调用或出错/取消。

### 2.4 流式与工具执行：`streamAndHandleEvents`

1. **调 LLM**：`eventChan := a.provider.StreamResponse(ctx, msgHistory, a.tools)`。Provider 内部会把 `a.tools` 转为各厂商的 function/tool 定义传给模型。
2. **创建 assistant 消息**：`a.messages.Create(..., Assistant, [], model)`，得到 `assistantMsg`。将 `sessionID`、`messageID` 写入 ctx 供工具使用。
3. **消费流事件**：`for event := range eventChan` 调用 **`processEvent`**：
   - `EventThinkingDelta` → `assistantMsg.AppendReasoningContent`，更新 DB。
   - `EventContentDelta` → `assistantMsg.AppendContent`，更新 DB。
   - `EventToolUseStart` → `assistantMsg.AddToolCall(*event.ToolCall)`，更新 DB。
   - `EventToolUseStop` → `assistantMsg.FinishToolCall(event.ToolCall.ID)`，更新 DB。
   - `EventComplete` → 设置 `assistantMsg` 的 ToolCalls、FinishReason，更新 DB，并 `TrackUsage`。
   - `EventError` → 若是 `context.Canceled` 则返回 Canceled，否则返回该错误。
4. **执行工具**：根据 `assistantMsg.ToolCalls()` 顺序执行：
   - 若 ctx 已取消，后续 tool call 统一返回「Tool execution canceled by user」。
   - 按 `toolCall.Name` 在 `a.tools` 中查找 `BaseTool`；找不到则写入 `Tool not found` 的 ToolResult。
   - 找到则 `tool.Run(ctx, tools.ToolCall{ID, Name, Input})`。若返回 `permission.ErrorPermissionDenied`，则本轮及后续 tool 都填取消信息，并 `finishMessage(..., FinishReasonPermissionDenied)`，跳出。
   - 正常则把 `ToolResult{ToolCallID, Content, Metadata, IsError}` 写入 `toolResults[i]`。
5. **写回对话**：用 `a.messages.Create(..., Tool, parts)` 创建一条 Tool 消息（parts 为所有 `toolResults`），返回 `assistantMsg` 和该 Tool 消息。若没有 tool 调用则返回 `(assistantMsg, nil, nil)`。

### 2.5 工具列表来源：`internal/llm/agent/tools.go`

- **`CoderAgentTools(...)`**：返回 Coder 代理可用工具列表。包含：`Bash`、`Edit`、`Fetch`、`Glob`、`Grep`、`Ls`、`Sourcegraph`、`View`、`Patch`、`Write`、`NewAgentTool`，以及 MCP 工具；若配置了 LSP 则追加 `Diagnostics`。
- **`TaskAgentTools(lspClients)`**：只读/探索类（Glob、Grep、Ls、Sourcegraph、View），用于不需写文件的 agent。

---

## 3. Provider 抽象：`internal/llm/provider/provider.go`

### 3.1 接口与事件

- **`Provider`**：
  - `SendMessages(ctx, messages, tools)` → `*ProviderResponse`（非流式，用于 title/summarize 等）。
  - **`StreamResponse(ctx, messages, tools)`** → **`<-chan ProviderEvent`**（流式，用于主对话）。
  - `Model()` → 当前模型信息。
- **`ProviderResponse`**：`Content`、`ToolCalls []message.ToolCall`、`Usage`、`FinishReason`。
- **`ProviderEvent`**：`Type`（见下）、`Content`、`Thinking`、`Response`、`ToolCall`、`Error`。

事件类型：`EventContentStart/Delta/Stop`、`EventToolUseStart/Delta/Stop`、`EventThinkingDelta`、`EventComplete`、`EventError`、`EventWarning`。Agent 只消费其中与「内容增量、思考增量、Tool Use 起止、完成、错误」相关的部分。

### 3.2 多厂商封装

- `ProviderClient` 接口：`send`、`stream`。
- `baseProvider[C]` 泛型：持有 `providerClientOptions`（apiKey、model、maxTokens、systemMessage 及各厂商扩展）和 `client C`。`StreamResponse` 即 `p.client.stream(ctx, messages, tools)`。
- `NewProvider(providerName, opts...)` 根据 `models.ModelProvider` 选择实现：Copilot、Anthropic、OpenAI、Gemini、Bedrock、GROQ、Azure、VertexAI、OpenRouter、XAI、Local、Mock 等。多数兼容 OpenAI API 的通过 `OpenAIClient` 统一封装。

工具定义由各 client 的 `stream` 实现里，将 `[]tools.BaseTool` 转成该厂商的 function/tool schema 随请求发送；模型返回的 tool_calls 被解析为 `message.ToolCall` 通过事件抛出。

---

## 4. 消息与内容部件：`internal/message/content.go`、`message.go`

- **角色**：`User`、`Assistant`、`System`、`Tool`。
- **ContentPart**：`TextContent`、`ReasoningContent`、`ImageURLContent`、`BinaryContent`、**`ToolCall`**、**`ToolResult`**、`Finish`。
- **ToolCall**：`ID`、`Name`、`Input`、`Type`、`Finished`。Agent 在流中逐步填充并最终在 `EventComplete` 时用服务端返回的完整列表覆盖。
- **ToolResult**：`ToolCallID`、`Name`、`Content`、`Metadata`、`IsError`。由 agent 在 `streamAndHandleEvents` 里创建并写入 Tool 消息的 Parts。
- **FinishReason**：`end_turn`、`max_tokens`、**`tool_use`**、`canceled`、`error`、`permission_denied` 等。当为 `tool_use` 时 agent 继续循环。
- **Message**：`Parts` 为 `ContentPart` 切片；提供 `ToolCalls()`、`ToolResults()`、`FinishReason()`、`AppendContent`、`AddToolCall`、`SetToolCalls`、`AddFinish` 等，用于流式更新与持久化。

---

## 5. 工具抽象与写代码三件套

### 5.1 BaseTool：`internal/llm/tools/tools.go`

- **`ToolInfo`**：`Name`、`Description`、`Parameters`（map）、`Required`（字段名列表）。供 Provider 转为 API 的 function/tool 描述。
- **`ToolCall`**：`ID`、`Name`、`Input`（JSON 字符串）。
- **`ToolResponse`**：`Type`（text/image）、`Content`、`Metadata`、`IsError`。`WithResponseMetadata` 可把 diff 等结构化信息放入 Metadata。
- **`BaseTool`**：`Info() ToolInfo`、`Run(ctx, ToolCall) (ToolResponse, error)`。
- Context 键：`SessionIDContextKey`、`MessageIDContextKey`，用于权限、历史等需要 session/message 的逻辑。

### 5.2 Edit 工具：`internal/llm/tools/edit.go`

- **参数**：`EditParams`：`FilePath`、`OldString`、`NewString`（JSON）。若路径非绝对则用 `config.WorkingDirectory()` 拼成绝对路径。
- **分支**：
  - `OldString == ""` → **新建文件**：`createNewFile`。要求文件不存在；生成 diff（空 → content）；请求 permission；写盘；`files.Create` + `CreateVersion`；返回 metadata（Diff、Additions、Removals）。
  - `NewString == ""` → **删内容**：`deleteContent`。要求文件存在、已被 View 读过、未在读后被外部修改；`old_string` 在文件中**唯一**；生成 newContent 并写盘；同样 permission、history、metadata。
  - 否则 → **替换**：`replaceContent`。同样「先读再改」与「old_string 唯一」校验；`newContent = oldContent[:index] + newString + oldContent[index+len(oldString):]`；permission、写盘、history、metadata。
- **共同约束**：
  - 写前必须「先 View 过」该文件（`getLastReadTime(filePath)`），且文件在最后一次读后未被修改。
  - `old_string` 必须唯一（`strings.Index` 与 `strings.LastIndex` 相等），否则返回错误提示「provide more context」。
- **成功后**：`waitForLspDiagnostics`、`getDiagnostics` 拼到返回的 Content 后，便于模型根据 LSP 报错再改。

### 5.3 Patch 工具：`internal/llm/tools/patch.go`

- **参数**：`PatchParams`：`PatchText`（字符串）。格式见描述中的 `*** Begin Patch` / `*** Update File` / `*** Add File` / `*** Delete File` / `*** End Patch` 及类 unified diff 的 `@@` 与 `-`/`+` 行。
- **流程**：
  1. **解析涉及文件**：`diff.IdentifyFilesNeeded(patchText)`、`diff.IdentifyFilesAdded(patchText)`。对需要读的文件检查「已读且未在读后被修改」；对 Add 检查文件不存在。
  2. **加载当前内容**：`currentFiles[path] = readFile(absPath)`。
  3. **解析 patch**：`diff.TextToPatch(params.PatchText, currentFiles)` → `Patch`、`fuzz`。若 `fuzz > 3` 则报错要求更精确的 context。
  4. **转成 Commit**：`diff.PatchToCommit(patch, currentFiles)` → `Commit`（map[path]FileChange，含 Add/Update/Delete）。
  5. **逐变更请求 permission**：按 `ActionAdd`/`ActionUpdate`/`ActionDelete` 用 `GenerateDiff` 生成展示用 diff，再 `permissions.Request(...)`。
  6. **写盘**：`diff.ApplyCommit(commit, writeFn, removeFn)`，其中 writeFn/removeFn 内部会处理相对路径转绝对路径、创建目录等。
  7. **历史与 LSP**：对每个变更文件 `CreateVersion`、`recordFileWrite/Read`、`waitForLspDiagnostics`；汇总 `PatchResponseMetadata`（FilesChanged、Additions、Removals）和 diagnostics 文本返回。

### 5.4 Write 工具：`internal/llm/tools/write.go`

- **参数**：`WriteParams`：`FilePath`、`Content`。
- **逻辑**：若文件已存在，检查「自上次读以来未修改」且「内容与本次不同」；若相同则直接返回「No changes made」。然后 `GenerateDiff(oldContent, content, filePath)`、permission、`os.WriteFile`、history、LSP、返回带 Diff/Additions/Removals 的 metadata。

三者均在写盘前通过 **`permission.Service.Request(CreatePermissionRequest{...})`** 请求用户授权；若拒绝则返回 `permission.ErrorPermissionDenied`，agent 会终止本轮并标记 `FinishReasonPermissionDenied`。

---

## 6. Diff 与 Patch：`internal/diff/`

### 6.1 diff.go

- **GenerateDiff(beforeContent, afterContent, fileName)** → `(diffString, additions, removals)`。用于 edit/patch/write 的展示与 metadata，以及 permission 请求中的 Diff 展示。
- 内部还包含解析、行类型（LineAdded/Removed/Context）、Hunk、以及用于 TUI 的 side-by-side 等，与「能写代码」直接相关的是 `GenerateDiff`。

### 6.2 patch.go

- **TextToPatch(text, orig map[string]string)** → `(Patch, fuzz, error)`：解析自定义块格式的 patch 文本，`orig` 为当前文件 path → content；`fuzz` 为上下文匹配的模糊度。
- **IdentifyFilesNeeded(text)**、**IdentifyFilesAdded(text)**：从 patch 文本中识别需要读取的文件列表、以及要新增的文件列表，供 patch 工具做「先读再改」和「新增文件不存在」校验。
- **PatchToCommit(patch, orig)** → `(Commit, error)`：将 `Patch` 转成对文件系统的「变更计划」`Commit`（每 path 一个 `FileChange`：Add/Update/Delete，含 OldContent/NewContent）。
- **ApplyCommit(commit, writeFn, removeFn)**：按 Commit 依次调用 `writeFn(path, content)` 或 `removeFn(path)` 落盘。patch 工具传入的 writeFn/removeFn 会做路径解析与 `MkdirAll`。

---

## 7. Coder 系统提示词：`internal/llm/prompt/coder.go`

- **CoderPrompt(provider)**：根据 `models.ModelProvider` 选择 `baseOpenAICoderPrompt` 或 `baseAnthropicCoderPrompt`，再拼接 **getEnvironmentInfo()**（工作目录、是否 git、平台、日期、项目根 LS 结果）和 **lspInformation()**（若启用 LSP 则追加一段说明）。
- **baseOpenAICoderPrompt**：角色为 OpenCode CLI、agentic coding assistant；要求持续执行直到问题解决再结束；改代码要先读再改、用绝对路径、最小改动、改完可 git status/diff 自检、不自动 commit；风格简洁、少废话。
- **baseAnthropicCoderPrompt**：更长，包含 Memory（OpenCode.md）、Tone、Proactiveness、Following conventions、Code style、Doing tasks（搜索 → 实现 → 验证 → lint/typecheck）、Tool usage policy（可并行无依赖的 tool 同批调用、用户看不到完整 tool 输出需在回复中总结）等。同样强调先读再改、不 commit 除非用户明确要求、回答简洁。

Provider 在构造请求时通过 **WithSystemMessage(prompt.GetAgentPrompt(agentName, model.Provider))** 注入；Coder 对应的是 `GetAgentPrompt` 中返回的 Coder 提示词（见 prompt 包内其他文件）。

---

## 8. 小结（与文档对应）

| 文档小节           | 代码位置与要点 |
|--------------------|----------------|
| Agent 循环          | `agent.go`：`Run` → `processGeneration` 的 for 循环；`streamAndHandleEvents` 中 StreamResponse + processEvent + 顺序执行 ToolCalls + 写回 Tool 消息。 |
| Provider 统一接口    | `provider.go`：Provider/ProviderEvent；多厂商通过 ProviderClient 的 send/stream 实现，工具定义由各 client 转为 API schema。 |
| edit / patch / write | `tools/edit.go`、`patch.go`、`write.go`：参数校验、先读再改/唯一匹配、permission.Request、diff、写盘、history、LSP。 |
| 消息与 Tool Use     | `message` 包：ContentPart（ToolCall、ToolResult）、Message 的 ToolCalls/SetToolCalls/AddFinish、FinishReasonToolUse。 |
| Coder 提示词        | `prompt/coder.go`：CoderPrompt + 环境信息 + LSP 说明；约束读→改、路径、最小改动、不自动 commit。 |
| 权限与 diff         | permission 在每次写前 Request；diff 包提供 GenerateDiff、TextToPatch、PatchToCommit、ApplyCommit、IdentifyFiles*。 |

以上即「能写代码」在 OpenCode 代码库中的详细实现路径；核心就是 **Agent 循环 + Provider 流式与 Tool Use + edit/patch/write 三个工具的实现与权限/diff/history/LSP 的配合**。
