# Reducer 设计

与 LangGraph 的 state reducer 对齐：节点可返回 **partial state**，由 **reducer** 将 update 与当前 state 合并。本文给出 Rust 侧的 reducer 抽象与 `add_messages` 设计。

**文档结构**：§1 背景与目标 → §2 Reducer 抽象 → §3 add_messages → §4 Partial State 与 merge → §5 使用方式与 Agent 组合 → §6 与最简 Agent 的关系 → §7 小结。

---

## 1. 背景与目标

### 1.1 LangGraph 侧语义

- **State schema**：每路 state 的每个 key 可带 **reducer**，签名为 `(left: T, right: T) -> T`。图运行时合并「当前 state」与「节点返回的 partial state」时，对每个 key 调用该 reducer。
- **add_messages**：`(left: list[Message], right: list[Message]) -> list[Message]`。按消息 **ID** 合并：若 `right` 中某条消息的 ID 在 `left` 中已存在则**覆盖**该位置，否则**追加**；未带 ID 的消息可分配新 ID（如 UUID）。
- **Partial state**：节点只返回要更新的键（如 `{"messages": [response]}`），未返回的 key 保持原样。

### 1.2 Rust 侧设计目标

- 定义 **Reducer** 抽象：对类型 `T` 的合并语义与 LangGraph 一致。
- 支持 **partial state**：节点可返回部分字段的更新，由运行时或 `State::merge` 按每字段的 reducer 合并。
- 提供 **add_messages** 的 Rust 实现：`Message` 带可选 `id`，合并规则与 Python 一致（同 ID 覆盖，否则追加）。

---

## 2. Reducer 抽象

### 2.1 泛型 Reducer（无状态）

合并语义由类型或函数表达，不依赖运行时状态：

```rust
/// Reducer: merges an update into the current value for a state field.
/// Aligns with LangGraph's per-key reducer (left, right) -> merged.
pub trait Reducer<T>: Send + Sync + 'static {
    /// Merges `update` into `current`, producing the new value for this field.
    fn merge(&self, current: T, update: T) -> T;
}
```

对「仅依赖 current/update 的合并」可用零大小类型包装函数：

```rust
/// Stateless reducer: holds a merge function.
pub struct ReducerFn<F>(F);

impl<T, F> Reducer<T> for ReducerFn<F>
where
    F: Fn(T, T) -> T + Send + Sync + 'static,
{
    fn merge(&self, current: T, update: T) -> T {
        (self.0)(current, update)
    }
}

// Example: replace (no merge, last wins)
pub static REPLACE: ReducerFn<fn(String, String) -> String> = ReducerFn(|_, update| update);
```

### 2.2 常用 Reducer 示例

| 语义       | 说明                     | Rust 示例                          |
|------------|--------------------------|------------------------------------|
| Replace    | 用 update 覆盖 current   | `ReducerFn(\|_, u\| u)`            |
| Append     | 列表拼接 current + update | `ReducerFn(\|mut c, u\| { c.extend(u); c })` |
| AddMessages | 按消息 ID：同 ID 覆盖，否则追加 | 见 §3                              |

---

## 3. add_messages：消息列表 Reducer

### 3.1 Message 与 id

与 LangGraph 对齐，支持「按 ID 覆盖」需要消息带可选 ID：

```rust
/// Message with optional id for reducer merge (replace-by-id, else append).
#[derive(Debug, Clone)]
pub struct Message {
    pub id: Option<String>,
    pub kind: MessageKind,
}

#[derive(Debug, Clone)]
pub enum MessageKind {
    System(String),
    User(String),
    Assistant(String),
}
```

为兼容最简设计，可保留现有 `Message::System(s)` 等形式，在「带 reducer 的 state」中改用 `Message { id, kind }`；或在一开始就给枚举变体加 `id: Option<String>`（略重）。本文用独立 `Message` 结构体表述 reducer 语义。

### 3.2 合并规则

- **输入**：`current: Vec<Message>`, `update: Vec<Message>`。
- **规则**：
  1. 为没有 `id` 的 `update` 中的消息分配新 id（如 UUID）。
  2. 建立 `current` 的 `id -> index` 映射。
  3. 遍历 `update`：若某条消息的 `id` 在 `current` 中已存在则替换该位置，否则追加到末尾。
  4. 「删除」：若 LangGraph 有 `RemoveMessage`，可在 Rust 用 `MessageKind::Remove` 或单独类型表示，合并时从结果中剔除对应 id；本方案可先只做「覆盖 + 追加」。

### 3.3 实现草图

