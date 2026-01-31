# Provider 模型提供商

> [← 返回目录](README.md)

---

### `GET /provider`

- **OperationId**: `provider.list`
- **摘要**: List providers
- **说明**: Get a list of all available AI providers, including both available and connected ones.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of providers | `object` |

---

### `GET /provider/auth`

- **OperationId**: `provider.auth`
- **摘要**: Get provider auth methods
- **说明**: Retrieve available authentication methods for all AI providers.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Provider auth methods | `object` |

---

### `POST /provider/{providerID}/oauth/authorize`

- **OperationId**: `provider.oauth.authorize`
- **摘要**: OAuth authorize
- **说明**: Initiate OAuth authorization for a specific AI provider to get an authorization URL.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `providerID` | string | **必填** | Provider ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `method` | number | **必填** | Auth method index |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Authorization URL and method | `ProviderAuthAuthorization` |
| 400 | Bad request | `BadRequestError` |

---

### `POST /provider/{providerID}/oauth/callback`

- **OperationId**: `provider.oauth.callback`
- **摘要**: OAuth callback
- **说明**: Handle the OAuth callback from a provider after user authorization.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `providerID` | string | **必填** | Provider ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `method` | number | **必填** | Auth method index |
| `code` | string | 可选 | OAuth authorization code |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | OAuth callback processed successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |

---
