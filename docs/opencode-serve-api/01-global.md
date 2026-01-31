# Global 全局

> [← 返回目录](README.md)

---

### `POST /global/dispose`

- **OperationId**: `global.dispose`
- **摘要**: Dispose instance
- **说明**: Clean up and dispose all OpenCode instances, releasing all resources.

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Global disposed | `boolean` |

---

### `GET /global/event`

- **OperationId**: `global.event`
- **摘要**: Get global events
- **说明**: Subscribe to global events from the OpenCode system using server-sent events.

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Event stream | `GlobalEvent` |

---

### `GET /global/health`

- **OperationId**: `global.health`
- **摘要**: Get health
- **说明**: Get health information about the OpenCode server.

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Health information | `object` |

---
