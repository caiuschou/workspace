# OpenCode Server Event 详细格式

> 基于 `GET /event` SSE 流实际日志整理。  
> [← 返回目录](README.md) | [Event API](17-event.md)

---

## 1. 通用结构

每条 SSE 事件的 `data` 为 JSON 对象，顶层字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `type` | string | 事件类型，见下表 |
| `properties` | object | 事件载荷，随 `type` 不同而不同 |

部分事件在 `properties` 内包含 `sessionID` / `sessionId`，用于区分会话。

---

## 2. 事件类型一览

| type | 说明 |
|------|------|
| `server.connected` | 客户端连接上 SSE 时 |
| `server.heartbeat` | 服务端定时心跳（约 30 秒） |
| `session.updated` | 会话元数据更新 |
| `session.status` | 会话状态变更（busy / idle） |
| `session.diff` | 会话 diff 变更 |
| `session.idle` | 会话进入空闲（本轮回复结束） |
| `message.updated` | 某条消息元数据更新 |
| `message.part.updated` | 某条消息的某一 part 更新（含流式 text/reasoning 增量） |

其他可能出现的类型（如 LSP）：`lsp.client.diagnostics` 等。

---

## 3. server.connected

客户端首次连接时发送。

```json
{
  "type": "server.connected",
  "properties": {}
}
```

---

## 4. server.heartbeat

保持连接用，无业务载荷。

```json
{
  "type": "server.heartbeat",
  "properties": { ... }
}
```

---

## 5. session.updated

会话信息变更（创建、标题/目录/摘要等更新）。

```json
{
  "type": "session.updated",
  "properties": {
    "info": {
      "id": "ses_xxx",
      "directory": "/path/to/project",
      "projectID": "global",
      "slug": "mighty-pixel",
      "title": "New session - 2026-02-01T03:25:09.384Z",
      "version": "1.1.36",
      "time": {
        "created": 1769916309384,
        "updated": 1769916309389
      },
      "summary": {
        "additions": 0,
        "deletions": 0,
        "files": 0
      }
    }
  }
}
```

---

## 6. session.status

会话状态：`busy`（处理中）或 `idle`（空闲）。

```json
{
  "type": "session.status",
  "properties": {
    "sessionID": "ses_xxx",
    "status": {
      "type": "busy"
    }
  }
}
```

`status.type` 为 `"idle"` 时表示当前会话本轮处理结束，可视为「完成」信号。

---

## 7. session.diff

会话维度的 diff 列表（文件变更摘要等）。

```json
{
  "type": "session.diff",
  "properties": {
    "sessionID": "ses_xxx",
    "diff": []
  }
}
```

---

## 8. session.idle

会话进入空闲，与 `session.status` 且 `status.type === "idle"` 语义一致，常用于判断「本轮回复结束」。

```json
{
  "type": "session.idle",
  "properties": {
    "sessionID": "ses_xxx"
  }
}
```

**SDK 完成检测建议**：收到 `type === "session.idle"` 或 `session.status` 且 `properties.status.type === "idle"` 且 `sessionID` 匹配时，可认为该会话本 turn 已完成。

---

## 9. message.updated

某条消息的元数据更新（不包含 part 内容流）。

**用户消息示例：**

```json
{
  "type": "message.updated",
  "properties": {
    "info": {
      "id": "msg_xxx",
      "role": "user",
      "sessionID": "ses_xxx",
      "agent": "build",
      "model": { "modelID": "big-pickle", "providerID": "opencode" },
      "time": { "created": 1769916309387 },
      "summary": { "diffs": [], "title": "Python 斐波那契数列代码需求" }
    }
  }
}
```

**Assistant 消息示例（进行中）：**

```json
{
  "type": "message.updated",
  "properties": {
    "info": {
      "id": "msg_yyy",
      "role": "assistant",
      "sessionID": "ses_xxx",
      "agent": "build",
      "mode": "build",
      "parentID": "msg_xxx",
      "modelID": "big-pickle",
      "providerID": "opencode",
      "path": { "cwd": "/tmp/example1", "root": "/" },
      "time": { "created": 1769916309390 },
      "tokens": {
        "input": 0,
        "output": 0,
        "reasoning": 0,
        "cache": { "read": 0, "write": 0 }
      }
    }
  }
}
```

**Assistant 消息完成示例：**

