# Agent & Skill

> [← 返回目录](README.md)

---

### `GET /agent`

- **OperationId**: `app.agents`
- **摘要**: List agents
- **说明**: Get a list of all available AI agents in the OpenCode system.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of agents | `array` |

---

### `GET /skill`

- **OperationId**: `app.skills`
- **摘要**: List skills
- **说明**: Get a list of all available skills in the OpenCode system.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of skills | `array` |

---
