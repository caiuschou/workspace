# Event 事件

> [← 返回目录](README.md)  
> **事件 payload 详细格式**：[17-event-format.md](17-event-format.md)

---

### `GET /event`

- **OperationId**: `event.subscribe`
- **摘要**: Subscribe to events
- **说明**: 订阅实例级服务端事件流（Server-Sent Events）。用于监听会话状态变化、消息更新、权限请求等，实现 UI 实时刷新或 SDK 完成检测。

**请求**

- **Headers**: `Accept: text/event-stream`（SSE 标准）
- **响应类型**: `text/event-stream`，每条 event 的 `data` 字段为 JSON 对象

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 | 指定工作目录，仅接收该目录（项目）相关的事件；未传时接收当前实例所有事件 |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Event stream | `Event` |

**事件结构**

每条 SSE 事件的 `data` 为 JSON，顶层含 `type` 和 `properties`。常见类型：

- `server.connected` — 连接建立
- `server.heartbeat` — 心跳（约 30 秒）
- `session.idle` / `session.status` — 会话空闲（完成信号）
- `message.updated` / `message.part.updated` — 消息/part 更新（含流式 text delta）

完整字段与示例见 [17-event-format.md](17-event-format.md)。

---
