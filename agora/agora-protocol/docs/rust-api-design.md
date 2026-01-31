# Agora Rust API 方案（最简版）

基于 [agent-communication.md](../../docs/agent-communication.md) 与 [registration-discovery.md](../../docs/registration-discovery.md)，为 Agora 设计一组最简 Rust API：仅定义协议消息类型与序列化，供 `agora-protocol` crate 与 `agora-server` 共用。

## 目标

- **范围**：仅消息类型 + JSON 序列化/反序列化，不包含 WebSocket、异步、网络逻辑。
- **用途**：Agent 与 Agora 服务器收发 JSON 文本帧时，统一用本 crate 的类型进行构造与解析。
- **原则**：类型与协议文档一一对应，保持最少字段、最少枚举变体。

---

## 1. 协议消息分类

所有消息均为 WebSocket 文本帧，JSON 格式，用 `type` 字段区分。

| 方向 | 用途 | 消息类型 |
|------|------|----------|
| Client → Server | 注册/发现 | `register`, `list` |
| Server → Client | 注册/发现响应 | `registered`, `agents` |
| Sender → Agora | 发消息/流式 | `send`, `send_chunk`, `send_end` |
| Agora → Receiver | 收消息/流式 | `message`, `message_chunk`, `message_end` |
| Receiver → Agora | 回复（可选） | `reply`, `reply_chunk`, `reply_end` |
| Agora → Sender | 转发回复 | 同上（`reply` / `reply_chunk` / `reply_end`） |

---

## 2. 类型设计

### 2.1 注册与发现（已有 + 补充）

- **AgentRecord**（已有）：`id`, `name`, `endpoint?`, `registered_at`
- **Register**：`type = "register"`, `id`, `name`, `endpoint?`
- **Registered**：`type = "registered"`, `id`
- **List**：`type = "list"`（无其它必选字段）
- **Agents**：`type = "agents"`, `agents: Vec<AgentRecord>`

### 2.2 发送端 → Agora

- **Send**：`type = "send"`, `to`, `stream_id?`, `stream?`, `payload?`
- **SendChunk**：`type = "send_chunk"`, `to`, `stream_id`, `chunk`
- **SendEnd**：`type = "send_end"`, `to`, `stream_id`

### 2.3 Agora → 接收端

- **Message**：`type = "message"`, `from`, `stream_id?`, `stream?`, `payload?`
- **MessageChunk**：`type = "message_chunk"`, `from`, `stream_id`, `chunk`
- **MessageEnd**：`type = "message_end"`, `from`, `stream_id`

### 2.4 接收端 → Agora / Agora → 发送端（回复）

- **Reply**：`type = "reply"`, `to`, `stream_id?`, `payload?`
- **ReplyChunk**：`type = "reply_chunk"`, `to`, `stream_id`, `chunk`
- **ReplyEnd**：`type = "reply_end"`, `to`, `stream_id`

协议中可选字段（如 `request_id`）可放在 `payload` 中，首版不单独建模，保持 API 最简。

---

## 3. Rust API 形态

### 3.1 枚举：统一入站/出站帧

- **ClientFrame**：客户端发往服务器的所有消息  
  `Register`, `List`, `Send`, `SendChunk`, `SendEnd`, `Reply`, `ReplyChunk`, `ReplyEnd`
- **ServerFrame**：服务器发往客户端的消息  
  `Registered`, `Agents`, `Message`, `MessageChunk`, `MessageEnd`, `Reply`, `ReplyChunk`, `ReplyEnd`

这样：
- 服务端：收 `ClientFrame`，发 `ServerFrame`
- 发送端 Agent：发 `ClientFrame`（Send/SendChunk/SendEnd），收 `ServerFrame`（含 Reply*）
- 接收端 Agent：收 `ServerFrame`（Message/MessageChunk/MessageEnd），发 `ClientFrame`（Reply/ReplyChunk/ReplyEnd）

### 3.2 序列化

- 使用 `serde` 的 `Serialize` / `Deserialize`，JSON 字段名与协议一致（snake_case）。
- `type` 由枚举变体通过 `#[serde(tag = "type")]` 或 `rename` 映射，保证线缆格式为 `"type": "send"` 等。

### 3.3 模块划分建议

| 模块 | 内容 |
|------|------|
| `types` | `AgentRecord`（已有）及所有消息结构体 |
| `client_frame` | `ClientFrame` 枚举及变体对应结构体 |
| `server_frame` | `ServerFrame` 枚举及变体对应结构体 |
| `lib.rs` | 重导出 `types`、`client_frame`、`server_frame` |

若追求"一个类型一个文件"，可将每个消息类型单独文件放在 `frames/` 下，首版也可全部放在 `types.rs` + `client_frame.rs` + `server_frame.rs` 三个文件，以简单为先。

---

## 4. 使用方式示例

```rust
// 解析客户端发来的一帧
let json = r#"{"type":"send","to":"agent-2","payload":{"action":"query"}}"#;
let frame: ClientFrame = serde_json::from_str(json)?;
match frame {
    ClientFrame::Send(s) => println!("Send to {} payload {:?}", s.to, s.payload),
    _ => {}
}

// 构造服务端回复
let out = ServerFrame::Registered(Registered { id: "agent-1".into() });
let json = serde_json::to_string(&out)?;
// 通过 WebSocket 发送 json
```

---

## 5. 依赖

- `serde`, `serde_json`：序列化
- `chrono`：`AgentRecord.registered_at`（已有）

不引入 tokio、tungstenite 等，保持 `agora-protocol` 为纯类型 + 序列化 crate。

---

## 6. 小结

| 项目 | 内容 |
|------|------|
| 输入 | agent-communication.md + registration-discovery.md |
| 输出 | 最简 Rust 类型 + ClientFrame/ServerFrame + JSON 序列化 |
| 不包含 | WebSocket、异步、路由、存储 |
| 下一步 | 在 `agora-protocol/src` 中实现上述类型与枚举，并补充单元测试（解析/序列化往返与示例 JSON）。 |

完成本方案后，在 `docs/` 的任务表中可将「Agora 最简 Rust API 方案」标为完成，并新增「实现 agora-protocol 消息类型与序列化」任务。
