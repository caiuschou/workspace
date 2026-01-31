# Permission 权限

> [← 返回目录](README.md)

---

### `GET /permission`

- **OperationId**: `permission.list`
- **摘要**: List pending permissions
- **说明**: Get all pending permission requests across all sessions.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of pending permissions | `array` |

---

### `POST /permission/{requestID}/reply`

- **OperationId**: `permission.reply`
- **摘要**: Respond to permission request
- **说明**: Approve or deny a permission request from the AI assistant.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `requestID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `reply` | string | **必填** |  |
| `message` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Permission processed successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---
