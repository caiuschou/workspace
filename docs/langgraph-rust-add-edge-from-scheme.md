# langgraph-rust 边支持 from 方案

## 1. 背景与目标

### 1.1 背景

- **Python LangGraph**：`add_edge(from, to)` 显式指定起点和终点，支持任意拓扑（分支、汇聚、多入口等）。
- **目标**：langgraph-rust 采用相同显式边模型。

### 1.2 目标

- 在 langgraph-rust 中支持 `add_edge(from, to)` 的显式边定义。
- 与 Python LangGraph 的边 API 对齐。
- 支持线性链、分支、汇聚等图结构。

---

## 2. Python LangGraph 对照

### 2.1 add_edge 签名

```python
def add_edge(self, start_key: str | list[str], end_key: str) -> Self:
```

- `start_key`：起点节点 id，或节点 id 列表（多源汇聚到同一目标）。**阶段一 Rust 仅支持单一起点**，多源边后续扩展。
- `end_key`：终点节点 id。
- 特殊常量：`START`（`"__start__"`）、`END`（`"__end__"`）。

### 2.2 内部表示

- `edges: set[(start, end)]` 存储 (from, to) 对。
- 入口：存在 `(START, node_id)` 的边。
- 出口：存在 `(node_id, END)` 的边。

### 2.3 便捷方法

- `set_entry_point(key)`：等价于 `add_edge(START, key)`。
- `set_finish_point(key)`：等价于 `add_edge(key, END)`。
- `add_sequence(nodes)`：Python 会同时添加节点并建链；Rust 版本见 4.2，仅连接**已存在**节点。

---

## 3. 数据结构

### 3.1 StateGraph

```rust
/// Special node IDs for graph boundaries. Align with Python START/END.
pub const START: &str = "__start__";
pub const END: &str = "__end__";

pub struct StateGraph<S> {
    nodes: HashMap<String, Box<dyn Node<S>>>,
    /// Explicit edges: (from, to). Supports START/END as virtual nodes.
    edges: Vec<(String, String)>,
    store: Option<Arc<dyn Store>>,
}
```

### 3.2 CompiledStateGraph

```rust
pub struct CompiledStateGraph<S> {
    pub(super) nodes: HashMap<String, Box<dyn Node<S>>>,
    /// Adjacency: from_id -> [to_id, ...]. Built from edges at compile time.
    pub(super) outgoing: HashMap<String, Vec<String>>,
    pub(super) checkpointer: Option<Arc<dyn Checkpointer<S>>>,
    pub(super) store: Option<Arc<dyn Store>>,
}
```

---

## 4. API 设计

### 4.1 新增 add_edge(from, to)

```rust
/// Adds a directed edge from `from_id` to `to_id`.
///
/// Use `START` and `END` for graph boundaries. Example:
/// ```ignore
/// graph.add_edge(START, "think")
///      .add_edge("think", "act")
///      .add_edge("act", "observe")
///      .add_edge("observe", END);
/// ```
pub fn add_edge(
    &mut self,
    from_id: impl Into<String>,
    to_id: impl Into<String>,
) -> &mut Self {
    let from_id = from_id.into();
    let to_id = to_id.into();
    // Validate: align with Python ValueError semantics
    assert!(from_id != END, "END cannot be used as edge source");
    assert!(to_id != START, "START cannot be used as edge target");
    self.edges.push((from_id, to_id));
    self
}
```

### 4.2 便捷方法

```rust
/// Equivalent to add_edge(START, key). Aligns with Python set_entry_point.
pub fn set_entry_point(&mut self, key: impl Into<String>) -> &mut Self {
    self.add_edge(START, key)
}

/// Equivalent to add_edge(key, END). Aligns with Python set_finish_point.
pub fn set_finish_point(&mut self, key: impl Into<String>) -> &mut Self {
    self.add_edge(key, END)
}

