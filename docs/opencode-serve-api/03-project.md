# Project 项目

> [← 返回目录](README.md)

---

## 关于「创建项目」

**当前 API 不提供「创建项目」接口**（无 `POST /project`）。

- **项目如何出现**：项目由 OpenCode 在**打开或使用某工作目录时自动识别/注册**。例如在该目录下运行 `opencode` 或 `opencode serve`，或通过 API 以该目录为 `directory` 进行操作时，对应项目会出现在 `GET /project` 列表中。
- **可用的项目接口**：仅支持**列出**（`GET /project`）、**获取当前**（`GET /project/current`）、**更新**（`PATCH /project/{projectID}`，如 name、icon、commands）。若需“新项目”，实际是**先在该目录下使用 OpenCode，再由 API 查询或更新**。

---

### `GET /project`

- **OperationId**: `project.list`
- **摘要**: List all projects
- **说明**: Get a list of projects that have been opened with OpenCode.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of projects | `array` |

---

### `GET /project/current`

- **OperationId**: `project.current`
- **摘要**: Get current project
- **说明**: Retrieve the currently active project that OpenCode is working with.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Current project information | `Project` |

---

### `PATCH /project/{projectID}`

- **OperationId**: `project.update`
- **摘要**: Update project
- **说明**: Update project properties such as name, icon, and commands.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `projectID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 可选 |  |
| `icon` | object | 可选 |  |
| `commands` | object | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Updated project information | `Project` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---
