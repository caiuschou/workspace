# Logging 日志

> [← 返回目录](README.md)

---

### `POST /log`

- **OperationId**: `app.log`
- **摘要**: Write log
- **说明**: Write a log entry to the server logs with specified level and metadata.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `service` | string | **必填** | Service name for the log entry |
| `level` | string | **必填** | Log level |
| `message` | string | **必填** | Log message |
| `extra` | object | 可选 | Additional metadata for the log entry |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Log entry written successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |

---
