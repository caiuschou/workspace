# MCP Resource 与 rust-langgraph Tool 方案整合

本文给出在现有 **ToolSource（tools）** 基础上，如何整合 MCP **resource** 的几种方案与推荐路径。

---

## 1. 现状与概念区分

### 1.1 当前 rust-langgraph tool 方案

- **ToolSource**：`list_tools()` → `Vec<ToolSpec>`，`call_tool(name, args)` → `ToolCallContent`
- **Think 节点**：用 `state.messages` 调 LLM，得到 `content` + `tool_calls`
- **Act 节点**：对每个 `tool_call` 调 `ToolSource::call_tool(name, arguments)`，结果写入 `tool_results`
- **Observe 节点**：把 `tool_results` 转成 User 消息追加到 `messages`，清空 `tool_calls`/`tool_results`

工具是「可调用的能力」：LLM 通过 tool_calls 主动发起，带 name + arguments，返回结果。

### 1.2 MCP Resource 是什么

- **resources/list**：返回 `ResourceListResult { resources: Vec<ResourceDefinition> }`
- **ResourceDefinition**：`uri`、`name?`、`mimeType?`、`description?`
- **resources/read**：请求 `uri`，返回 `ReadResourceResult { contents: Vec<ResourceContents> }`（Text/Blob）

Resource 是**只读数据**：由 Server 暴露的「可读上下文」，不是「可执行的工具」。语义上更适合作为**注入到对话里的上下文**，而不是再包装成一次 tool call。

---

## 2. 整合方案对比

### 方案 A：Resource 作为「上下文注入」——推荐

**思路**：Resource 不进入 ToolSource，而是单独的「上下文来源」；在 Think 前（或 Think 内）拉取并注入到 `messages`。

| 要素 | 设计 |
|------|------|
| 抽象 | 新增 **ResourceSource**（或 **ContextSource**）：`list_resources()`、`read_resource(uri)` |
| 注入时机 | 在 Think 之前：例如新增「Context 节点」，或 Think 节点内部先拉 resource 再调 LLM |
| 注入形式 | 把 `read_resource(uri)` 得到的文本（或摘要）作为 **System 或 User 消息** 追加到 `state.messages` |
| ToolSource | 不变，仍只负责 tools/list、tools/call |

**优点**：  
- 与 MCP 语义一致：resource 是只读数据，用于丰富上下文。  
- 边界清晰：ToolSource = 工具调用，ResourceSource = 上下文注入。  
- Think 看到的 messages 里已经包含 resource 内容，无需 LLM 再「调工具读 resource」。

**实现要点**：  
- 定义 `ResourceSource` trait（或复用/扩展现有 MCP client 的 list/read 接口）。  
- 在 ReAct 中增加「拉 resource 并注入」的入口：  
  - 要么在 Think 前加一个 Context 节点（先 `list_resources()`，再按策略选 uri 做 `read_resource(uri)`，结果拼进 messages）；  
  - 要么在 ThinkNode 构造时传入 `Option<ResourceSource>`，在 `run` 里先注入再 `llm.invoke`。  
- mcp-client 若尚无 `read_resource(uri)`，需在 client 层增加 `resources/read` 请求与结果解析。

---

### 方案 B：Resource 暴露为「只读工具」

**思路**：把 MCP resource 暴露成「伪工具」，LLM 通过 tool call 来「读」某个 resource。

| 要素 | 设计 |
|------|------|
| 工具形态 | 方式 1：一个工具 `read_resource`，参数为 `uri`（或 schema 里描述可选 uri 列表）。  
方式 2：每个 resource 一个工具，如 `resource_<name>`，参数为空，内部对应用固定 uri。 |
| ToolSource | McpToolSource 的 `list_tools()` 同时拉 `resources/list`，生成上述 ToolSpec；`call_tool` 时若匹配 read_resource 或 resource_xxx，则发 `resources/read`，把内容填进 `ToolCallContent::text`。 |

**优点**：  
- 不增加新抽象，只扩展现有 ToolSource。  
- LLM 按需「调工具」读 resource，实现简单。

**缺点**：  
- Resource 的语义是「可读数据」，用「工具调用」来读，容易让 prompt 里工具列表膨胀（很多 resource 就很多工具）。  
- 每次读 resource 都走一轮 tool_call + observe，轮次多、成本高。

**适用**：resource 数量少、且希望「完全用现有 ReAct + ToolSource 不改」时可选。

---

### 方案 C：双源抽象（ToolSource + ResourceSource）统一入口

**思路**：在 ReAct 层同时支持「工具」与「资源」，但二者在类型和用法上区分开。

