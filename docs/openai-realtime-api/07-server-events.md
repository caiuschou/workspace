# 服务端事件参考

本文档列出了所有从服务端发送到客户端的事件。

## 事件概览

| 事件类型 | 类别 | 说明 |
|---------|------|------|
| `session.created` | 会话 | 会话创建 |
| `session.updated` | 会话 | 会话更新完成 |
| `conversation.created` | 对话 | 对话创建 |
| `conversation.item.created` | 对话 | 对话项创建 |
| `conversation.item.deleted` | 对话 | 对话项删除 |
| `conversation.item.truncated` | 对话 | 对话项截断 |
| `input_audio_buffer.committed` | 音频 | 音频缓冲区已提交 |
| `input_audio_buffer.cleared` | 音频 | 音频缓冲区已清空 |
| `input_audio_buffer.speech_started` | 音频 | 检测到语音开始 |
| `input_audio_buffer.speech_stopped` | 音频 | 检测到语音停止 |
| `response.created` | 响应 | 响应创建 |
| `response.done` | 响应 | 响应完成 |
| `response.cancelled` | 响应 | 响应已取消 |
| `response.failed` | 响应 | 响应失败 |
| `response.output_item.added` | 响应 | 输出项添加 |
| `response.output_item.done` | 响应 | 输出项完成 |
| `response.content_part.added` | 响应 | 内容部分添加 |
| `response.content_part.done` | 响应 | 内容部分完成 |
| `response.text.delta` | 响应 | 文本增量 |
| `response.text.done` | 响应 | 文本完成 |
| `response.audio.delta` | 响应 | 音频增量 |
| `response.audio.done` | 响应 | 音频完成 |
| `response.audio_transcript.delta` | 响应 | 音频转录增量 |
| `response.audio_transcript.done` | 响应 | 音频转录完成 |
| `rate_limits.updated` | 系统 | 速率限制更新 |
| `error` | 错误 | 错误事件 |

## 通用字段

所有服务端事件都包含以下字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `type` | string | 事件类型 |
| `event_id` | string | 服务端生成的唯一事件 ID |

---

## 会话事件

### session.created

连接建立后自动发送，表示会话已创建。

```json
{
  "type": "session.created",
  "event_id": "evt_001",
  "session": {
    "id": "sess_abc123",
    "object": "realtime.session",
    "model": "gpt-4o-realtime-preview-2024-12-17",
    "modalities": ["text", "audio"],
    "instructions": "A helpful AI assistant.",
    "voice": "alloy",
    "input_audio_format": "pcm16",
    "output_audio_format": "pcm16",
    "input_audio_transcription": null,
    "turn_detection": {
      "type": "server_vad",
      "threshold": 0.5,
      "prefix_padding_ms": 300,
      "silence_duration_ms": 500
    },
    "tools": [],
    "tool_choice": "auto",
    "temperature": 0.8,
    "max_response_output_tokens": null
  }
}
```

### session.updated

`session.update` 配置成功后发送。

```json
{
  "type": "session.updated",
  "event_id": "evt_002",
  "session": {
    "id": "sess_abc123",
    "modalities": ["text"],
    "instructions": "你是一个友好的助手。",
    "temperature": 0.7
  }
}
```

---

## 对话事件

### conversation.created

会话创建时自动发送。

```json
{
  "type": "conversation.created",
  "event_id": "evt_003",
  "conversation": {
    "id": "conv_abc123",
    "object": "realtime.conversation"
  }
}
```

### conversation.item.created

对话项创建时发送。

```json
{
  "type": "conversation.item.created",
  "event_id": "evt_004",
  "previous_item_id": "msg_123",
  "item": {
    "id": "msg_124",
    "type": "message",
    "role": "user",
    "content": [
      {
        "type": "input_text",
        "text": "你好"
      }
    ]
  }
}
```

### conversation.item.deleted

对话项删除成功后发送。

```json
{
  "type": "conversation.item.deleted",
  "event_id": "evt_005",
  "item_id": "msg_123"
}
```

### conversation.item.truncated

对话项截断成功后发送。

