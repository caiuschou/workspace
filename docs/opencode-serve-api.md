# OpenCode Serve API 接口详细文档（汇总）

> 本文件由脚本生成。**按模块查找请使用 [opencode-serve-api/README.md](opencode-serve-api/README.md) 目录。**

---

## Global 全局

### `POST /global/dispose`

- **OperationId**: `global.dispose`
- **摘要**: Dispose instance
- **说明**: Clean up and dispose all OpenCode instances, releasing all resources.

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Global disposed | `boolean` |

---

### `GET /global/event`

- **OperationId**: `global.event`
- **摘要**: Get global events
- **说明**: Subscribe to global events from the OpenCode system using server-sent events.

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Event stream | `GlobalEvent` |

---

### `GET /global/health`

- **OperationId**: `global.health`
- **摘要**: Get health
- **说明**: Get health information about the OpenCode server.

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Health information | `object` |

---

## Instance 实例

### `POST /instance/dispose`

- **OperationId**: `instance.dispose`
- **摘要**: Dispose instance
- **说明**: Clean up and dispose the current OpenCode instance, releasing all resources.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Instance disposed | `boolean` |

---

## Project 项目

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

## Path & VCS

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

## Config 配置

### `GET /config`

- **OperationId**: `config.get`
- **摘要**: Get configuration
- **说明**: Retrieve the current OpenCode configuration settings and preferences.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Get config info | `Config` |

---

### `PATCH /config`

- **OperationId**: `config.update`
- **摘要**: Update configuration
- **说明**: Update OpenCode configuration settings and preferences.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `$schema` | string | 可选 | JSON schema reference for configuration validation |
| `theme` | string | 可选 | Theme name to use for the interface |
| `keybinds` | KeybindsConfig | 可选 |  |
| `logLevel` | LogLevel | 可选 |  |
| `tui` | object | 可选 | TUI specific settings |
| `server` | ServerConfig | 可选 |  |
| `command` | object | 可选 | Command configuration, see https://opencode.ai/docs/commands |
| `watcher` | object | 可选 |  |
| `plugin` | array | 可选 |  |
| `snapshot` | boolean | 可选 |  |
| `share` | string | 可选 | Control sharing behavior:'manual' allows manual sharing via commands, 'auto' enables automatic sharing, 'disabled' disables all sharing |
| `autoshare` | boolean | 可选 | @deprecated Use 'share' field instead. Share newly created sessions automatically |
| `autoupdate` | anyOf | 可选 | Automatically update to the latest version. Set to true to auto-update, false to disable, or 'notify' to show update notifications |
| `disabled_providers` | array | 可选 | Disable providers that are loaded automatically |
| `enabled_providers` | array | 可选 | When set, ONLY these providers will be enabled. All other providers will be ignored |
| `model` | string | 可选 | Model to use in the format of provider/model, eg anthropic/claude-2 |
| `small_model` | string | 可选 | Small model to use for tasks like title generation in the format of provider/model |
| `default_agent` | string | 可选 | Default agent to use when none is specified. Must be a primary agent. Falls back to 'build' if not set or if the specified agent is invalid. |
| `username` | string | 可选 | Custom username to display in conversations instead of system username |
| `mode` | object | 可选 | @deprecated Use `agent` field instead. |
| `agent` | object | 可选 | Agent configuration, see https://opencode.ai/docs/agents |
| `provider` | object | 可选 | Custom provider configurations and model overrides |
| `mcp` | object | 可选 | MCP (Model Context Protocol) server configurations |
| `formatter` | anyOf | 可选 |  |
| `lsp` | anyOf | 可选 |  |
| `instructions` | array | 可选 | Additional instruction files or patterns to include |
| `layout` | LayoutConfig | 可选 |  |
| `permission` | PermissionConfig | 可选 |  |
| `tools` | object | 可选 |  |
| `enterprise` | object | 可选 |  |
| `compaction` | object | 可选 |  |
| `experimental` | object | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully updated config | `Config` |
| 400 | Bad request | `BadRequestError` |

---

### `GET /config/providers`

- **OperationId**: `config.providers`
- **摘要**: List config providers
- **说明**: Get a list of all configured AI providers and their default models.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of providers | `object` |

---

## Provider 模型提供商

### `GET /provider`

- **OperationId**: `provider.list`
- **摘要**: List providers
- **说明**: Get a list of all available AI providers, including both available and connected ones.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of providers | `object` |

---

### `GET /provider/auth`

- **OperationId**: `provider.auth`
- **摘要**: Get provider auth methods
- **说明**: Retrieve available authentication methods for all AI providers.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Provider auth methods | `object` |

---

### `POST /provider/{providerID}/oauth/authorize`

- **OperationId**: `provider.oauth.authorize`
- **摘要**: OAuth authorize
- **说明**: Initiate OAuth authorization for a specific AI provider to get an authorization URL.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `providerID` | string | **必填** | Provider ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `method` | number | **必填** | Auth method index |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Authorization URL and method | `ProviderAuthAuthorization` |
| 400 | Bad request | `BadRequestError` |

---

### `POST /provider/{providerID}/oauth/callback`

- **OperationId**: `provider.oauth.callback`
- **摘要**: OAuth callback
- **说明**: Handle the OAuth callback from a provider after user authorization.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `providerID` | string | **必填** | Provider ID |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `method` | number | **必填** | Auth method index |
| `code` | string | 可选 | OAuth authorization code |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | OAuth callback processed successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |

---

## Auth 认证

### `PUT /auth/{providerID}`

