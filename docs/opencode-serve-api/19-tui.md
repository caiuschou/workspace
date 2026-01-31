# TUI 界面控制

> [← 返回目录](README.md)

---

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
