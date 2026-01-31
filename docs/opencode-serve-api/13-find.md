# Find 查找

> [← 返回目录](README.md)

---

### `GET /find`

- **OperationId**: `find.text`
- **摘要**: Find text
- **说明**: Search for text patterns across files in the project using ripgrep.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| query | `pattern` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Matches | `array` |

---

### `GET /find/file`

- **OperationId**: `find.files`
- **摘要**: Find files
- **说明**: Search for files or directories by name or pattern in the project directory.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| query | `query` | string | **必填** |  |
| query | `dirs` | string | 可选 |  |
| query | `type` | string | 可选 |  |
| query | `limit` | integer | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | File paths | `array` |

---

### `GET /find/symbol`

- **OperationId**: `find.symbols`
- **摘要**: Find symbols
- **说明**: Search for workspace symbols like functions, classes, and variables using LSP.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| query | `query` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Symbols | `array` |

---
