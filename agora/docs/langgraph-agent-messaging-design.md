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
│   │  State: user_message,       │      │  State: inbound, outbound   │   │
│   │  inbound, outbound,         │      │                             │   │
│   │  reply_to_user              │      │                             │   │
│   └──────────────┬──────────────┘      └──────────────▲──────────────┘   │
│                  │ outbound (envelope)                │ outbound (reply) │
│                  └────────────────────────────────────┘                  │
└──────────────────────────────────────────────────────────────────────────┘
```

- **用户** 只与运行时通信；运行时把用户消息写入助理 state，把助理的 `reply_to_user` 发回用户。
- **助理图** 与 **邮件图** 之间不直接连接；运行时根据 `outbound.to` 投递，并把邮件 Agent 的回复写回助理的 `inbound` 后再次 invoke 助理。

---

## 2. 核心思路

- **消息只存在于 State 里**：不改变 `Node::run(state) -> state` 的接口。
- **「发消息」** = 节点往 state 的 **outbound** 写入 `AgentEnvelope`。
- **「收消息」** = 节点从 state 的 **inbound** 读取 `AgentEnvelope`。
- **谁真正送信**：由图外部的**运行时**（Agora 或测试用 runner）负责：把 outbound 投递到对应 agent，把回复写回该 agent 的 state（inbound 或按 request_id 的回复表），再下一次 `invoke`。

langgraph-rust 只约定「消息长什么样、在 state 里放在哪」；传输、路由、WebSocket 由 Agora 实现。

### 2.1 内外状态边界（以 invoke 为界）

```
  外部（用户 / 其他 Agent）           运行时                    内部（图 State）
  ────────────────────────          ──────                    ─────────────────

        用户消息                     invoke 前
             │                         │
             └────────────────────────►│  写入 user_message / inbound
                                       │
                                       ▼
                              ┌─────────────────┐
                              │  state (入参)   │
                              │  inbound        │
                              │  user_message   │
                              └────────┬────────┘
                                       │
                                       │  invoke(state)
                                       ▼
                              ┌─────────────────┐
                              │  图内节点读写     │
                              │  outbound        │
                              │  reply_to_user   │
                              └────────┬────────┘
                                       │
                                       │  invoke 后
                                       ▼
        用户 / 其他 Agent ◄────────────│  读出 outbound, reply_to_user 并发送
```

- **外部 → 内部**：仅在 invoke 前，由运行时写入 state 的 `user_message` 或 `inbound`。
- **内部 → 外部**：仅在 invoke 后，由运行时从 state 读出 `outbound`、`reply_to_user` 并发送。

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

**实现归属**：类型放在 **agora-protocol**（或共享 crate），便于 Agora 与 langgraph 共用同一信封格式；langgraph 可依赖或 re-export。

---

## 4. State 形状（消息能力）

需要与其它 agent 通信的图，在 State 中增加「消息收/发」的一块，例如：

```text
// 消息状态块（与业务 state 组合）
struct MessagingState {
    inbound: Vec<AgentEnvelope>,   // 本轮或上一轮投递进来的
    outbound: Vec<AgentEnvelope>, // 本轮要发出去的
}

// 助理图状态示例
struct AssistantState {
    user_message: Message,           // 或 messages: Vec<Message>
    messaging: MessagingState,
    reply_to_user: Option<Message>,  // 最终给用户的回复
}
```

- **inbound**：运行时在每次 `invoke` 前写入（例如邮件 Agent 的回复、或用户请求转换后的 envelope）。
- **outbound**：节点在 `run` 里 push；`invoke` 结束后由运行时取走并投递。

「拥有消息能力」= 使用带 `MessagingState`（或等价字段）的 State，并在节点中读写 inbound/outbound。

---

## 5. 助理图与邮件 Agent 图（langgraph-rust）

### 5.0 场景流程（用户发邮件）

```
用户          运行时                助理图                    邮件 Agent 图
 │              │                     │                            │
 │  用户消息    │ 写 user_message      │                            │
 │─────────────►│────────────────────►│ invoke                     │
 │              │                     │  解析 → outbound            │
 │              │◄────────────────────│  (to=邮件Agent, req_id)    │
 │              │ 读 outbound         │                            │
 │              │ 存助理 state         │                            │
 │              │                     │                            │
 │              │ 写 inbound          │                            │
 │              │──────────────────────────────────────────────────►│ invoke
 │              │                     │                            │  处理 → outbound
 │              │◄──────────────────────────────────────────────────│  (reply, req_id)
 │              │ 读 outbound         │                            │
 │              │                     │                            │
 │              │ 写 inbound (回复)    │                            │
 │              │────────────────────►│ invoke                     │
 │              │                     │  汇总 → reply_to_user        │
 │              │◄────────────────────│                            │
 │  回复        │ 读 reply_to_user    │                            │
 │◄─────────────│                     │                            │
