# 摘要、记忆与技能实现深度解析

## 1. SummarizationMiddleware

**位置**：`libs/deepagents/deepagents/middleware/summarization.py`

继承 LangChain 的 `BaseSummarizationMiddleware`，增加 **历史持久化** 和 **参数截断**。

### 1.1 触发与保留策略

- **trigger**：`("fraction", 0.85)` 或 `("tokens", N)` 或 `("messages", N)`
- **keep**：保留最近若干条消息或 token 比例
- graph.py 中：有 `max_input_tokens` 时用 `("fraction", 0.85)` / `("fraction", 0.10)`，否则 `("tokens", 170000)` / `("messages", 6)`

### 1.2 历史持久化 `_offload_to_backend`

摘要前先将待 evict 的消息写入 backend：

- 路径：`{history_path_prefix}/{thread_id}.md`（默认 `/conversation_history/{thread_id}.md`）
- 格式：每次摘要追加 `## Summarized at {timestamp}\n\n{get_buffer_string(filtered_messages)}\n\n`
- 使用 `download_files` / `edit` 或 `write` 实现 append（不用 `read`，因 read 返回 line-numbered 格式）

### 1.3 过滤摘要消息

`_filter_summary_messages` 去掉之前的摘要 HumanMessage（`lc_source='summarization'`），避免重复持久化。

### 1.4 摘要消息格式

若 offload 成功：

```markdown
You are in the middle of a conversation that has been summarized.
The full conversation history has been saved to {file_path} should you need to refer back to it for details.
A condensed summary follows:
<summary>{summary}</summary>
```

### 1.5 参数截断 `truncate_args_settings`

- **trigger**：何时截断（messages/tokens/fraction）
- **keep**：哪些消息不截断
- **max_length**：单参数超过多少字符截断（默认 2000）
- **truncation_text**：替换为 `"...(argument truncated)"`

只对 `write_file`、`edit_file` 的 tool_call 的 `args` 做截断，减少旧消息中的大块内容占用 token。

---

## 2. MemoryMiddleware

**位置**：`libs/deepagents/deepagents/middleware/memory.py`

### 2.1 加载时机

`before_agent` / `abefore_agent`：仅在 `memory_contents` 不在 state 时加载，之后复用。

### 2.2 数据源

- `sources`：路径列表，如 `["/memory/AGENTS.md"]`
- 通过 `backend.download_files([path])` 加载
- `file_not_found` 时静默跳过；其他错误抛出

### 2.3 注入 system prompt

`wrap_model_call` 中调用 `modify_request`，将 `_format_agent_memory(contents)` 追加到 system message：

```python
sections = [f"{path}\n{contents[path]}" for path in sources if contents.get(path)]
memory_body = "\n\n".join(sections)
# 套入 MEMORY_SYSTEM_PROMPT 模板
```

### 2.4 MEMORY_SYSTEM_PROMPT 内容

- 包裹在 `<agent_memory>...</agent_memory>`
- 含 `<memory_guidelines>`：何时更新记忆、何时不更新、示例
- 强调：记忆更新应优先于其他操作；不要存储 API key 等凭证

---

## 3. SkillsMiddleware

**位置**：`libs/deepagents/deepagents/middleware/skills.py`

### 3.1 SKILL.md 结构

```markdown
---
name: blog-post
description: Use this skill when writing long-form blog posts...
license: MIT
allowed-tools: tool1 tool2
---

# Blog Post Writing Skill
...
```

- 使用 YAML frontmatter（`---` 包裹）
- `name` 必须与目录名一致
- `description` 最大 1024 字符
- `allowed-tools`：空格分隔的工具名（实验性）

### 3.2 加载流程 `_list_skills`

1. `backend.ls_info(source_path)` 获取子目录
2. 对每个子目录检查 `{dir}/SKILL.md`
3. `backend.download_files(paths)` 批量下载
4. `_parse_skill_metadata` 解析 frontmatter，校验 name/description
5. 多 source 时，同名 skill 后者覆盖前者（`all_skills[skill["name"]] = skill`）

### 3.3 注入 system prompt

`SKILLS_SYSTEM_PROMPT` 模板包含：

- `{skills_locations}`：各 source 路径
- `{skills_list}`：`- **name**: description` + `-> Read \`path\` for full instructions`

**Progressive Disclosure**：只注入 name + description + path，不注入全文。模型需要时用 `read_file` 读取 SKILL.md。

### 3.4 校验与安全

- `_validate_skill_name`：小写字母数字 + 单连字符，与目录名一致
- `MAX_SKILL_FILE_SIZE = 10MB` 限制单文件大小
- `MAX_SKILL_DESCRIPTION_LENGTH = 1024`

---

## 4. 与 OpenCode 对照

| 功能 | Deep Agents | OpenCode SDK |
|------|-------------|--------------|
| 记忆 | AGENTS.md + MemoryMiddleware | Session context / 持久化 |
| 技能 | SKILL.md + SkillsMiddleware | Agent Skill / SKILL.md |
| 摘要 | SummarizationMiddleware + backend offload | 待设计 |
| 历史持久化 | `/conversation_history/{thread_id}.md` | 待设计 |
