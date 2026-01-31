# LSP / Formatter / MCP

> [← 返回目录](README.md)

---

### `GET /formatter`

- **OperationId**: `formatter.status`
- **摘要**: Get formatter status
- **说明**: Get formatter status

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Formatter status | `array` |

---

### `GET /lsp`

- **OperationId**: `lsp.status`
- **摘要**: Get LSP status
- **说明**: Get LSP server status

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | LSP server status | `array` |

---

### `GET /mcp`

- **OperationId**: `mcp.status`
- **摘要**: Get MCP status
- **说明**: Get the status of all Model Context Protocol (MCP) servers.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | MCP server status | `object` |

---

### `POST /mcp`

- **OperationId**: `mcp.add`
- **摘要**: Add MCP server
- **说明**: Dynamically add a new Model Context Protocol (MCP) server to the system.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | **必填** |  |
| `config` | McpLocalConfig | McpRemoteConfig | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | MCP server added successfully | `object` |
| 400 | Bad request | `BadRequestError` |

---

### `DELETE /mcp/{name}/auth`

- **OperationId**: `mcp.auth.remove`
- **摘要**: Remove MCP OAuth
- **说明**: Remove OAuth credentials for an MCP server

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `name` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | OAuth credentials removed | `object` |
| 404 | Not found | `NotFoundError` |

---

### `POST /mcp/{name}/auth`

- **OperationId**: `mcp.auth.start`
- **摘要**: Start MCP OAuth
- **说明**: Start OAuth authentication flow for a Model Context Protocol (MCP) server.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `name` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | OAuth flow started | `object` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /mcp/{name}/auth/authenticate`

- **OperationId**: `mcp.auth.authenticate`
- **摘要**: Authenticate MCP OAuth
- **说明**: Start OAuth flow and wait for callback (opens browser)

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `name` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | OAuth authentication completed | `MCPStatus` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /mcp/{name}/auth/callback`

- **OperationId**: `mcp.auth.callback`
- **摘要**: Complete MCP OAuth
- **说明**: Complete OAuth authentication for a Model Context Protocol (MCP) server using the authorization code.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `name` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `code` | string | **必填** | Authorization code from OAuth callback |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | OAuth authentication completed | `MCPStatus` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /mcp/{name}/connect`

- **OperationId**: `mcp.connect`
- **摘要**: 
- **说明**: Connect an MCP server

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `name` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | MCP server connected successfully | `boolean` |

---

### `POST /mcp/{name}/disconnect`

- **OperationId**: `mcp.disconnect`
- **摘要**: 
- **说明**: Disconnect an MCP server

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `name` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | MCP server disconnected successfully | `boolean` |

---