/// Adds a linear chain: START -> nodes[0] -> ... -> nodes[n] -> END.
/// Connects **existing** nodes only (nodes must be added via add_node first).
/// Semantic difference: Python add_sequence adds nodes and edges in one call.
pub fn add_sequence(&mut self, node_ids: &[impl AsRef<str>]) -> &mut Self {
    if node_ids.is_empty() {
        return self;
    }
    self.add_edge(START, node_ids[0].as_ref());
    for i in 0..node_ids.len() - 1 {
        self.add_edge(node_ids[i].as_ref(), node_ids[i + 1].as_ref());
    }
    self.add_edge(node_ids[node_ids.len() - 1].as_ref(), END);
    self
}
```

---

## 5. invoke 逻辑（基于 outgoing）

1. **入口**：收集所有 `outgoing[START]` 的目标；若多个，按 Python 语义可并行（第一阶段可限制为单入口）。
2. **Next::Continue**：查 `outgoing[current_id]`：
   - 若为空：等价于 End（无出边即结束）。
   - 若恰好一个目标且为 END：停止并返回 state。
   - 若恰好一个目标且为节点：执行该节点。
   - 若多个目标：第一阶段报错或取 `outgoing` 中第一个目标（顺序与 `edges` 插入顺序一致）；后续可扩展并行。
3. **Next::Node(id)**：跳转到 id（与现有一致）。

### 5.1 编译时验证

- 所有边的 `from`、`to` 除 START/END 外必须已 `add_node`（与 Python `ValueError` 一致）。
- 至少存在一条 `(START, x)` 边作为入口。
- 建议：若存在 `(START, a)` 和 `(START, b)` 多入口，编译时可接受，invoke 阶段先只支持单入口并返回明确错误。
- **边去重**：构建 `outgoing` 时对同一 `(from, to)` 去重，与 Python `set[(start, end)]` 语义一致；`edges` 仍用 `Vec` 保留添加顺序，用于多出边时「第一个」的确定。

---

## 6. 实现步骤

| 序号 | 任务 | 交付物 | 依赖 |
|------|------|--------|------|
| 6.1 | 定义 START/END 常量 | `graph/constants.rs` 或 `state_graph.rs` 顶部 | 无 |
| 6.2 | StateGraph 使用 edges 存储边 | `state_graph.rs` | 6.1 |
| 6.3 | 实现 add_edge(from, to) | `state_graph.rs` | 6.2 |
| 6.4 | 实现 set_entry_point、set_finish_point、add_sequence | `state_graph.rs` | 6.3 |
| 6.5 | compile 中构建 outgoing，校验边的节点存在 | `state_graph.rs`、`compiled.rs` | 6.3 |
| 6.6 | CompiledStateGraph 使用 outgoing，重写 invoke 入口与 Next::Continue | `compiled.rs` | 6.5 |
| 6.7 | 更新 examples 和 tests 使用新 API | `langgraph/tests/`、`langgraph-examples/examples/`、`README.md` | 6.4 |
| 6.8 | 更新 langgraph-rust-alignment.md | 本仓库 `docs/` | 6.7 |

---

## 7. 使用示例

### 7.1 线性链

```rust
graph.add_sequence(&["think", "act", "observe"]);
// 或显式：
graph.add_edge(START, "think")
     .add_edge("think", "act")
     .add_edge("act", "observe")
     .add_edge("observe", END);
