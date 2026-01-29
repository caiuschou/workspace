# rust-langgraph

LangGraph 的 Rust 原生实现，采用类型安全的状态机、trait 多态和 actor 模式。

## 包布局

所有实现均放在本目录下，根目录为 `rust-langgraph/`，各 crate 位于 `crates/` 下：

```
rust-langgraph/
├── README.md           # 本文件
├── Cargo.toml          # 本包内 workspace（可选，便于在本目录独立构建）
├── crates/
│   ├── langgraph/      # 核心库：trait、Agent、状态机、工具、记忆等
│   ├── langgraph-react/    # ReAct Agent：Think → Act → Observe 循环（依赖 langgraph）
│   ├── langgraph-openai/   # （可选）OpenAI LLM/Embedder 实现
│   └── ...             # 其他实现包按需在此追加
├── examples/           # （可选）顶层示例，或放在各 crate 的 examples/
└── docs/               # （可选）本包内文档
```

- **根 workspace**：本仓库根目录的 `Cargo.toml` 通过 members 引入 `rust-langgraph/crates/langgraph` 等，统一构建与依赖。
- **本目录**：作为「rust-langgraph 包」的根，所有 LangGraph 相关实现都放在 `rust-langgraph/` 下，与 `crates/`、`examples/`、`mcp-impls/` 等平级。

开发计划与 Sprint 划分见 [docs/rust-langgraph/ROADMAP.md](../docs/rust-langgraph/ROADMAP.md)。
