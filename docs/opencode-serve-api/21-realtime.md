# 实时接口

> [← 返回目录](README.md)

OpenCode Serve API 提供多种实时能力，支持事件推送、流式响应和双向通信。

---

## 概览

| 类型 | 接口 | 用途 |
|------|------|------|
| **SSE** | `GET /global/event` | 全局事件流（服务级） |
| **SSE** | `GET /event` | 实例级事件流（需 `directory`） |
| **WebSocket** | `GET /pty/{ptyID}/connect` | PTY 伪终端实时交互 |
| **流式 HTTP** | `POST /session/{sessionID}/message` | 流式返回 AI 回复 |

---

## 1. SSE 事件流

### `GET /global/event`

- **OperationId**: `global.event`
- **说明**: 订阅 OpenCode 全局事件（如实例创建/销毁等）
- **响应**: Server-Sent Events 流，类型 `GlobalEvent`
- **文档**: [01-global.md](01-global.md)

### `GET /event`

- **OperationId**: `event.subscribe`
- **说明**: 订阅当前实例的服务端事件
- **请求参数**: `directory`（可选，指定工作目录）
- **响应**: Server-Sent Events 流，类型 `Event`
- **文档**: [17-event.md](17-event.md)

**使用场景**：监听会话状态变化、消息更新、权限请求等，实现 UI 实时刷新。

---

## 2. WebSocket

### `GET /pty/{ptyID}/connect`

- **OperationId**: `pty.connect`
- **说明**: 建立 WebSocket 连接，与 PTY 伪终端会话实时交互
- **请求参数**: `directory`（可选）、`ptyID`（路径参数，必填）
- **文档**: [18-pty.md](18-pty.md)

**使用流程**：先通过 `POST /pty` 创建 PTY 会话，再调用本接口建立 WebSocket 连接，用于 shell 输入输出双向通信。

---

## 3. 流式 AI 响应

### `POST /session/{sessionID}/message`

- **OperationId**: `session.prompt`
- **说明**: 发送消息到会话，**以流式方式返回 AI 回复**
- **文档**: [08-session.md](08-session.md)

响应为流式 HTTP，客户端可边收边渲染，实现打字机效果。

---

## 组合使用建议

实现完整实时对话体验可组合使用：

1. **发送消息**：`POST /session/{sessionID}/message` 获取流式 AI 回复
2. **订阅事件**：`GET /event` 或 `GET /global/event` 监听会话、消息、权限等变化
3. **终端交互**：`GET /pty/{ptyID}/connect` 进行 PTY 实时输入输出