```rust
/// Merges two message lists: same id => replace at that index, else append.
/// Aligns with LangGraph's add_messages(left, right).
pub fn add_messages(current: Vec<Message>, update: Vec<Message>) -> Vec<Message> {
    let mut update = update;
    for m in &mut update {
        if m.id.is_none() {
            m.id = Some(uuid::Uuid::new_v4().to_string());
        }
    }
    let mut merged = current;
    let mut by_id: std::collections::HashMap<String, usize> = merged
        .iter()
        .enumerate()
        .filter_map(|(i, m)| m.id.as_ref().map(|id| (id.clone(), i)))
        .collect();
    for m in update {
        if let Some(ref id) = m.id {
            if let Some(&idx) = by_id.get(id) {
                merged[idx] = m;
                continue;
            }
            by_id.insert(id.clone(), merged.len());
        }
        merged.push(m);
    }
    merged
}
```

### 3.4 作为 Reducer 使用

将 `add_messages` 挂到「消息列表」这一 state 字段上：

```rust
pub struct AddMessages;

impl Reducer<Vec<Message>> for AddMessages {
    fn merge(&self, current: Vec<Message>, update: Vec<Message>) -> Vec<Message> {
        add_messages(current, update)
    }
}
```

---

## 4. Partial State 与 State::merge

### 4.1 类型定义

若 state 有多个字段，节点可能只更新其中一部分；未出现的字段保持原样。Rust 侧用「每字段 Optional」表示 partial：

```rust
/// Partial update for AgentState: only present fields are merged.
#[derive(Debug, Clone, Default)]
pub struct AgentStateUpdate {
    pub messages: Option<Vec<Message>>,
}

/// Full state (same as minimal agent design).
#[derive(Debug, Clone, Default)]
pub struct AgentState {
    pub messages: Vec<Message>,
}
```

### 4.2 按字段应用 Reducer

每个字段绑定一个 reducer，合并时仅对 `Some(update)` 的字段调用 reducer，其余沿用 current：

```rust
impl AgentState {
    /// Merges a partial update into this state using the reducer for each field.
    pub fn merge(self, update: AgentStateUpdate) -> Self {
        let messages = match update.messages {
            Some(u) => add_messages(self.messages, u),
            None => self.messages,
        };
        AgentState { messages }
    }
}
```

若 state 扩展（如 `metadata: HashMap<String, String>`），可为该字段定义 reducer，在 `merge` 中同样按 `Option` 分支调用。

### 4.3 与 Agent / 图运行时对接

- **节点返回 partial**：节点签名可为 `fn run(&self, state: AgentState) -> Result<AgentStateUpdate, AgentError>`，只填充要更新的字段。
- **运行时合并**：调用方或图运行时执行 `new_state = current_state.clone().merge(node_return)`，再进入下一节点或返回。

与 LangGraph 的「节点返回 partial → 运行时按 reducer 合并」一致。

---

## 5. 使用方式与 Agent 组合

### 5.1 三种使用方式

| 方式 | 说明 | 代码要点 |
|------|------|----------|
| **节点内直接 add_messages** | 节点持有完整 state，内部合并后返回完整 State | `add_messages(state.messages, new_messages)` → `Ok(AgentState { messages })` |
| **通过 Reducer trait** | 按字段类型复用合并逻辑，或图运行时按字段 reducer 合并 | `AddMessages.merge(current_messages, update_messages)` |
| **节点返回 partial + 运行时 merge** | 节点只返回 `AgentStateUpdate`，由调用方/图运行时执行 merge | `state.merge(partial)` |

**节点返回 partial 示例**：

```rust
fn chatbot_node(state: &AgentState) -> Result<AgentStateUpdate, AgentError> {
    let reply = do_chat(state)?;
    Ok(AgentStateUpdate {
        messages: Some(vec![Message { id: None, kind: MessageKind::Assistant(reply) }]),
    })
}

// 调用方
let current_state = load_state(thread_id)?;
let partial = chatbot_node(&current_state)?;
let new_state = current_state.clone().merge(partial);
save_state(thread_id, &new_state)?;
```

### 5.2 端到端示例：单节点 Chat + add_messages

从「调用方传入 state」到「得到合并后的新 state」：

```rust
// 1) 当前 state（例如从 checkpointer 加载，或默认 + 本轮的 User 消息）
let mut state = AgentState::default();
state.messages.push(Message {
    id: Some("msg-1".into()),
    kind: MessageKind::User("你好".into()),
});

// 2) 节点只返回 partial：本轮要追加的 Assistant 消息
let partial = AgentStateUpdate {
    messages: Some(vec![Message {
        id: None,
        kind: MessageKind::Assistant("你好！有什么可以帮你？".into()),
    }]),
};

// 3) 运行时合并
state = state.merge(partial);

// 4) 使用新 state（取最后一条、写回 checkpointer 等）
let last = state.messages.last().unwrap();
```

### 5.3 Agent 与 Reducer 的组合模式

