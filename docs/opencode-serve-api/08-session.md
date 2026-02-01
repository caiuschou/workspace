# Session 会话 / Message 消息

> [← 返回目录](README.md)

---

## Session 是什么

在 OpenCode Serve API 中，**Session（会话）** 是与 AI 助手进行多轮对话的**基本工作单元**。

- **用途**：一次会话对应一段连续的对话上下文，包含用户的多条输入（prompt）和 AI 的多条回复（message）。创建会话、在会话内发消息、查看/回滚消息、总结会话等，都通过本模块的接口完成。
- **生命周期**：通过 `POST /session` 创建；通过 `GET /session`、`GET /session/status` 查询列表与状态；通过 `DELETE /session/{sessionID}` 删除。会话有 **active（进行中）**、**idle（空闲）**、**completed（已完成）** 等状态。
- **与项目的关系**：会话通常绑定到某个工作目录（`directory`），对应一个项目下的对话；可在同一项目下创建多个会话，用于不同任务或分支（例如通过 fork 从某条消息分叉出新会话）。
- **Message（消息）**：会话内的每条用户或 AI 内容是一条 **Message**；每条 Message 可包含多个 **Part**（文本、文件、工具调用等）。发送消息用 `POST /session/{id}/message` 或异步的 `POST /session/{id}/prompt_async`，查询用 `GET /session/{id}/message`。
- **常见操作**：初始化会话（生成 AGENTS.md）、fork、总结（summarize）、回滚（revert）、分享（share）、权限回复（permissions）等，均以会话为粒度，通过本页下方接口完成。

---

### `GET /session`

- **OperationId**: `session.list`
- **摘要**: List sessions
- **说明**: Get a list of all OpenCode sessions, sorted by most recently updated.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 | Filter sessions by project directory |
| query | `roots` | boolean | 可选 | Only return root sessions (no parentID) |
| query | `start` | number | 可选 | Filter sessions updated on or after this timestamp (milliseconds since epoch) |
| query | `search` | string | 可选 | Filter sessions by title (case-insensitive) |
| query | `limit` | number | 可选 | Maximum number of sessions to return |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of sessions | `array` |

---

### `POST /session`

- **OperationId**: `session.create`
- **摘要**: Create session
- **说明**: Create a new OpenCode session for interacting with AI assistants and managing conversations.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `parentID` | string | 可选 | pattern: `^ses.*` |
| `title` | string | 可选 |  |
| `permission` | PermissionRuleset | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully created session | `Session` |
| 400 | Bad request | `BadRequestError` |

---

### `GET /session/status`

- **OperationId**: `session.status`
- **摘要**: Get session status
- **说明**: Retrieve the current status of all sessions, including active, idle, and completed states.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Get session status | `object` |
| 400 | Bad request | `BadRequestError` |

---

### `DELETE /session/{sessionID}`

- **OperationId**: `session.delete`
- **摘要**: Delete session
- **说明**: Delete a session and permanently remove all associated data, including messages and history.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully deleted session | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `GET /session/{sessionID}`

- **OperationId**: `session.get`
- **摘要**: Get session
- **说明**: Retrieve detailed information about a specific OpenCode session.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Get session | `Session` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `PATCH /session/{sessionID}`

- **OperationId**: `session.update`
- **摘要**: Update session
- **说明**: Update properties of an existing session, such as title or other metadata.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `title` | string | 可选 |  |
| `time` | object | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully updated session | `Session` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/abort`

- **OperationId**: `session.abort`
- **摘要**: Abort session
- **说明**: Abort an active session and stop any ongoing AI processing or command execution.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Aborted session | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `GET /session/{sessionID}/children`

- **OperationId**: `session.children`
- **摘要**: Get session children
- **说明**: Retrieve all child sessions that were forked from the specified parent session.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of children | `array` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/command`

- **OperationId**: `session.command`
- **摘要**: Send command
- **说明**: Send a new command to a session for execution by the AI assistant.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `messageID` | string | 可选 | pattern: `^msg.*` |
| `agent` | string | 可选 |  |
| `model` | string | 可选 |  |
| `arguments` | string | **必填** |  |
| `command` | string | **必填** |  |
| `variant` | string | 可选 |  |
| `parts` | array | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Created message | `object` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `GET /session/{sessionID}/diff`

- **OperationId**: `session.diff`
- **摘要**: Get message diff
- **说明**: Get the file changes (diff) that resulted from a specific user message in the session.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |
| query | `messageID` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully retrieved diff | `array` |

---

### `POST /session/{sessionID}/fork`

- **OperationId**: `session.fork`
- **摘要**: Fork session
- **说明**: Create a new session by forking an existing session at a specific message point.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `messageID` | string | 可选 | pattern: `^msg.*` |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | 200 | `Session` |

---

### `POST /session/{sessionID}/init`

