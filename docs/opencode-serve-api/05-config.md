# Config 配置

> [← 返回目录](README.md)

---

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
