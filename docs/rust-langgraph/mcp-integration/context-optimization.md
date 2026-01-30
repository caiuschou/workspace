# MCP 多工具场景下的上下文优化方案

当接入的 MCP 很多、每个 MCP 又暴露大量工具时，若把所有工具描述（schema）都塞进 LLM 的 prompt，会导致**上下文膨胀**（prompt bloat），进而影响工具选择准确率和成本。本文整理问题成因、业界最新研究及可选方案，供 Cursor/IDE 使用与 rust-langgraph MCP 客户端设计参考。

---

## 1. 问题与成因

| 来源 | 对上下文的影响 |
|------|----------------|
| **工具 schema** | 每个 MCP 的每个工具都有 name、description、inputSchema；全量放入 prompt 会随 MCP/工具数量线性增长。 |
| **按需 vs 全量** | 若客户端「按需」只在调用前读某工具 schema，则可见工具少；若「全量」把所有工具描述塞进 system/context，则上下文很大。 |
| **调用与返回** | 每次 `tools/call` 的参数和返回内容也会进入对话；工具多、调用多、返回大（如 list 出大量项）都会推高 token。 |

结论：**MCP 很多且全量把工具描述放进上下文时，上下文会明显变大**；需要在「工具发现」与「上下文占用」之间做权衡。

---

## 2. 业界最新研究概览

### 2.1 RAG-MCP：检索再给 LLM（直接针对 MCP）

- **论文**：*RAG-MCP: Mitigating Prompt Bloat in LLM Tool Selection via Retrieval-Augmented Generation*（arXiv:2505.03275, 2025）
- **做法**：不把所有 MCP 工具描述都塞进 prompt；先用**语义检索**从外部索引中选出与当前 query 最相关的 MCP/工具，只把**被选中工具**的 schema 传给 LLM。
- **效果**（文中报告）：
  - prompt token 减少 **50%+**；
  - 工具选择准确率从 **13.62% → 43.13%**（benchmark）；
  - 通过 MCP stress test 验证可扩展。
- **适用**：无需改模型，客户端/中间层实现「工具索引 + 检索 + 只传检索结果」即可，与现有 MCP 协议兼容。

### 2.2 ToolGen：工具当「虚拟 token」、不占上下文

- **论文**：*ToolGen: Unified Tool Retrieval and Calling via Generation*（ICLR 2025, arXiv:2410.03439）
- **做法**：每个工具映射为**唯一虚拟 token**，通过训练把工具信息压进模型参数；调用工具 = 模型按「下一个 token」生成工具 id + 参数，不再在上下文里塞完整工具列表。
- **效果**：在 **4.7 万+ 工具**规模下仍可检索与调用，无需额外检索步骤。
- **代价**：需要**专门训练/微调**，不适合「现成 API + 零训练」的 MCP 客户端直接套用；代表「工具极多」时的长期方向。

### 2.3 用 LLM 生成检索 query

- **思路**：不用用户原句或简单关键词直接做向量检索，而是先用 LLM 根据用户意图**生成一条专门用于检索的 query**，再对该 query 做 embedding 检索工具。
- **好处**：在复杂、多工具场景下，比单纯「用户句子的 embedding」或词频匹配更能对准「该用哪些工具」，从而用更少的工具描述覆盖当前任务。

### 2.4 检索 vs 长上下文

- 有研究显示：**4K 上下文 + 检索** 可达到接近 **16K 全上下文** 的效果且计算更省。
- 结论：**检索增强 + 适度长上下文** 组合优于单纯堆上下文；对应到 MCP即「按需检索工具描述 + 合理限制单次可见工具数」。

### 2.5 上下文 / Prompt 压缩

- **TCRA-LLM** 等：对已检索到的内容做摘要或语义压缩再塞进上下文（如 token 减 65%、语义压缩约 20%），在 RAG 场景里控制检索结果长度。
- **SARA** 等：选择性压缩，保留关键片段 + 压缩向量。
- 用在 MCP 上：可对**工具描述**或**工具返回结果**做摘要/压缩再给 LLM，而不是原样全部放进上下文。

