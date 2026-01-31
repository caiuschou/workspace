# Auth 认证

> [← 返回目录](README.md)

---

### `PUT /auth/{providerID}`

- **OperationId**: `auth.set`
- **摘要**: Set auth credentials
- **说明**: Set authentication credentials

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `providerID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully set authentication credentials | `boolean` |
| 400 | Bad request | `BadRequestError` |

---
