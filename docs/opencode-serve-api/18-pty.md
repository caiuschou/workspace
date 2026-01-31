# PTY 伪终端

> [← 返回目录](README.md)

---

### `GET /pty`

- **OperationId**: `pty.list`
- **摘要**: List PTY sessions
- **说明**: Get a list of all active pseudo-terminal (PTY) sessions managed by OpenCode.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of sessions | `array` |

---

### `POST /pty`

- **OperationId**: `pty.create`
- **摘要**: Create PTY session
- **说明**: Create a new pseudo-terminal (PTY) session for running shell commands and processes.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `command` | string | 可选 |  |
| `args` | array | 可选 |  |
| `cwd` | string | 可选 |  |
| `title` | string | 可选 |  |
| `env` | object | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Created session | `Pty` |
| 400 | Bad request | `BadRequestError` |

---

### `DELETE /pty/{ptyID}`

- **OperationId**: `pty.remove`
- **摘要**: Remove PTY session
- **说明**: Remove and terminate a specific pseudo-terminal (PTY) session.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `ptyID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Session removed | `boolean` |
| 404 | Not found | `NotFoundError` |

---

### `GET /pty/{ptyID}`

- **OperationId**: `pty.get`
- **摘要**: Get PTY session
- **说明**: Retrieve detailed information about a specific pseudo-terminal (PTY) session.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `ptyID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Session info | `Pty` |
| 404 | Not found | `NotFoundError` |

---

### `PUT /pty/{ptyID}`

- **OperationId**: `pty.update`
- **摘要**: Update PTY session
- **说明**: Update properties of an existing pseudo-terminal (PTY) session.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `ptyID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `title` | string | 可选 |  |
| `size` | object | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Updated session | `Pty` |
| 400 | Bad request | `BadRequestError` |

---

### `GET /pty/{ptyID}/connect`

- **OperationId**: `pty.connect`
- **摘要**: Connect to PTY session
- **说明**: Establish a WebSocket connection to interact with a pseudo-terminal (PTY) session in real-time.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `ptyID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Connected session | `boolean` |
| 404 | Not found | `NotFoundError` |

---
