# Question 问题

> [← 返回目录](README.md)

---

### `GET /question`

- **OperationId**: `question.list`
- **摘要**: List pending questions
- **说明**: Get all pending question requests across all sessions.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of pending questions | `array` |

---

### `POST /question/{requestID}/reject`

- **OperationId**: `question.reject`
- **摘要**: Reject question request
- **说明**: Reject a question request from the AI assistant.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `requestID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Question rejected successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /question/{requestID}/reply`

- **OperationId**: `question.reply`
- **摘要**: Reply to question request
- **说明**: Provide answers to a question request from the AI assistant.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `requestID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `answers` | array<QuestionAnswer> | **必填** | User answers in order of questions (each answer is an array of selected labels) |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Question answered successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---