```

### 7.2 ReAct 等分支图

```rust
graph.add_edge(START, "agent");
graph.add_edge("tools", "agent");
// 条件边由 Node::run 返回 Next::Node(id) 实现
```

---

## 8. 测试与验证

| 测试项 | 说明 |
|--------|------|
| 线性链 | add_sequence / add_edge(START→…→END) |
| 单节点 | add_edge(START, "echo").add_edge("echo", END) |
| 条件边 | observe → Next::Node("think") 循环，与现有 ReAct 一致 |
| 多入口 | add_edge(START, "a").add_edge(START, "b")，编译通过，invoke 语义可后续定义 |
| 单节点多出边 | add_edge("a","b").add_edge("a","c")，第一阶段取第一个或报错，验证顺序/错误信息 |
| 无效边 | from=END 或 to=START，add_edge 或 compile 时报错 |

---

## 9. 源码修改清单

> 源码位置：`thirdparty/langgraph-rust/langgraph/`（不提交，见 .gitignore）

### 9.1 需修改的文件

| 文件路径 | 修改内容 |
|----------|----------|
| `src/graph/state_graph.rs` | ① 移除 `edge_order`，新增 `edges: Vec<(String, String)>`；② 新增 `add_edge(from, to)` 替换原 `add_edge(to)`；③ 新增 `set_entry_point`、`set_finish_point`、`add_sequence`；④ `compile` 遍历 edges 校验节点存在、构建 outgoing 传给 CompiledStateGraph；⑤ 定义 `START`/`END` 常量（或见下 constants） |
| `src/graph/compiled.rs` | ① 将 `edge_order` 替换为 `outgoing: HashMap<String, Vec<String>>`；② 重写 `invoke`：入口从 `outgoing[START]` 取，`Next::Continue` 从 `outgoing[current_id]` 取，支持 END 虚拟节点 |
| `src/graph/compile_error.rs` | ① 更新注释（`edge_order` → edges）；② 新增 `NoEntryPoint`（无 `(START, x)` 边时 compile 报错），与 invoke 的 "empty graph" 语义对齐 |
| `src/graph/mod.rs` | ① 导出 `START`、`END`（若放在 constants.rs 则 `pub use constants::{START, END}`） |
| `src/lib.rs` | ① 在 `pub use graph::...` 中增加 `START`、`END` 导出 |

### 9.2 可选新建文件

| 文件路径 | 说明 |
|----------|------|
| `src/graph/constants.rs` | 存放 `pub const START`、`pub const END`；在 `mod.rs` 中 `mod constants; pub use constants::{START, END};` |

### 9.3 需更新的测试与示例

| 路径 | 文件 | 修改说明 |
|------|------|----------|
| `langgraph/tests/` | `state_graph.rs` | 4 处：`add_edge("echo")` → `add_edge(START,"echo").add_edge("echo",END)` 或 `add_sequence(&["echo"])` |
| `langgraph/tests/` | `react_linear_chain.rs` | 2 处：`add_edge("think").add_edge("act").add_edge("observe")` → `add_sequence(&["think","act","observe"])` |
| `langgraph-examples/examples/` | `state_graph_echo.rs` | 1 处 |
| `langgraph-examples/examples/` | `react_linear.rs` | 1 处 |
| `langgraph-examples/examples/` | `react_zhipu.rs` | 1 处 |
| `langgraph-examples/examples/` | `react_mcp.rs` | 1 处 |
| `langgraph-examples/examples/` | `react_mcp_gitlab.rs` | 1 处 |
| `langgraph-examples/examples/` | `react_exa.rs` | 1 处 |
| `langgraph-examples/examples/` | `memory_checkpoint.rs` | 1 处 |
| `langgraph-examples/examples/` | `memory_persistence.rs` | 1 处 |
| `README.md`（仓库根） | — | 代码示例中的 `add_edge` 调用 |

### 9.4 无需修改的模块

- `src/graph/next.rs` — Next 枚举不变
- `src/graph/node.rs` — Node trait 不变
- `src/react/*`、`src/llm/*`、`src/memory/*` 等 — 与边定义无关

### 9.5 实现步骤与文件对应

| 步骤 | 涉及文件 |
|------|----------|
| 6.1 | `state_graph.rs` 顶部 或 `constants.rs` + `mod.rs` |
| 6.2 | `state_graph.rs` |
| 6.3 | `state_graph.rs` |
| 6.4 | `state_graph.rs` |
| 6.5 | `state_graph.rs`（compile 逻辑）、`compiled.rs`（接收 outgoing） |
| 6.6 | `compiled.rs` |
| 6.7 | `langgraph/tests/*.rs`、`langgraph-examples/examples/*.rs`、`README.md` |
| 6.8 | `docs/langgraph-rust-alignment.md`（本仓库 docs/） |

---

## 10. 相关文档

- [langgraph-rust-alignment.md](langgraph-rust-alignment.md) — 与 Python 对齐说明
- [agora/docs/langgraph-agent-messaging-design.md](../agora/docs/langgraph-agent-messaging-design.md) — 消息能力设计
- thirdparty/langgraph-rust — 实现源码（不提交）

---

## 11. 更新记录

| 日期 | 更新内容 |
|------|----------|
| 2025-01-31 | 初始方案：数据结构、API、invoke 逻辑、迁移指南、任务表 |
| 2025-01-31 | 评审后更新：add_edge 校验、add_sequence 语义说明、边去重策略、多出边顺序约定、测试用例补充 |
| 2025-01-31 | 移除向后兼容与迁移指南，精简为纯新 API 方案 |
| 2025-01-31 | 执行修改：thirdparty/langgraph-rust 实现完成，tests/examples 已更新 |