```json
{
  "type": "message.updated",
  "properties": {
    "info": {
      "id": "msg_yyy",
      "role": "assistant",
      "sessionID": "ses_xxx",
      "agent": "build",
      "mode": "build",
      "finish": "stop",
      "time": { "created": 1769916309390 },
      "tokens": {
        "input": 1,
        "output": 81,
        "reasoning": 1,
        "cache": { "read": 11299, "write": 0 }
      }
    }
  }
}
```

`info.finish` 存在（如 `"stop"`）表示该条 assistant 消息已结束。

---

## 10. message.part.updated

某条消息的**某一个 part** 的更新，用于流式输出文本/推理等。

顶层 `properties` 常见字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `part` | object | part 内容与元数据 |
| `delta` | string | 可选，本事件对应的**增量**文本（流式时使用） |

### 10.1 part 对象常见字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | part ID |
| `messageID` | string | 所属消息 ID |
| `sessionID` | string | 所属会话 ID |
| `type` | string | part 类型：见下表 |
| `text` | string | 当前累积文本（或本段内容） |
| `time` | object | 可选，`start` / `end` 时间戳（毫秒） |
| `delta` | string | 可选，仅在本事件中表示增量 |

### 10.2 part.type 取值

| part.type | 说明 |
|-----------|------|
| `text` | 普通文本；流式时多次 `message.part.updated`，`delta` 为增量，`text` 为累积内容 |
| `reasoning` | 推理过程（可能流式） |
| `step-start` | 某一步开始 |
| `step-finish` | 某一步结束，含 `reason`、`tokens` 等 |

### 10.3 流式文本增量示例

```json
{
  "type": "message.part.updated",
  "properties": {
    "delta": "def ",
    "part": {
      "id": "prt_xxx",
      "messageID": "msg_yyy",
      "sessionID": "ses_xxx",
      "type": "text",
      "text": "\n```python\ndef ",
      "time": { "start": 1769916310623 }
    }
  }
}
```

- **流式展示**：用 `properties.delta` 或 `properties.part.text` 的增量即可。
- **会话归属**：用 `properties.part.sessionID` 或 `properties.sessionID` 过滤当前会话。

### 10.4 用户消息 part（非流式）

```json
{
  "type": "message.part.updated",
  "properties": {
    "part": {
      "id": "prt_xxx",
      "messageID": "msg_xxx",
      "sessionID": "ses_xxx",
      "type": "text",
      "text": "写一个 python 的斐波那契数列的代码"
    }
  }
}
```

### 10.5 step-finish（一步结束）

```json
{
  "type": "message.part.updated",
  "properties": {
    "part": {
      "id": "prt_xxx",
      "messageID": "msg_yyy",
      "sessionID": "ses_xxx",
      "type": "step-finish",
      "reason": "stop",
      "cost": 0,
      "tokens": {
        "input": 1,
        "output": 81,
        "reasoning": 1,
        "cache": { "read": 11299, "write": 0 }
      }
    }
  }
}
```

**完成检测**：同一会话下收到 `part.type === "step-finish"` 或 `message.updated` 且 `info.finish` 存在，再结合 `session.idle` / `session.status`（idle），可判断本轮回复完成。

---

## 11. 完成检测小结（SDK 用）

| 策略 | 条件 |
|------|------|
| 推荐 | `type === "session.idle"` 且 `properties.sessionID` 匹配当前会话 |
| 备选 | `type === "session.status"` 且 `properties.status.type === "idle"` 且 sessionID 匹配 |
| 备选 | `type === "message.part.updated"` 且 `properties.part.type === "step-finish"` 且 sessionID 匹配 |
| 备选 | `type === "message.updated"` 且 `properties.info.finish` 存在且 sessionID 匹配 |

收到任一带 sessionID 的完成信号后，可拉取一次 `GET /session/{id}/message` 获取完整消息列表与最后一条 assistant 消息。

---

## 12. 日志文件路径

opencode-sdk 写入的日志文件默认路径见 [opencode-sdk 日志配置](../../opencode-sdk/README.md)：

- 默认：`$HOME/.local/share/opencode-sdk/opencode-sdk.log`（或 `$XDG_DATA_HOME/opencode-sdk/opencode-sdk.log`）
- 可通过 `init_logger(Some(dir))` 指定目录，文件名为 `opencode-sdk.log`

日志中若开启 `event full payload` 打印，可看到与上述结构一致的 JSON，便于对照本格式文档。
