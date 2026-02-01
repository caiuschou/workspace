# Deep Agents 架构

## 入口：create_deep_agent

**位置**：`libs/deepagents/deepagents/graph.py`

返回 `CompiledStateGraph`（LangGraph），可 streaming、checkpoint、Studio 等。

## 中间件链（Middleware Stack）

按顺序执行：

| 顺序 | 中间件 | 作用 |
|------|--------|------|
| 1 | TodoListMiddleware | 任务列表 `write_todos` / `read_todos` |
| 2 | MemoryMiddleware | 从 `AGENTS.md` 等加载长期记忆到 system prompt |
| 3 | SkillsMiddleware | 从 `skills/` 目录加载 SKILL.md 到 system prompt |
| 4 | FilesystemMiddleware | 文件工具 `ls`, `read_file`, `write_file`, `edit_file`, `glob`, `grep`, `execute` |
| 5 | SubAgentMiddleware | `task` 工具，调用子 Agent |
| 6 | SummarizationMiddleware | 长对话自动摘要（约 85% 窗口触发） |
| 7 | AnthropicPromptCachingMiddleware | Prompt 缓存 |
| 8 | PatchToolCallsMiddleware | 工具调用补丁 |
| 9 | HumanInTheLoopMiddleware | 可选，指定工具前暂停审批 |

## FilesystemMiddleware

**位置**：`libs/deepagents/deepagents/middleware/filesystem.py`

### 工具

| 工具 | 说明 |
|------|------|
| ls | 列出目录 |
| read_file | 读取文件（支持 offset/limit 分页） |
| write_file | 写入新文件 |
| edit_file | 精确字符串替换（old_string, new_string） |
| glob | 模式匹配 `**/*.py` |
| grep | 文本搜索（StateBackend/FilesystemBackend 用 regex；Sandbox 用 grep -F literal） |
| execute | 执行 shell 命令（需 SandboxBackendProtocol） |

### 设计要点

- **虚拟路径**：以 `/` 开头，禁止 `..`、`~`、Windows 绝对路径
- **大结果处理**：超过约 20k tokens 时写入 `/large_tool_results/` 并返回摘要
- **read_file 分页**：`offset`, `limit` 避免一次读入过大

## SubAgentMiddleware

**位置**：`libs/deepagents/deepagents/middleware/subagents.py`

### task 工具

主 Agent 通过 `task(description=..., subagent_type=...)` 调用子 Agent。

### 子 Agent 类型

- **SubAgent**：配置创建（name, description, system_prompt, tools, model）
- **CompiledSubAgent**：预编译的 Runnable（如自定义 LangGraph）

### 默认 general-purpose

默认包含一个 general-purpose 子 Agent，具备与主 Agent 相同的工具集，用于隔离上下文。

### 调用流程

```
主 Agent 调用 task(description, subagent_type)
  → 创建子 Agent 初始 state（messages: [HumanMessage(description)]）
  → 子 Agent 独立运行至完成
  → 取子 Agent 最后一条 messages 作为 ToolMessage 返回主 Agent
```

## SummarizationMiddleware

- 当对话接近上下文窗口上限时（如 85%）触发摘要
- 保留最近若干条消息
- 摘要结果作为新消息插入，替代历史长尾

## Skills 与 Memory

- **Skills**：`skills/` 下每个子目录可含 SKILL.md，描述该技能的用法，注入 system prompt
- **Memory**：`AGENTS.md` 等文件内容加载到 system prompt，作为长期记忆/风格指南
