# langgraph-rust Agent 消息能力方案

基于 [agent-messaging-abstraction](agent-messaging-abstraction.md) 与 [scenario-assistant-email](scenario-assistant-email.md)，给出在 **langgraph-rust** 中让 agent 具备「谁对谁、完整/流式消息、可选 request_id 关联」能力的简单设计。实现时按任务表推进，完成后标记。

**前置阅读**：agent-messaging-abstraction.md、scenario-assistant-email.md、[rust-langgraph 11-state-graph-design](../../docs/rust-langgraph/11-state-graph-design.md)、[09-minimal-agent-design](../../docs/rust-langgraph/09-minimal-agent-design.md)。

---

## 1. 目标

| 维度 | 内容 |
|------|------|
| **抽象** | 满足 agent-messaging-abstraction：from/to、完整消息或流、可选 request_id 关联请求与回复。 |
| **场景** | 支持 scenario-assistant-email：用户 → 助理 → 邮件 Agent；助理转发、邮件 Agent 可选回复并带同一 request_id。 |
| **与 langgraph 对齐** | 不改变 State + Node + StateGraph.invoke 的模型；消息能力通过 State 形状与运行时约定实现。 |

### 1.1 架构总图

```
                    ┌─────────────┐
                    │    用户      │
                    └──────┬──────┘
                           │ 用户消息 / 回复
                           ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  运行时 (Agora)：invoke 前写 state，invoke 后读 state，负责投递与存储      │
│                                                                          │
│   ┌─────────────────────────────┐      ┌─────────────────────────────┐   │
│   │  助理 Agent (StateGraph)    │      │  邮件 Agent (StateGraph)     │   │
│   │  ┌─────┐     ┌───────────┐  │      │  ┌───────────┐               │   │
│   │  │解析 │ ──► │ 汇总回复   │  │      │  │  处理     │               │   │
│   │  └─────┘     └───────────┘  │      │  └───────────┘               │   │
│   │  State: user_message,       │      │  State: 请求/结果字段         │   │
│   │  reply_to_user, 远程输入/结果 │     │                             │   │
│   └──────────────┬──────────────┘      └──────────────▲──────────────┘   │
│                  │ 远程节点：Agora 传 state 子集        │ Agora 返回写回 state │
│                  └────────────────────────────────────┘                  │
└──────────────────────────────────────────────────────────────────────────┘
```

- **用户** 只与运行时通信；运行时把用户消息写入助理 state，把助理的 `reply_to_user` 发回用户。
- **助理图** 与 **邮件图** 之间不直接连接；执行到**远程节点**（邮件 Agent）时，运行时通过 **Agora** 把约定 state 子集发给邮件 Agent，收回复后写回助理 state，再继续执行。

---

## 2. 核心思路

- **不改变 Node 接口**：仍为 `Node::run(state) -> state`；State 只含业务字段（如 `user_message`、`reply_to_user`、远程调用的输入/结果字段）。
- **远程节点**：图中标注为远程的节点不本地执行；执行到该节点时，由**运行时**（Agora）把当前 state 的约定部分作为请求发给远程 agent，收回复后写回 state 的约定字段，再继续执行后续节点。
- **谁真正送信**：仅对**远程节点**，由运行时通过 **Agora** 协议完成请求/回复；传输、路由、WebSocket 由 Agora 实现。

langgraph-rust 只约定「远程节点标注 + state 中约定输入/结果字段」；不约定统一的 inbound/outbound 队列。

### 2.1 内外状态边界（以 invoke 为界）

```
  外部（用户 / 其他 Agent）           运行时                    内部（图 State）
  ────────────────────────          ──────                    ─────────────────

        用户消息                     invoke 前
             │                         │
             └────────────────────────►│  写入 user_message 等
                                       │
                                       ▼
                              ┌─────────────────┐
                              │  state (入参)   │
                              │  user_message   │
                              │  远程结果等      │
                              └────────┬────────┘
                                       │
                                       │  invoke(state)
                                       │  遇远程节点 → Agora 发请求、收回复、写回 state
                                       ▼
                              ┌─────────────────┐
                              │  图内节点读写     │
                              │  reply_to_user   │
                              └────────┬────────┘
                                       │
                                       │  invoke 后
                                       ▼
        用户 / 其他 Agent ◄────────────│  读出 reply_to_user 并发送
```

- **外部 → 内部**：仅在 invoke 前，由运行时写入 state 的 `user_message` 等。
- **内部 → 外部**：仅在 invoke 后，由运行时从 state 读出 `reply_to_user` 并发送；远程节点处的收发由 Agora 在 invoke 过程中完成。

