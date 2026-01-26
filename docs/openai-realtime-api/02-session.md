# 会话管理

本文档说明如何配置和管理 Realtime API 会话。

## 会话概述

Realtime API 会话是**有状态的**，服务端会自动维护对话历史。一个会话包含：

- **Session 对象** - 控制会话参数（模型、声音、配置等）
- **Conversation** - 对话历史（用户消息和助手回复）
- **Response** - 模型生成的响应

## session.update 事件

`session.update` 用于更新会话配置。可以在会话期间的任何时候发送。

### 完整配置示例

```json
{
  "type": "session.update",
  "event_id": "evt_001",
  "session": {
    "modalities": ["text"],
    "instructions": "你是一个友好的助手。",
    "voice": "alloy",
    "input_audio_format": "pcm16",
    "output_audio_format": "pcm16",
    "input_audio_transcription": {
      "model": "whisper-1"
    },
    "turn_detection": {
      "type": "server_vad",
      "threshold": 0.5,
      "prefix_padding_ms": 300,
      "silence_duration_ms": 500
    },
    "tools": [],
    "tool_choice": "auto",
    "temperature": 0.8,
    "max_response_output_tokens": "inf"
  }
}
```

## 配置参数说明

### modalities（模态）

设置模型可以使用的响应类型。

| 值 | 说明 |
|----|------|
| `["text"]` | 仅文本响应 |
| `["audio"]` | 仅音频响应 |
| `["text", "audio"]` | 文本和音频响应 |

```json
{
  "type": "session.update",
  "session": {
    "modalities": ["text"]
  }
}
```

### instructions（系统提示）

设置系统提示词，类似于 Chat Completions API 的 `system` 消息。

```json
{
  "type": "session.update",
  "session": {
    "instructions": "你是一个专业的客服代表，请用简洁友好的语言回答问题。"
  }
}
```

### voice（声音）

设置模型输出音频时使用的声音。

**可用声音**：
- `alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, `cedar`

**注意**：一旦模型在会话中输出过音频，`voice` 将无法更改。

```json
{
  "type": "session.update",
  "session": {
    "voice": "marin"
  }
}
```

### input_audio_format（输入音频格式）

设置音频输入格式。

| 格式 | 说明 |
|------|------|
| `pcm16` | 16-bit PCM，24kHz，单声道（推荐） |
| `g711_ulaw` | G.711 μ-law |
| `g711_alaw` | G.711 A-law |

```json
{
  "type": "session.update",
  "session": {
    "input_audio_format": "pcm16"
  }
}
```

### output_audio_format（输出音频格式）

设置音频输出格式，可选值同 `input_audio_format`。

### input_audio_transcription（输入音频转录）

配置用户音频的实时转录（使用 Whisper 模型）。

```json
{
  "type": "session.update",
  "session": {
    "input_audio_transcription": {
      "model": "whisper-1"
    }
  }
}
```

**关闭转录**：

```json
{
  "type": "session.update",
  "session": {
    "input_audio_transcription": null
  }
}
```

### turn_detection（语音活动检测）

配置语音活动检测 (VAD)。

#### Server VAD（默认）

服务端自动检测用户说话的开始和结束。

```json
{
  "type": "session.update",
  "session": {
    "turn_detection": {
      "type": "server_vad",
      "threshold": 0.5,
      "prefix_padding_ms": 300,
      "silence_duration_ms": 500
    }
  }
}
```

| 参数 | 说明 | 默认值 | 范围 |
|------|------|--------|------|
| `threshold` | VAD 激活阈值 | 0.5 | 0.0 - 1.0 |
| `prefix_padding_ms` | 语音检测前包含的音频时长 | 300 | - |
| `silence_duration_ms` | 检测语音停止需要的静音时长 | 500 | - |

**注意**：
- 阈值越高，需要更大的声音才能激活，适合嘈杂环境
- `silence_duration_ms` 越短，模型响应越快，但可能在用户停顿时打断

#### 关闭 VAD

```json
{
  "type": "session.update",
  "session": {
    "turn_detection": null
  }
}
```

关闭 VAD 后，需要手动提交音频和触发响应。详细参考 [音频对话文档](./04-audio-conversation.md)。

### tools（工具/函数）

配置可调用的函数。详细参考 [函数调用文档](./05-function-calling.md)。

```json
{
  "type": "session.update",
  "session": {
    "tools": [
      {
        "type": "function",
        "name": "get_weather",
        "description": "获取指定城市的天气",
        "parameters": {
          "type": "object",
          "properties": {
            "city": {
              "type": "string",
              "description": "城市名称"
            }
          },
          "required": ["city"]
        }
      }
    ],
    "tool_choice": "auto"
  }
}
```

### temperature（温度）

控制响应的随机性。

| 范围 | 默认值 |
|------|--------|
| 0.6 - 1.2 | 0.8 |

```json
{
  "type": "session.update",
  "session": {
    "temperature": 0.7
  }
}
```

### max_response_output_tokens（最大输出令牌数）

限制单次响应的最大输出令牌数。

```json
{
  "type": "session.update",
  "session": {
    "max_response_output_tokens": 1000
  }
}
```

使用 `"inf"` 表示无限制（默认）。

## session.updated 事件

配置更新成功后，服务端会返回 `session.updated` 事件：

```json
{
  "type": "session.updated",
  "event_id": "evt_002",
  "session": {
    "id": "sess_xxx",
    "modalities": ["text"],
    "instructions": "你是一个友好的助手。",
    "temperature": 0.7
  }
}
```

## 会话生命周期

```
连接建立
    ↓
session.created（服务端发送）
    ↓
session.update（客户端配置，可选）
    ↓
session.updated（服务端确认）
    ↓
对话进行中...
    ↓
连接关闭 或 60分钟超时
```

## 注意事项

1. **无法更改的字段**：一旦模型在会话中输出过音频，`voice` 字段将无法更改
2. **清空字段**：要清空某个字段（如 `instructions`），传递空字符串 `""` 而非 `null`
3. **部分更新**：只传递需要更新的字段，未传递的字段保持不变
4. **会话时长**：单个会话最长 60 分钟，超时后需要重新连接