Agent（[09-minimal-agent-design](09-minimal-agent-design.md)）负责「从 state 得到更新」；Reducer 负责「把更新合并进 state」。组合方式如下。

| 模式 | Agent 产出 | 谁做合并 | 适用 |
|------|------------|----------|------|
| **一** | 完整 State | Agent 内部（add_messages） | 单节点、保持最简 Agent 签名 |
| **二** | partial (Update) | 调用方 / 图运行时 `state.merge(partial)` | 多节点、统一由运行时 merge |
| **三** | 多节点各返回 partial | 图运行时按字段 reducer 依次 merge | 多 Agent、多边图 |

**模式一**：Agent 内部用 reducer，返回完整 State。

```rust
#[async_trait]
impl Agent for ChatAgent<L>
where
    L: LlmClient + Send + Sync,
{
    type State = AgentState;

    async fn run(&self, state: Self::State) -> Result<Self::State, AgentError> {
        let reply = self.llm.chat(&state.messages).await?;
        let new_msg = Message { id: None, kind: MessageKind::Assistant(reply) };
        let merged = add_messages(state.messages, vec![new_msg]);
        Ok(AgentState { messages: merged })
    }
}
```

**模式二**：Agent 返回 partial，调用方 merge。可扩展 Agent trait（如 `run_partial`）或约定「节点返回 partial 则由运行时 merge」：

```rust
let partial = agent.run_partial(state.clone()).await?;
let new_state = state.merge(partial);
```

**模式三**：图运行时驱动多节点，每步 partial + merge。

```text
  ┌─────────────┐
  │   START     │
  └──────┬──────┘
         │ state_0
         ▼
  ┌─────────────┐     partial_1      ┌─────────────┐
  │  node_chat  │ ─────────────────► │   merge     │
  └─────────────┘                    │ (reducer)   │
         ▲                            └──────┬──────┘
         │ state_1 = state_0.merge(partial_1) │
         │                                    ▼
         │     partial_2               ┌─────────────┐
         └────────────────────────────│  node_tool   │
                                       └──────┬──────┘
                                              │ partial_2
                                              ▼
                                       state_2 = state_1.merge(partial_2) → END
```

### 5.4 使用场景小结

| 场景 | 用法 |
|------|------|
| 单节点、节点内合并 | 节点内直接 `add_messages`，返回完整 `State`。 |
| 单节点、由运行时合并 | 节点返回 `AgentStateUpdate`，调用方 `state.merge(partial)`。 |
| 多节点、每节点只写部分字段 | 每节点返回自己的 `AgentStateUpdate`，运行时依次 `state = state.merge(partial)`。 |
| 流式 chunk、按 ID 覆盖 | 先追加带固定 id 的 Assistant 占位，后续 chunk 用同 id 的 update 调用 add_messages，实现同一条消息被多次更新。 |

---

## 6. 与最简 Agent 的关系

- **09-minimal-agent-design** 中当前为「节点返回完整 State」：`run(state) -> State`，不依赖 reducer 即可工作。
- 引入 reducer 后可以：
  1. **保持现有签名**：节点仍可返回完整 state；在图运行时或多节点场景下，用 `State::merge(partial)` + 每字段 reducer 统一合并逻辑。
  2. **可选扩展**：支持节点返回 `AgentStateUpdate`，由运行时执行 `state.merge(update)`；reducer 与 `AgentStateUpdate` 成为图/运行时的标准约定。

**建议**：先实现 **Reducer trait** 与 **add_messages**（及 `Message` 带 id），单节点仍用「返回完整 state」；多节点或引入图运行时再统一改为 partial + `merge`。

---

## 7. 小结

| 项 | 设计 |
|----|------|
| **Reducer** | `trait Reducer<T>: merge(current, update) -> T`；可配合无状态 `ReducerFn`。 |
| **add_messages** | `Message` 带 `id: Option<String>`；合并规则：同 ID 覆盖，否则追加；可 impl `Reducer<Vec<Message>>`（如 `AddMessages`）。 |
| **Partial state** | `AgentStateUpdate` 每字段 `Option<T>`；`AgentState::merge(update)` 对 `Some` 字段调用对应 reducer。 |
| **使用** | 节点内 `add_messages`、通过 `Reducer`、节点返回 partial + 运行时 `merge`（§5）。 |
| **Agent 组合** | 模式一：Agent 内 reducer 返回完整 State；模式二：Agent 返回 partial、调用方 merge；模式三：图运行时多节点 partial + merge（§5.3）。 |
| **与最简 Agent** | 单节点可继续 `run(state) -> State`；多节点/图运行时用 partial + `merge` 与 LangGraph 对齐。 |

在 Rust 侧得到与 LangGraph 一致的 reducer 语义，并支持后续多节点与图运行时扩展。