---

## 3. 方案对比与推荐

| 方向 | 做法简述 | 是否要训练 | 与 MCP 的契合度 |
|------|-----------|------------|-----------------|
| **RAG-MCP** | 先检索相关 MCP/工具，只传少量描述 | 一般不需要 | 直接针对 MCP，易落地 |
| **ToolGen** | 工具进参数，当虚拟 token 生成 | 需要 | 工具极多时的长期方向 |
| **LLM 生成 query** | 用 LLM 生成检索 query 再检索工具 | 不需要 | 可与 RAG-MCP 组合 |
| **Prompt/上下文压缩** | 对描述或返回做摘要/压缩 | 视方法而定 | 辅助，进一步省 token |

**推荐**：当前与「很多 MCP」最贴合、且已有论文与实验支撑的，是 **RAG-MCP 这类「先检索、再只把相关工具描述塞进上下文」** 的方案；ToolGen 适合作为「工具规模极大且能接受训练」时的演进方向。

---

## 4. 对 Cursor / IDE 的实践建议

- **只启用当前任务需要的 MCP**，不用时在设置中关闭，可显著减小上下文。
- 若实现允许，优先采用**按需加载工具 schema**（或 RAG-MCP 式检索），避免「所有 MCP 的 schema 常驻上下文」。
- 对会返回大量数据的 MCP 工具（如大目录 list、大文件内容），在服务端或客户端做**分页、过滤或摘要**，控制单次返回体积。

---

## 5. 对 rust-langgraph MCP 客户端的可选扩展

在现有 [overview](overview.md) 与 [mcp-tool-devplan](mcp-tool-devplan.md) 基础上，若未来需要支持「多 MCP / 多工具」且控制上下文：

- **可选 A**：实现「工具索引 + 语义检索」：在 `list_tools()` 或等价层维护工具描述的向量索引，在拼 Think 节点 prompt 前先按当前 query 检索 top-k 工具，只把这 k 个工具的 schema 传给 LLM（RAG-MCP 思路）。
- **可选 B**：对 `ToolSource::list_tools()` 增加「按 query 过滤」接口，例如 `list_tools_for_query(query: &str, limit: usize)`，由 MCP 客户端内部做检索或关键词/标签过滤，再只返回少量工具描述。
- **可选 C**：对工具返回内容做长度/条数上限或摘要（例如只取前 N 条、或对长文本做 summarize），避免单次 tool 结果撑爆上下文。

以上为可选扩展，当前阶段仍以「单 MCP / 工具数可控」的最简集成为主；待确有「大量 MCP/工具」需求时再落地 A/B/C。

---

## 6. 任务与进度

| 任务 | 状态 | 说明 |
|------|------|------|
| 问题与成因整理 | 完成 | 见第 1 节 |
| 业界研究整理（RAG-MCP、ToolGen 等） | 完成 | 见第 2 节 |
| 方案对比与推荐 | 完成 | 见第 3 节 |
| Cursor/IDE 实践建议 | 完成 | 见第 4 节 |
| rust-langgraph 可选扩展设计 | 完成 | 见第 5 节 |
| （可选）实现工具索引 + 检索（RAG-MCP 风格） | 待规划 | 依赖「多 MCP/多工具」需求明确 |
| （可选）list_tools_for_query 或返回摘要 | 待规划 | 同上 |

---

## 7. 参考文献

- RAG-MCP: Mitigating Prompt Bloat in LLM Tool Selection via Retrieval-Augmented Generation. arXiv:2505.03275, 2025.
- ToolGen: Unified Tool Retrieval and Calling via Generation. ICLR 2025, arXiv:2410.03439.
- MCP 规范：<https://modelcontextprotocol.io/specification/2024-11-05>（及 2025-11-25 等更新版本）。
