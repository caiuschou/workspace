# Path & VCS

> [← 返回目录](README.md)

---

### `GET /path`

- **OperationId**: `path.get`
- **摘要**: Get paths
- **说明**: Retrieve the current working directory and related path information for the OpenCode instance.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Path | `Path` |

---

### `GET /vcs`

- **OperationId**: `vcs.get`
- **摘要**: Get VCS info
- **说明**: Retrieve version control system (VCS) information for the current project, such as git branch.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | VCS info | `VcsInfo` |

---
