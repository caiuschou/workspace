# 数据流（Chat 模型）

1. 用户调用：`invoke(input)` 或 `generate([messages])`。
2. 输入经 `_convert_input` 转为 `PromptValue`（如 `ChatPromptValue(messages)`），再进入 `generate_prompt` / `generate`。
3. 内部：缓存（若开启）、rate_limiter，再根据是否流式：
   - 流式：走 `_stream` / `_astream`，产出 `ChatGenerationChunk`，再合并为 `ChatResult`；
   - 非流式：直接 `_generate` / `_agenerate` → `ChatResult`。
4. `ChatResult` 含 `generations: list[ChatGeneration]`，每个 `ChatGeneration` 含 `message: AIMessage` 与可选 `generation_info`。
5. 对外返回 `AIMessage`（或流式时的 `AIMessageChunk`）。