---

## 3. 类型设计（对齐抽象）

与 agent-messaging-abstraction 一致，最小约定如下。

| 类型 | 说明 |
|------|------|
| **AgentId** | 稳定标识，如 `String` 或 newtype。 |
| **RequestId** | 可选，用于请求–回复对应，如 `String` 或 `Uuid`。 |
| **EnvelopeBody** | 枚举：`Complete(payload)`、`StreamStart { stream_id, metadata }`、`StreamChunk { stream_id, chunk }`、`StreamEnd { stream_id }`。 |
| **AgentEnvelope** | `from: AgentId`, `to: AgentId`, `request_id: Option<RequestId>`, `body: EnvelopeBody`。 |

业务内容（如 `action: "send"`, `body: {...}`）放在 `Complete(payload)` 里，由助理与邮件 Agent 自行约定。流式用同一 `stream_id` 串起 start → chunks → end。

**实现归属**：类型放在 **agora-protocol**（或共享 crate），供 Agora 协议层在线上传递时使用；图内 state 不包含 inbound/outbound 队列，仅用业务字段（如 `mail_request`、`mail_result`）。langgraph 可依赖或 re-export 类型。

---

## 4. State 形状

与其它 agent 通信的图，State 只含业务字段；远程调用的输入与结果用约定字段表示，由运行时在执行远程节点时通过 Agora 传递。例如：

```text
// 助理图状态示例（无 inbound/outbound）
struct AssistantState {
    user_message: Message,           // 或 messages: Vec<Message>
    mail_request: Option<MailRequest>,  // 发给邮件 Agent 的输入（解析节点写入）
    mail_result: Option<MailResult>,   // 邮件 Agent 的回复（运行时通过 Agora 写回）
    reply_to_user: Option<Message>,  // 最终给用户的回复
}
```

- **远程输入**：本地节点（如「解析」）在 `run` 里写入约定字段（如 `mail_request`）。
- **远程结果**：执行到远程节点时，运行时通过 Agora 发请求、收回复，将结果写回约定字段（如 `mail_result`）。
- **回复用户**：本地节点（如「汇总回复」）读 `mail_result` 等，生成 `reply_to_user`。

---

## 5. 远程节点（Remote Node）

在 **add_node** 时可将节点标注为「远程节点」。仅对远程节点，运行时使用 **Agora** 处理消息与协议；本地节点行为不变，图模型仍是 State + Node + invoke，不新增远程调用原语。

### 5.1 约定

| 方式 | 说明 |
|------|------|
| **add_node 扩展** | `add_node(name, node, remote: bool)` 或 `add_node(name, node, NodeKind::Local \| Remote(AgentId))`：标注该节点是否为远程、以及远程时的 agent_id。 |
| **add_remote_node（可选）** | `add_remote_node(name, remote_agent_id)`：仅声明图中有一远程节点 name，对应远程 agent_id，不绑定本地 Node 实现。 |

- **本地节点**：照常执行 `node.run(state)`，在 state 中读写业务字段。
- **远程节点**：不在本进程执行节点代码；运行时（Agora）根据该节点对应的 remote agent_id，将 state 的约定输入字段通过 Agora 协议发给远程 agent，接收回复后写回本图 state 的约定结果字段，再继续执行后续节点。

即：**远程节点 = 图中的占位 + 由 Agora 完成「发请求 → 收回复 → 写回 state」**。

### 5.2 与运行时衔接

执行到图中**标注为远程**的节点时，运行时使用 **Agora** 协议：取 state 的约定输入、发给对应 remote agent、收回复、写回 state 的约定结果字段，然后继续执行下一节点。

---

## 6. 助理图与邮件 Agent 图（langgraph-rust）

### 6.0 场景流程（用户发邮件）

```
用户          运行时                助理图                    邮件 Agent 图
 │              │                     │                            │
 │  用户消息    │ 写 user_message      │                            │
 │─────────────►│────────────────────►│ invoke                     │
 │              │                     │  解析 → mail_request        │
 │              │                     │  执行到远程节点「邮件 Agent」│
 │              │ 取 mail_request      │                            │
 │              │──────────────────────────────────────────────────►│ Agora 调用
 │              │                     │                            │  处理 → 返回结果
 │              │◄──────────────────────────────────────────────────│
 │              │ 写回 mail_result     │                            │
 │              │                     │  汇总 → reply_to_user       │
 │              │◄────────────────────│                            │
 │  回复        │ 读 reply_to_user    │                            │
 │◄─────────────│                     │                            │
```

