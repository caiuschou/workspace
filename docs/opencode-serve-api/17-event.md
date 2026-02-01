# Event 事件

> [← 返回目录](README.md)  
> **事件 payload 详细格式**：[17-event-format.md](17-event-format.md)

---

### `GET /event`

- **OperationId**: `event.subscribe`
- **摘要**: Subscribe to events
- **说明**: Get events

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Event stream | `Event` |

---