```json
{
  "type": "conversation.item.truncated",
  "event_id": "evt_006",
  "item_id": "msg_assist_001",
  "content_index": 0,
  "audio_end_ms": 1500
}
```

---

## 音频缓冲区事件

### input_audio_buffer.committed

音频缓冲区提交后发送。

```json
{
  "type": "input_audio_buffer.committed",
  "event_id": "evt_007",
  "previous_item_id": "msg_123",
  "item_id": "msg_124"
}
```

### input_audio_buffer.cleared

音频缓冲区清空后发送。

```json
{
  "type": "input_audio_buffer.cleared",
  "event_id": "evt_008"
}
```

### input_audio_buffer.speech_started

VAD 检测到语音开始时发送。

```json
{
  "type": "input_audio_buffer.speech_started",
  "event_id": "evt_009",
  "audio_start_ms": 1500,
  "item_id": "msg_user_001"
}
```

### input_audio_buffer.speech_stopped

VAD 检测到语音停止时发送。

```json
{
  "type": "input_audio_buffer.speech_stopped",
  "event_id": "evt_010",
  "audio_end_ms": 5000,
  "item_id": "msg_user_001"
}
```

---

## 响应事件

### response.created

响应创建开始时发送。

```json
{
  "type": "response.created",
  "event_id": "evt_011",
  "response": {
    "id": "resp_001",
    "status": "in_progress",
    "output": []
  }
}
```

### response.done

响应完成时发送，包含完整响应信息。

```json
{
  "type": "response.done",
  "event_id": "evt_012",
  "response": {
    "id": "resp_001",
    "status": "completed",
    "status_details": null,
    "output": [
      {
        "id": "msg_assist_001",
        "type": "message",
        "role": "assistant",
        "content": [
          {
            "type": "text",
            "text": "你好！我是AI助手。"
          }
        ]
      }
    ],
    "usage": {
      "total_tokens": 42,
      "input_tokens": 18,
      "output_tokens": 24,
      "input_token_details": {
        "cached_tokens": 0,
        "text_tokens": 18,
        "audio_tokens": 0
      },
      "output_token_details": {
        "text_tokens": 24,
        "audio_tokens": 0
      }
    }
  }
}
```

#### 状态值

| 状态 | 说明 |
|------|------|
| `completed` | 响应完成 |
| `cancelled` | 响应被取消 |
| `incomplete` | 响应未完成（如达到 token 限制） |
| `failed` | 响应失败 |

### response.cancelled

响应被取消时发送。

```json
{
  "type": "response.cancelled",
  "event_id": "evt_013",
  "response": {
    "id": "resp_001",
    "status": "cancelled",
    "status_details": {
      "type": "turn_detected",
      "reason": "用户开始说话"
    }
  }
}
```

### response.failed

响应失败时发送。

```json
{
  "type": "response.failed",
  "event_id": "evt_014",
  "response": {
    "id": "resp_001",
    "status": "failed",
    "status_details": {
      "type": "content_filter",
      "reason": "内容被安全过滤器拦截"
    }
  }
}
```

### response.output_item.added

输出项添加时发送。

```json
{
  "type": "response.output_item.added",
  "event_id": "evt_015",
  "response_id": "resp_001",
  "output_index": 0,
  "item": {
    "id": "msg_assist_001",
    "type": "message",
    "role": "assistant",
    "content": []
  }
}
```

### response.output_item.done

输出项完成时发送。

```json
{
  "type": "response.output_item.done",
  "event_id": "evt_016",
  "response_id": "resp_001",
  "output_index": 0,
  "item": {
    "id": "msg_assist_001",
    "type": "message",
    "role": "assistant",
    "content": [
      {
        "type": "text",
        "text": "你好！我是AI助手。"
      }
    ]
  }
}
```

### response.content_part.added

内容部分添加时发送。

```json
{
  "type": "response.content_part.added",
  "event_id": "evt_017",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "part": {
    "type": "text",
    "text": ""
  }
}
```

### response.content_part.done

内容部分完成时发送。

```json
{
  "type": "response.content_part.done",
  "event_id": "evt_018",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "part": {
    "type": "text",
    "text": "你好！我是AI助手。"
  }
}
```

### response.text.delta

文本增量事件（流式）。