1. 用户发话 → 运行时写入助理 state → 助理 invoke，解析节点写出 `mail_request`。
2. 执行到远程节点「邮件 Agent」→ 运行时通过 Agora 把 `mail_request` 发给邮件 Agent，收回复写回 `mail_result`。
3. 汇总节点读 `mail_result`，写出 `reply_to_user`。
4. 运行时把 `reply_to_user` 发回用户。

### 6.1 助理图

- **状态**：如 `AssistantState`（含 `user_message`、`mail_request`、`mail_result`、`reply_to_user`）。
- **节点 1「解析」**：读 `user_message`；若需发/查邮件，构造 `mail_request` 并写回 state。
- **节点 2「邮件 Agent」**：标注为**远程节点**（如 `add_node("mail_agent", ..., remote: true)` 或 `add_remote_node("mail_agent", mail_agent_id)`）；执行到该节点时由运行时通过 Agora 把 `mail_request` 发给邮件 Agent，收回复写回 `mail_result`。
- **节点 3「汇总回复」**：读 `mail_result`，生成 `reply_to_user`，写回 state。
- **边**：START → 解析 → 邮件 Agent（远程）→ 汇总回复 → END；远程节点处由运行时通过 Agora 完成请求/回复后再继续。

### 6.2 邮件 Agent 图

- **状态**：含业务字段（如请求 payload、结果）；由 Agora 调用时传入请求、写回结果。
- **节点「处理」**：读 state 中的请求字段，执行 send/list，将结果写回 state 的结果字段。
- 可选回复与 request_id 关联由 Agora 协议层约定；图内只处理「请求 → 结果」的 state 字段。

图结构保持线性链或条件边即可；跨 agent 的调用由运行时在远程节点处通过 Agora 完成。

---

## 7. 运行时职责（与 Agora 的衔接）

| 时机 | 职责 |
|------|------|
| **invoke 前** | 根据会话/线程，把用户消息等写入 state 的约定字段（如 `user_message`）。 |
| **invoke 中** | 执行到**标注为远程**的节点时：取 state 的约定输入字段，通过 **Agora** 协议发送请求（对应 [agent-communication](agent-communication.md) 的 JSON 帧）；收回复后写回 state 的约定结果字段，再继续执行下一节点。 |
| **invoke 后** | 从 state 读出 `reply_to_user` 等，发回用户。 |

流式与 request_id 在 Agora 协议层实现；langgraph 侧只约定 state 中的业务字段（输入/结果），不约定统一的 inbound/outbound 队列。

---

## 8. 小结

| 维度 | 做法 |
|------|------|
| 谁对谁 | 远程节点对应 remote agent_id；运行时按标注的 agent_id 通过 Agora 调用。 |
| 完整 vs 流式 | 由 Agora 协议层约定；state 中只含业务输入/结果字段。 |
| request_id | 由 Agora 协议层约定；state 中不强制信封队列。 |
| **远程节点** | `add_node` 可标注 `remote` 或使用 `NodeKind::Remote(agent_id)`；仅对远程节点由 Agora 处理请求与回复。 |
| 助理场景 | 助理图：解析 → 写 mail_request；邮件 Agent（远程节点）→ 运行时通过 Agora 发请求、写回 mail_result；汇总读 mail_result 写 reply_to_user。邮件图：读请求字段 → 执行 → 写结果字段。 |
| langgraph-rust | 不引入「远程调用」原语；只约定 State 中的业务字段 + 远程节点标注；图仍是 `invoke(state) -> state`；无 inbound/outbound。 |

---

## 9. 任务表

| 序号 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 1 | 在 agora-protocol 中定义 AgentId、RequestId、EnvelopeBody、AgentEnvelope（供 Agora 协议层） | 未开始 | 与 agent-messaging-abstraction 对齐 |
| 2 | 文档或示例中定义 AssistantState/邮件 Agent State 示例（业务字段，无 inbound/outbound） | 未开始 | 可放在本目录或 langgraph 示例 |
| 3 | langgraph 侧扩展 add_node 支持远程节点标注（remote / NodeKind::Remote(agent_id) 或 add_remote_node） | 未开始 | 运行时仅对远程节点走 Agora 协议 |
| 4 | langgraph 侧示例图 + 节点（助理/邮件 Agent，含远程节点，state 用约定输入/结果字段） | 未开始 | 可选 re-export 类型 |
| 5 | Agora 侧实现「invoke 前写 state、执行到远程节点时通过 Agora 发请求收回复写回 state」 | 未开始 | 与 agent-communication 协议对接；仅对图中标注为远程的节点使用 Agora |

完成一项即在表中将状态改为「已完成」并补充备注。
