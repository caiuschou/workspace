# 中间件深度解析

基于 `libs/deepagents/deepagents/middleware/` 源码的深度分析。

## 1. AgentMiddleware 生命周期钩子

| 钩子 | 时机 | 典型用途 |
|------|------|----------|
| before_agent | Agent 启动前 | 加载 Memory、Skills 到 state |
| wrap_model_call | 每次调用 LLM 前 | 注入 system prompt、过滤工具 |
| wrap_tool_call | 每次执行工具后 | 大结果 eviction |
| before_model | 每次调用 LLM 前（BaseSummarizationMiddleware） | 摘要、截断 |

## 2. FilesystemMiddleware 实现细节

### 2.1 工具与 Backend 绑定

每个工具通过 `runtime: ToolRuntime` 获取 backend：

```python
def _get_backend(self, runtime: ToolRuntime) -> BackendProtocol:
    if callable(self.backend):
        return self.backend(runtime)
    return self.backend
```

`backend` 可为实例或工厂 `Callable[[ToolRuntime], BackendProtocol]`。

### 2.2 路径校验 `_validate_path`

- 禁止：`..`、`~`、Windows 绝对路径 `C:\...`
- 规范化：`os.path.normpath` + 统一 `/`
- 可选：`allowed_prefixes` 限制访问范围

### 2.3 大结果 Eviction 流程

`wrap_tool_call` 拦截工具返回：

1. 若 `tool_token_limit_before_evict` 为 None 或工具在 `TOOLS_EXCLUDED_FROM_EVICTION` 中，直接返回
2. 否则调用 `_intercept_large_tool_result`
3. 若内容超过 `NUM_CHARS_PER_TOKEN * tool_token_limit_before_evict`（默认 4 * 20000）：
   - 写入 `/large_tool_results/{sanitized_tool_call_id}`
   - 替换为 `TOO_LARGE_TOOL_MSG` + head/tail 预览

**排除 Eviction 的工具**：`ls`, `glob`, `grep`, `read_file`, `edit_file`, `write_file`（自带 truncation 或结果小）。

### 2.4 write_file / edit_file 的 Command 返回

当 backend 返回 `files_update`（StateBackend）时，返回 `Command` 而非字符串：

```python
return Command(update={
    "files": res.files_update,
    "messages": [ToolMessage(content="...", tool_call_id=...)]
})
```

这样 LangGraph state 的 `files` 键能正确合并。

### 2.5 read_file 分页与 truncation

- 默认 `offset=0`, `limit=100`
- 超过 `NUM_CHARS_PER_TOKEN * token_limit` 时，在 read_file 内部 truncate 并追加 `READ_FILE_TRUNCATION_MSG`
- 长行：`format_content_with_line_numbers` 对超过 5000 字符的行拆分，用 `5.1`, `5.2` 等 continuation 标记

### 2.6 execute 工具动态过滤

`wrap_model_call` 中根据 backend 是否实现 `SandboxBackendProtocol` 决定是否暴露 `execute` 工具。若不支持，从 `request.tools` 中移除。

## 3. SubAgentMiddleware 实现细节

### 3.1 子 Agent 构建 `_get_subagents`

```python
subagent_graphs, subagent_descriptions = _get_subagents(
    default_model=default_model,
    default_tools=default_tools,
    default_middleware=default_subagent_middleware,
    ...
)
```

- `default_middleware` 先于每个 SubAgent 的 `middleware`
- `general_purpose_agent=True` 时，创建 `general-purpose` 子 Agent，使用 `DEFAULT_SUBAGENT_PROMPT` 和相同 tools

### 3.2 状态传递 `_EXCLUDED_STATE_KEYS`

传给子 Agent 时排除：`messages`, `todos`, `structured_response`。子 Agent 只接收 `HumanMessage(description)`，不继承主 Agent 的对话历史。

### 3.3 返回 `_return_command_with_state_update`

子 Agent 的 state 必须有 `messages`。取 `result["messages"][-1].text` 作为 `ToolMessage` 内容返回主 Agent。其他 state 键（除 `_EXCLUDED_STATE_KEYS`）会 merge 回主 Agent（若 reducer 支持）。

### 3.4 task 工具描述

`TASK_TOOL_DESCRIPTION` 包含 `{available_agents}` 占位符，运行时替换为各子 Agent 的 `name: description` 列表，供主 Agent 选择。

## 4. PatchToolCallsMiddleware

**位置**：`libs/deepagents/deepagents/middleware/patch_tool_calls.py`

在 `before_agent` 中扫描 `messages`：若 `AIMessage` 有 `tool_calls` 但后续无对应 `ToolMessage`，补上 "cancelled" 的 `ToolMessage`，避免 dangling tool call 导致模型困惑。

```python
if corresponding_tool_msg is None:
    tool_msg = (
        f"Tool call {tool_call['name']} with id {tool_call['id']} was "
        "cancelled - another message came in before it could be completed."
    )
    patched_messages.append(ToolMessage(content=tool_msg, ...))
```

## 5. _utils.append_to_system_message

```python
def append_to_system_message(system_message, text):
    new_content = list(system_message.content_blocks) if system_message else []
    if new_content:
        text = f"\n\n{text}"
    new_content.append({"type": "text", "text": text})
    return SystemMessage(content=new_content)
```

支持 `SystemMessage` 的 `content_blocks` 结构（含图片等），纯文本追加为 `{"type": "text", "text": ...}`。