| 要素 | 设计 |
|------|------|
| 抽象 | 保留 ToolSource；新增 ResourceSource（`list_resources`、`read_resource(uri)`）。 |
| 使用 | Think 前（或 Think 内）：从 ResourceSource 取 list，按策略（如全部、或按 name/uri 过滤）读部分 resource，将内容注入 messages；Think 仍只用 ToolSource 的 list_tools 拼工具描述。Act 只做 call_tool。 |
| 可选扩展 | 若希望「一个 MCP 连接同时提供 tools + resources」，可引入 **McpContext**：内部持有一个 MCP client，实现 ToolSource（tools/list、tools/call）和 ResourceSource（resources/list、resources/read），ReAct 同时持有 ToolSource 与 ResourceSource，可指向同一 McpContext。 |

**优点**：  
- 语义清晰，且和方案 A 一致；多了一个「统一 MCP 入口」的便利实现（McpContext）。  
- 后续加 prompts、list_changed 等也可挂在同一 McpContext 上。

**实现要点**：  
- 与方案 A 相同：ResourceSource trait + 注入时机（Context 节点或 Think 内）。  
- 可选：McpContext 实现 `ToolSource + ResourceSource`，ReAct 用同一个 McpContext 作两种 Source。

---

## 3. 推荐路径

**推荐采用方案 A（或方案 C，本质是 A + 统一 MCP 入口）**：

1. **保持 ToolSource 只负责 tools**  
   - 不把 resource 变成「工具」，避免语义混淆和轮次膨胀。

2. **新增 ResourceSource（或 ContextSource）**  
   - `list_resources() -> Vec<ResourceDefinition>`（或简化为 uri + name/description）  
   - `read_resource(uri) -> String`（或 `ReadResourceResult`，先只处理 text 类型）

3. **在 ReAct 中增加「resource 上下文注入」**  
   - 方式 1：在 Think 前加 **Context 节点**：调用 `list_resources()`，再按策略（例如全部、或前 N 个、或按 name 过滤）对部分 uri 调用 `read_resource(uri)`，将返回文本作为 User（或 System）消息追加到 `state.messages`，再进入 Think。  
   - 方式 2：ThinkNode 接受 `Option<Arc<dyn ResourceSource>>`，在 `run` 里先注入再 `llm.invoke`。  
   - 策略（读哪些、读多少）可先写死（例如全读），后续再做成配置或策略接口。

4. **mcp-client 补齐 resources/read**  
   - 若当前只有 `resources/list`，需在 mcp-client 增加 `read_resource(uri)`（发 `resources/read`，解析 `ReadResourceResult`，取 text 内容返回），供 ResourceSource 实现使用。

5. **可选：McpToolSource + McpResourceSource 共用同一 Client**  
   - 一个 MCP 连接同时实现 ToolSource 与 ResourceSource，ReAct 侧持有一个「MCP 连接」即可同时拿 tools 和 resources，对应方案 C 的「统一入口」。

---

## 4. 任务表（按方案 A/C 落地）

| 项 | 状态 | 说明 |
|----|------|------|
| ResourceSource trait 定义 | 待办 | `list_resources()`、`read_resource(uri)`，放在 `tool_source/` 或新 `resource_source/` |
| mcp-client 增加 read_resource(uri) | 待办 | 发 resources/read，解析 ReadResourceResult，返回文本或结构化内容 |
| McpResourceSource 实现 | 待办 | 依赖 mcp-client 的 list_resources + read_resource |
| ReAct 注入点：Context 节点或 Think 内 | 待办 | 在 Think 前拉 resource 并拼进 state.messages |
| 策略：读哪些 resource | 待办 | 先全读或按 name/uri 过滤，后可配置 |
| 可选：McpContext 同时实现 ToolSource + ResourceSource | 待办 | 同一连接供 tools + resources |

---

## 5. 小结

- **Tool**：保留现有 ToolSource，只负责「可调用工具」的 list/call。  
- **Resource**：通过 **ResourceSource** 做「上下文注入」，在 Think 前（或 Think 内）拉取并写入 messages，不变成 tool。  
- **推荐**：方案 A（仅 ResourceSource + 注入）；若希望一个 MCP 连接同时提供 tools 与 resources，用方案 C（同一 McpContext 实现双 Source）。  
- **不推荐**：把每个 resource 都变成工具（方案 B），除非 resource 极少且坚持零新抽象。

每项任务完成后可在本表「状态」列改为「已完成」，并在代码处补充注释引用本文（如 `mcp-integration/resource-integration.md`）。