```

1. 用户发话 → 运行时写入助理 state → 助理 invoke，写出 outbound。
2. 运行时把 outbound 发给邮件 Agent，写邮件 Agent 的 inbound → 邮件 invoke，写出回复 outbound。
3. 运行时把回复写回助理 inbound → 助理再次 invoke，写出 reply_to_user。
4. 运行时把 reply_to_user 发回用户。

### 5.1 助理图

- **状态**：如 `AssistantState`（含 `user_message`、`messaging`、`reply_to_user`）。
- **节点 1「解析」**：读 `user_message`（或唯一一条 inbound）；若需发/查邮件，构造 `AgentEnvelope`（to = 邮件 Agent 的 agent-id，from = 助理 agent-id，request_id = 新 UUID，body = Complete({ action, to, subject, body } 等）并 push 到 `state.messaging.outbound`。
- **节点 2「汇总回复」**：读 `state.messaging.inbound`（或按 request_id 的回复），生成 `reply_to_user`，写回 state。
- **边**：START → 解析 → 汇总回复 → END；若需「等回复再汇总」，可由运行时在收到回复后再次 invoke，此时 inbound 已填好。

### 5.2 邮件 Agent 图

- **状态**：含 `MessagingState`（inbound/outbound）。
- **节点「处理」**：从 `inbound` 取一条（或按 request_id），执行 send/list，将结果写入 `outbound` 一条信封（to = 请求的 from，request_id = 原 request_id，body = Complete(result)）。
- 实现「可选回复 + request_id 关联」。

图结构保持线性链或条件边即可；跨 agent 的送达由运行时完成。

---

## 6. 运行时职责（与 Agora 的衔接）

| 时机 | 职责 |
|------|------|
| **invoke 前** | 根据会话/线程，把「该 agent 本轮应收到的信封」写入 state 的 `inbound`（或按 request_id 索引的回复表）。 |
| **invoke 后** | 若 `state.messaging.outbound` 非空：按 `to` 找到目标 agent（及连接）；发送 send / send_chunk / send_end（对应 [agent-communication](agent-communication.md) 的 JSON 帧）；若需回复，收 reply/reply_chunk/reply_end（带同一 request_id），将回复信封写回**发送方**的 state（下一轮 inbound 或 reply 表），并对发送方再调用一次 `invoke`。 |

流式与 request_id 在 Agora 协议层实现；langgraph 侧只处理「完整一条」或「流式一条」在 state 中的表示（如流式用 StreamStart/Chunk/End 在 inbound/outbound 中顺序排列）。

---

## 7. 小结

| 维度 | 做法 |
|------|------|
| 谁对谁 | `AgentEnvelope.from` / `.to`；运行时按 `to` 投递。 |
| 完整 vs 流式 | `EnvelopeBody` 的 Complete vs Stream*；state 中统一用信封队列。 |
| request_id | 信封带可选 request_id，回复带同值；运行时负责关联。 |
| 助理场景 | 助理图：解析 → 写 outbound；汇总 → 读 inbound 写 reply_to_user。邮件图：读 inbound → 执行 → 写 outbound。 |
| langgraph-rust | 不引入「远程调用」原语；只约定 State 中的 MessagingState + 信封类型；图仍是 `invoke(state) -> state`。 |

---

## 8. 任务表

| 序号 | 任务 | 状态 | 备注 |
|------|------|------|------|
| 1 | 在 agora-protocol 中定义 AgentId、RequestId、EnvelopeBody、AgentEnvelope | 未开始 | 与 agent-messaging-abstraction 对齐 |
| 2 | 文档或示例中定义 MessagingState 与 AssistantState/邮件 Agent State 示例 | 未开始 | 可放在本目录或 langgraph 示例 |
| 3 | langgraph 侧示例图 + 节点（使用上述类型的助理/邮件 Agent） | 未开始 | 可选 re-export 类型 |
| 4 | Agora 侧实现「invoke 前写 inbound、invoke 后处理 outbound 并投递、收回复再 invoke」循环 | 未开始 | 与 agent-communication 协议对接 |

完成一项即在表中将状态改为「已完成」并补充备注。
