# 上下文压缩：以 Claude Code Auto-Compact 为例

本文说明「上下文压缩」在长对话中的用法，以 **Claude Code 的 Auto-Compact** 为对照，并对应到本系列中 LangGraph 记忆与摘要压缩的设计。

---

## Claude Code 的上下文压缩（Auto-Compact）

Claude Code 通过 **Auto-Compact** 在接近上下文窗口上限时，用「摘要」替代「整段旧消息」，使长会话可持续进行。

### 工作机制

当对话接近上下文上限时，会：

1. **分析对话**：识别值得保留的信息（代码变更、架构决策、当前任务等）。
2. **生成摘要**：用模型把此前多轮交互压缩成一段「此前对话摘要」。
3. **替换历史**：用该摘要替换原有旧消息，只保留「摘要 + 最近若干条」。
4. **继续对话**：后续请求只带「摘要 + 最近消息」，会话不中断。

自 v2.0.64 起，压缩为即时完成，无需等待。

### 三种使用方式

| 方式 | 说明 |
|------|------|
| **Auto-Compact（默认）** | 快满时自动完成上述分析→摘要→替换，无需操作。 |
| **Manual Compact（`/compact`）** | 用户手动触发，并可指定保留内容，例如：<br>`/compact preserve current architecture decisions`<br>`/compact keep the solution we found, remove debugging steps` |
| **Clear（`/clear`）** | 清空会话，不做保留或摘要。 |

### 保留 vs 被压缩的内容

- **倾向保留**：最近代码改动、项目结构、重要架构决定、当前任务目标、命名与编码习惯、关键配置。
- **倾向压成摘要或去掉**：已不相关的冗长解释、已解决的调试过程、未形成代码的探索性讨论、对当前任务无用的历史背景。

---

## 与本系列文档的对应关系

[09-memory-chat-case.md](09-memory-chat-case.md) 中的「摘要压缩」描述为：

> 超出窗口的旧对话用 LLM 生成一段「此前对话摘要」，以后每轮只发「摘要 + 最近 N 条」给模型。

与 Claude Code 的 Auto-Compact 属同一思路：**旧消息 → 摘要；每轮仅发送「摘要 + 最近 N 条」**。

差异主要在实现层级：

- **Claude Code**：在产品/会话层完成，用户通过 `/compact`、自动压缩、`/config` 查看上下文用量。
- **本系列设计**：在应用/图层实现，例如在 `build_context` 或调用 LLM 前，从 checkpoint 取消息、必要时做摘要，再拼成「摘要 + 最近 N 条」作为当次请求的 `messages`。

[assistant/memory.md](../assistant/memory.md) 中的 `get_compressed_context`（`context_summary` + 最近一半消息）和 `ContextCompressor`（用 LLM 把一段对话压成简短说明），即「摘要 + 最近 N 条」的一种实现形态。

---

## 在 LangGraph 记忆设计中的对应

结合 [10-memory-deep-dive.md](10-memory-deep-dive.md) 的术语：

- **会话内记忆**仍由 **thread + Checkpoint + `messages` channel** 负责；上下文压缩不替代 Checkpoint，而是**在调用 LLM 的那一步**对「从 state/checkpoint 取出的 `messages`」做裁剪与摘要。
- **实现要点**（与 Claude Code 思路一致）：
  - 设定「发给模型的 message 数量/长度上限」。
  - 若当前 `messages` 超出：用 LLM 将超出部分（或全部旧消息）压成「此前对话摘要」，再构造「系统/上下文中带摘要 + 最近 N 条 messages」送入模型。
  - 摘要可仅存在于当次请求的 prompt 中，不必写回 checkpoint；checkpoint 中仍可保留更长的原始 `messages`，供下次摘要时使用（与 09 中「`messages` 在 Checkpoint 里仍可多留一些用于摘要生成」一致）。

若要支持类似 `/compact` 的手动控制，可在 state 中增加 channel（如 `context_summary: str`），在某一节点或人审节点中调用摘要逻辑，将当前 `messages` 压成摘要并写入 `context_summary`，并可选择清空或裁剪 `messages`；此后 `build_context` 使用「`context_summary` + 最近 N 条」作为送入 LLM 的上下文。

---

## 小结

| 主题 | 要点 |
|------|------|
| **Claude Code Auto-Compact** | 快满时自动分析→摘要→替换旧消息，保留「摘要 + 最近消息」；支持 `/compact` 手动与 `/clear` 清空。 |
| **与本系列「摘要压缩」** | 同一思路：旧消息→摘要，每轮只发「摘要 + 最近 N 条」；本系列在应用层、`build_context` / ContextCompressor 中实现。 |
| **在 LangGraph 中的位置** | 在「调用 LLM 的那一步」对 `messages` 做裁剪与摘要；可选 `context_summary` channel 实现手动压缩控制。 |

---

## 参考资料

- [What is Claude Code Auto-Compact](https://claudelog.com/faqs/what-is-claude-code-auto-compact/)
- [Anthropic: Automatic context compaction](https://platform.claude.com/cookbook/tool-use-automatic-context-compaction)（API 层 `compaction_control` 参数）
