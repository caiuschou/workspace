# 设计决策与数据流

## 1. 核心设计决策

### 1.1 Backend 抽象

- **统一接口**：`BackendProtocol` 抽象 ls/read/write/edit/grep/glob/upload/download
- **StateBackend 与 State 同步**：返回 `Command(update={"files": files_update})` 让 LangGraph state 正确合并
- **工厂模式**：`lambda rt: StateBackend(rt)` 使 backend 能访问 runtime/state
- **CompositeBackend**：按路径前缀路由，支持混合存储（临时 + 持久）

### 1.2 中间件链顺序

1. TodoList → 任务列表最先
2. Memory → 记忆在模型调用前注入
3. Skills → 技能元数据在模型调用前注入
4. Filesystem → 文件工具与 execute
5. SubAgent → task 工具，子 Agent 可复用上述能力
6. Summarization → 长对话 eviction，在模型调用前（before_model）
7. AnthropicPromptCaching → 缓存优化
8. PatchToolCalls → 修补 dangling tool call，在 agent 启动前
9. HumanInTheLoop → 可选，按工具名暂停

### 1.3 大结果处理

- **工具结果**：超过 20k tokens 写入 `/large_tool_results/`，返回 head+tail 预览
- **read_file**：内部 truncate 并提示用 offset/limit 分页
- **摘要前历史**：写入 `/conversation_history/{thread_id}.md`，避免数据丢失

### 1.4 子 Agent 隔离

- 不传递主 Agent 的 `messages`、`todos`、`structured_response`
- 子 Agent 只收到 `HumanMessage(description)`，独立运行
- 返回时取最后一条 message 作为 ToolMessage，其余 state 可选 merge

---

## 2. 数据流

### 2.1 invoke 传入文件（StateBackend）

```text
invoke({"messages": [...], "files": {"/path": FileData}})
  → StateBackend.runtime.state["files"] 已有初始数据
  → FilesystemMiddleware 工具读写 state["files"]
  → write/edit 返回 Command(update={"files": files_update})
  → LangGraph state reducer 合并 files
```

### 2.2 摘要流程

```text
before_model(state, runtime)
  → token_counter(messages) 检查是否超过 trigger
  → _partition_messages 划分 messages_to_summarize / preserved
  → _offload_to_backend(messages_to_summarize) 写入 /conversation_history/{thread_id}.md
  → _create_summary(messages_to_summarize)
  → 返回 RemoveMessage(id=REMOVE_ALL) + new_summary_message + preserved_messages
```

### 2.3 task 工具流程

```text
主 Agent AIMessage(tool_calls=[{name: "task", args: {description, subagent_type}}])
  → SubAgentMiddleware.task(description, subagent_type, runtime)
  → _validate_and_prepare_state：subagent_state = {k: v for k, v in runtime.state if k not in _EXCLUDED}
  → subagent_state["messages"] = [HumanMessage(description)]
  → subagent.invoke(subagent_state)
  → _return_command_with_state_update：取 result["messages"][-1] 作为 ToolMessage 返回
```

---

## 3. 安全考量

| 方面 | 措施 |
|------|------|
| 路径遍历 | `_validate_path` 禁止 `..`、`~`、Windows 绝对路径 |
| 符号链接 | FilesystemBackend 使用 `O_NOFOLLOW` |
| virtual_mode | FilesystemBackend 可限制访问范围到 root_dir |
| 大文件 | SKILL.md 限制 10MB；grep 跳过超大文件 |
| Sandbox write/edit | BaseSandbox 用 base64+heredoc 避免 shell 注入 |

---

## 4. 可移植到 Rust 的要点

- **Backend 协议**：抽象 read/write/edit/ls/grep/glob，支持 State/Filesystem/Composite
- **中间件链**：before_agent / wrap_model_call / wrap_tool_call 的 hook 顺序
- **Command 返回**：工具可返回 state 更新而非仅字符串
- **SKILL.md**：YAML frontmatter + Progressive Disclosure
- **AGENTS.md**：Memory 加载与 memory_guidelines prompt
