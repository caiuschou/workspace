# File 文件

> [← 返回目录](README.md)

---

### `GET /file`

- **OperationId**: `file.list`
- **摘要**: List files
- **说明**: List files and directories in a specified path.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| query | `path` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Files and directories | `array` |

---

### `GET /file/content`

- **OperationId**: `file.read`
- **摘要**: Read file
- **说明**: Read the content of a specified file.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| query | `path` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | File content | `FileContent` |

---

### `GET /file/status`

- **OperationId**: `file.status`
- **摘要**: Get file status
- **说明**: Get the git status of all files in the project.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | File status | `array` |

---