- **OperationId**: `session.init`
- **摘要**: Initialize session
- **说明**: Analyze the current application and create an AGENTS.md file with project-specific agent configurations.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `modelID` | string | **必填** |  |
| `providerID` | string | **必填** |  |
| `messageID` | string | **必填** | pattern: `^msg.*` |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | 200 | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `GET /session/{sessionID}/message`

- **OperationId**: `session.messages`
- **摘要**: Get session messages
- **说明**: Retrieve all messages in a session, including user prompts and AI responses.

**兼容性**：部分 OpenCode 版本使用 `GET /session/{sessionID}/messages`（复数）作为路径。客户端可依次尝试两者以兼容不同版本。

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 | 指定工作目录 |
| path | `sessionID` | string | **必填** | Session ID |
| query | `limit` | number | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of messages | `array` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/message`

- **OperationId**: `session.prompt`
- **摘要**: Send message
- **说明**: Create and send a new message to a session, streaming the AI response.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `messageID` | string | 可选 | pattern: `^msg.*` |
| `model` | object | 可选 |  |
| `agent` | string | 可选 |  |
| `noReply` | boolean | 可选 |  |
| `tools` | object | 可选 | @deprecated tools and permissions have been merged, you can set permissions on the session itself now |
| `system` | string | 可选 |  |
| `variant` | string | 可选 |  |
| `parts` | array | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Created message | `object` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `GET /session/{sessionID}/message/{messageID}`

- **OperationId**: `session.message`
- **摘要**: Get message
- **说明**: Retrieve a specific message from a session by its message ID.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |
| path | `messageID` | string | **必填** | Message ID |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Message | `object` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `DELETE /session/{sessionID}/message/{messageID}/part/{partID}`

- **OperationId**: `part.delete`
- **摘要**: 
- **说明**: Delete a part from a message

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |
| path | `messageID` | string | **必填** | Message ID |
| path | `partID` | string | **必填** | Part ID |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully deleted part | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `PATCH /session/{sessionID}/message/{messageID}/part/{partID}`

- **OperationId**: `part.update`
- **摘要**: 
- **说明**: Update a part in a message

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |
| path | `messageID` | string | **必填** | Message ID |
| path | `partID` | string | **必填** | Part ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully updated part | `Part` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/permissions/{permissionID}`

- **OperationId**: `permission.respond`
- **摘要**: Respond to permission
- **说明**: Approve or deny a permission request from the AI assistant.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |
| path | `permissionID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `response` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Permission processed successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/prompt_async`

- **OperationId**: `session.prompt_async`
- **摘要**: Send async message
- **说明**: Create and send a new message to a session asynchronously, starting the session if needed and returning immediately.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `messageID` | string | 可选 | pattern: `^msg.*` |
| `model` | object | 可选 |  |
| `agent` | string | 可选 |  |
| `noReply` | boolean | 可选 |  |
| `tools` | object | 可选 | @deprecated tools and permissions have been merged, you can set permissions on the session itself now |
| `system` | string | 可选 |  |
| `variant` | string | 可选 |  |
| `parts` | array | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 204 | Prompt accepted | - |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/revert`

- **OperationId**: `session.revert`
- **摘要**: Revert message
- **说明**: Revert a specific message in a session, undoing its effects and restoring the previous state.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `messageID` | string | **必填** | pattern: `^msg.*` |
| `partID` | string | 可选 | pattern: `^prt.*` |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Updated session | `Session` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `DELETE /session/{sessionID}/share`

- **OperationId**: `session.unshare`
- **摘要**: Unshare session
- **说明**: Remove the shareable link for a session, making it private again.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully unshared session | `Session` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/share`

- **OperationId**: `session.share`
- **摘要**: Share session
- **说明**: Create a shareable link for a session, allowing others to view the conversation.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully shared session | `Session` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/shell`

- **OperationId**: `session.shell`
- **摘要**: Run shell command
- **说明**: Execute a shell command within the session context and return the AI's response.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `agent` | string | **必填** |  |
| `model` | object | 可选 |  |
| `command` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Created message | `AssistantMessage` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/summarize`

- **OperationId**: `session.summarize`
- **摘要**: Summarize session
- **说明**: Generate a concise summary of the session using AI compaction to preserve key information.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `providerID` | string | **必填** |  |
| `modelID` | string | **必填** |  |
| `auto` | boolean | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Summarized session | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `GET /session/{sessionID}/todo`

- **OperationId**: `session.todo`
- **摘要**: Get session todos
- **说明**: Retrieve the todo list associated with a specific session, showing tasks and action items.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** | Session ID |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Todo list | `array` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /session/{sessionID}/unrevert`

- **OperationId**: `session.unrevert`
- **摘要**: Restore reverted messages
- **说明**: Restore all previously reverted messages in a session.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `sessionID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Updated session | `Session` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---
