# Experimental 实验性

> [← 返回目录](README.md)

---

### `GET /experimental/resource`

- **OperationId**: `experimental.resource.list`
- **摘要**: Get MCP resources
- **说明**: Get all available MCP resources from connected servers. Optionally filter by name.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | MCP resources | `object` |

---

### `GET /experimental/tool`

- **OperationId**: `tool.list`
- **摘要**: List tools
- **说明**: Get a list of available tools with their JSON schema parameters for a specific provider and model combination.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| query | `provider` | string | **必填** |  |
| query | `model` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Tools | `ToolList` |
| 400 | Bad request | `BadRequestError` |

---

### `GET /experimental/tool/ids`

- **OperationId**: `tool.ids`
- **摘要**: List tool IDs
- **说明**: Get a list of all available tool IDs, including both built-in tools and dynamically registered tools.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Tool IDs | `ToolIDs` |
| 400 | Bad request | `BadRequestError` |

---

### `DELETE /experimental/worktree`

- **OperationId**: `worktree.remove`
- **摘要**: Remove worktree
- **说明**: Remove a git worktree and delete its branch.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `directory` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Worktree removed | `boolean` |
| 400 | Bad request | `BadRequestError` |

---

### `GET /experimental/worktree`

- **OperationId**: `worktree.list`
- **摘要**: List worktrees
- **说明**: List all sandbox worktrees for the current project.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of worktree directories | `array` |

---

### `POST /experimental/worktree`

- **OperationId**: `worktree.create`
- **摘要**: Create worktree
- **说明**: Create a new git worktree for the current project and run any configured startup scripts.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 可选 |  |
| `startCommand` | string | 可选 | Additional startup script to run after the project's start command |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Worktree created | `Worktree` |
| 400 | Bad request | `BadRequestError` |

---

### `POST /experimental/worktree/reset`

- **OperationId**: `worktree.reset`
- **摘要**: Reset worktree
- **说明**: Reset a worktree branch to the primary default branch.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `directory` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Worktree reset | `boolean` |
| 400 | Bad request | `BadRequestError` |

---