- **OperationId**: `auth.set`
- **摘要**: Set auth credentials
- **说明**: Set authentication credentials

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `providerID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Successfully set authentication credentials | `boolean` |
| 400 | Bad request | `BadRequestError` |

---

## Session 会话 / Message 消息

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

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
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

## Permission 权限

### `GET /permission`

- **OperationId**: `permission.list`
- **摘要**: List pending permissions
- **说明**: Get all pending permission requests across all sessions.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of pending permissions | `array` |

---

### `POST /permission/{requestID}/reply`

- **OperationId**: `permission.reply`
- **摘要**: Respond to permission request
- **说明**: Approve or deny a permission request from the AI assistant.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `requestID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `reply` | string | **必填** |  |
| `message` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Permission processed successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

## Question 问题

### `GET /question`

- **OperationId**: `question.list`
- **摘要**: List pending questions
- **说明**: Get all pending question requests across all sessions.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | List of pending questions | `array` |

---

### `POST /question/{requestID}/reject`

- **OperationId**: `question.reject`
- **摘要**: Reject question request
- **说明**: Reject a question request from the AI assistant.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `requestID` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Question rejected successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /question/{requestID}/reply`

- **OperationId**: `question.reply`
- **摘要**: Reply to question request
- **说明**: Provide answers to a question request from the AI assistant.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |
| path | `requestID` | string | **必填** |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `answers` | array<QuestionAnswer> | **必填** | User answers in order of questions (each answer is an array of selected labels) |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Question answered successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

## Command 命令

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

## File 文件

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

## Find 查找

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

## LSP / Formatter / MCP

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

## Agent & Skill

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

## Logging 日志

### `POST /log`

- **OperationId**: `app.log`
- **摘要**: Write log
- **说明**: Write a log entry to the server logs with specified level and metadata.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `service` | string | **必填** | Service name for the log entry |
| `level` | string | **必填** | Log level |
| `message` | string | **必填** | Log message |
| `extra` | object | 可选 | Additional metadata for the log entry |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Log entry written successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |

---

## Event 事件

### `GET /event`

- **OperationId**: `event.subscribe`
- **摘要**: Subscribe to events
- **说明**: Get events

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Event stream | `Event` |

---

## PTY 伪终端

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

## TUI 界面控制

### `POST /tui/append-prompt`

- **OperationId**: `tui.appendPrompt`
- **摘要**: Append TUI prompt
- **说明**: Append prompt to the TUI

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `text` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Prompt processed successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |

---

### `POST /tui/clear-prompt`

- **OperationId**: `tui.clearPrompt`
- **摘要**: Clear TUI prompt
- **说明**: Clear the prompt

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Prompt cleared successfully | `boolean` |

---

### `GET /tui/control/next`

- **OperationId**: `tui.control.next`
- **摘要**: Get next TUI request
- **说明**: Retrieve the next TUI (Terminal User Interface) request from the queue for processing.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Next TUI request | `object` |

---

### `POST /tui/control/response`

- **OperationId**: `tui.control.response`
- **摘要**: Submit TUI response
- **说明**: Submit a response to the TUI request queue to complete a pending request.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Response submitted successfully | `boolean` |

---

### `POST /tui/execute-command`

- **OperationId**: `tui.executeCommand`
- **摘要**: Execute TUI command
- **说明**: Execute a TUI command (e.g. agent_cycle)

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `command` | string | **必填** |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Command executed successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |

---

### `POST /tui/open-help`

- **OperationId**: `tui.openHelp`
- **摘要**: Open help dialog
- **说明**: Open the help dialog in the TUI to display user assistance information.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Help dialog opened successfully | `boolean` |

---

### `POST /tui/open-models`

- **OperationId**: `tui.openModels`
- **摘要**: Open models dialog
- **说明**: Open the model dialog

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Model dialog opened successfully | `boolean` |

---

### `POST /tui/open-sessions`

- **OperationId**: `tui.openSessions`
- **摘要**: Open sessions dialog
- **说明**: Open the session dialog

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Session dialog opened successfully | `boolean` |

---

### `POST /tui/open-themes`

- **OperationId**: `tui.openThemes`
- **摘要**: Open themes dialog
- **说明**: Open the theme dialog

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Theme dialog opened successfully | `boolean` |

---

### `POST /tui/publish`

- **OperationId**: `tui.publish`
- **摘要**: Publish TUI event
- **说明**: Publish a TUI event

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Event published successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |

---

### `POST /tui/select-session`

- **OperationId**: `tui.selectSession`
- **摘要**: Select session
- **说明**: Navigate the TUI to display the specified session.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `sessionID` | string | **必填** | Session ID to navigate to (pattern: `^ses`) |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Session selected successfully | `boolean` |
| 400 | Bad request | `BadRequestError` |
| 404 | Not found | `NotFoundError` |

---

### `POST /tui/show-toast`

- **OperationId**: `tui.showToast`
- **摘要**: Show TUI toast
- **说明**: Show a toast notification in the TUI

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**请求体** (`application/json`)

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `title` | string | 可选 |  |
| `message` | string | **必填** |  |
| `variant` | string | **必填** |  |
| `duration` | number | 可选 | Duration in milliseconds |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Toast notification shown successfully | `boolean` |

---

### `POST /tui/submit-prompt`

- **OperationId**: `tui.submitPrompt`
- **摘要**: Submit TUI prompt
- **说明**: Submit the prompt

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Prompt submitted successfully | `boolean` |

---

## Experimental 实验性

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