```json
{
  "type": "response.text.delta",
  "event_id": "evt_019",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "delta": "你好"
}
```

### response.text.done

文本完成时发送。

```json
{
  "type": "response.text.done",
  "event_id": "evt_020",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "text": "你好！我是AI助手。"
}
```

### response.audio.delta

音频增量事件（流式）。

```json
{
  "type": "response.audio.delta",
  "event_id": "evt_021",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "delta": "base64编码的音频数据"
}
```

### response.audio.done

音频完成时发送。

```json
{
  "type": "response.audio.done",
  "event_id": "evt_022",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0
}
```

### response.audio_transcript.delta

音频转录增量事件（流式）。

```json
{
  "type": "response.audio_transcript.delta",
  "event_id": "evt_023",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "delta": "你好"
}
```

### response.audio_transcript.done

音频转录完成时发送。

```json
{
  "type": "response.audio_transcript.done",
  "event_id": "evt_024",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "transcript": "你好！我是AI助手。"
}
```

---

## 其他事件

### rate_limits.updated

速率限制更新时发送。

```json
{
  "type": "rate_limits.updated",
  "event_id": "evt_025",
  "rate_limits": {
    "name": "requests",
    "limit": 100,
    "remaining": 95,
    "reset_seconds": 60
  }
}
```

### error

发生错误时发送。

```json
{
  "type": "error",
  "event_id": "evt_026",
  "error": {
    "type": "invalid_request_error",
    "code": "invalid_value",
    "message": "Invalid value for parameter 'type'",
    "param": "type",
    "event_id": "evt_client_001"
  }
}
```

#### 错误类型

| 类型 | 说明 |
|------|------|
| `invalid_request_error` | 请求参数无效 |
| `authentication_error` | 认证失败 |
| `permission_error` | 权限不足 |
| `rate_limit_error` | 超过速率限制 |
| `api_error` | API 内部错误 |
| `content_filter` | 内容被安全过滤器拦截 |

---

## 文本对话事件流

```
服务端                                              客户端
   │                                                    │
   │<───── conversation.item.create ─────────────────────┤
   │<───── response.create ─────────────────────────────┤
   │                                                    │
   ├───── response.created ───────────────────────────>│
   ├───── conversation.item.created ──────────────────>│
   ├───── response.output_item.added ─────────────────>│
   ├───── response.content_part.added ────────────────>│
   │                                                    │
   ├───── response.text.delta (多次) ─────────────────>│
   │                                                    │
   ├───── response.text.done ─────────────────────────>│
   ├───── response.content_part.done ─────────────────>│
   ├───── response.output_item.done ──────────────────>│
   ├───── response.done ───────────────────────────────>│
```

## 音频对话事件流

```
服务端                                              客户端
   │                                                    │
   │<───── input_audio_buffer.append (多次) ────────────┤
   │                                                    │
   ├───── input_audio_buffer.speech_started ──────────>│
   │                                                    │
   ├───── input_audio_buffer.speech_stopped ──────────>│
   ├───── input_audio_buffer.committed ───────────────>│
   ├───── conversation.item.created ──────────────────>│
   │                                                    │
   ├───── response.created ───────────────────────────>│
   │                                                    │
   ├───── response.audio.delta (多次) ─────────────────>│
   ├───── response.audio_transcript.delta (多次) ──────>│
   │                                                    │
   ├───── response.audio.done ─────────────────────────>│
   ├───── response.audio_transcript.done ─────────────>│
   ├───── response.done ───────────────────────────────>│
```

## 函数调用事件流

```
服务端                                              客户端
   │                                                    │
   │<───── conversation.item.create ─────────────────────┤
   │<───── response.create ─────────────────────────────┤
   │                                                    │
   ├───── response.created ───────────────────────────>│
   │                                                    │
   ├───── response.done (function_call) ──────────────>│
   │                                                    │
   │           [客户端执行函数]                           │
   │                                                    │
   │<───── conversation.item.create (结果) ─────────────┤
   │<───── response.create ─────────────────────────────┤
   │                                                    │
   ├───── response.created ───────────────────────────>│
   ├───── response.done (最终回复) ───────────────────>│
```
