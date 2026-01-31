# Command 命令

> [← 返回目录](README.md)

---

### `GET /command`

- **OperationId**: `command.list`
- **摘要**: List commands
- **说明**: Get a list of all available commands in the OpenCode system.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of commands | `array` |

---
