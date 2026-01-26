# 客户端事件参考

本文档列出了所有可以从客户端发送到服务端的事件。

## 事件概览

| 事件类型 | 用途 |
|---------|------|
| `session.update` | 更新会话配置 |
| `conversation.item.create` | 创建对话项 |
| `conversation.item.truncate` | 截断对话项 |
| `conversation.item.delete` | 删除对话项 |
| `input_audio_buffer.append` | 追加音频数据 |
| `input_audio_buffer.commit` | 提交音频缓冲区 |
| `input_audio_buffer.clear` | 清空音频缓冲区 |
| `response.create` | 创建响应 |
| `response.cancel` | 取消正在进行的响应 |

## 通用字段

所有客户端事件都包含以下字段：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `event_id` | string | 否 | 客户端生成的唯一事件 ID，用于错误追踪 |
| `type` | string | 是 | 事件类型 |

---

## session.update

更新会话配置。可以在会话期间任何时候发送。

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

### session 参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `modalities` | array | 否 | 响应模态：`["text"]`、`["audio"]` 或 `["text", "audio"]` |
| `instructions` | string | 否 | 系统提示词 |
| `voice` | string | 否 | 声音：alloy、ash、ballad、coral、echo、sage、shimmer、verse、marin、cedar |
| `input_audio_format` | string | 否 | 输入音频格式：pcm16、g711_ulaw、g711_alaw |
| `output_audio_format` | string | 否 | 输出音频格式：pcm16、g711_ulaw、g711_alaw |
| `input_audio_transcription` | object | 否 | 音频转录配置 |
| `turn_detection` | object | 否 | VAD 配置，设为 null 禁用 |
| `tools` | array | 否 | 可用函数列表 |
| `tool_choice` | string | 否 | 工具选择策略：auto、none、required |
| `temperature` | number | 否 | 采样温度 (0.6-1.2) |
| `max_response_output_tokens` | integer | 否 | 最大输出 token 数 |

---

## conversation.item.create

创建新的对话项。

```json
{
  "type": "conversation.item.create",
  "event_id": "evt_002",
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

### 参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `previous_item_id` | string | 否 | 在此 ID 后插入新项，默认追加到末尾 |
| `item` | object | 是 | 对话项对象 |

### item 类型

#### 1. 消息 (message)

```json
{
  "id": "msg_001",
  "type": "message",
  "role": "user",
  "content": [
    { "type": "input_text", "text": "你好" }
  ]
}
```

| role | 支持的 content 类型 |
|------|---------------------|
| `user` | `input_text`、`input_audio`、`input_image` |
| `assistant` | `text`、`audio` |
| `system` | `input_text` |

#### 2. 函数调用 (function_call)

```json
{
  "id": "fc_001",
  "type": "function_call",
  "call_id": "call_abc",
  "name": "get_weather",
  "arguments": "{\"city\":\"北京\"}"
}
```

#### 3. 函数调用结果 (function_call_output)

```json
{
  "id": "fco_001",
  "type": "function_call_output",
  "call_id": "call_abc",
  "output": "{\"temperature\":25}"
}
```

---

## conversation.item.truncate

截断助手消息的音频。用于处理用户中断场景。

```json
{
  "type": "conversation.item.truncate",
  "event_id": "evt_003",
  "item_id": "msg_assist_001",
  "content_index": 0,
  "audio_end_ms": 1500
}
```

### 参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `item_id` | string | 是 | 要截断的助手消息 ID |
| `content_index` | integer | 是 | 内容索引，通常为 0 |
| `audio_end_ms` | integer | 是 | 截断点（毫秒） |

---

## conversation.item.delete

删除对话项。

```json
{
  "type": "conversation.item.delete",
  "event_id": "evt_004",
  "item_id": "msg_001"
}
```

### 参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `item_id` | string | 是 | 要删除的对话项 ID |

---

## input_audio_buffer.append

追加音频数据到输入缓冲区。

```json
{
  "type": "input_audio_buffer.append",
  "event_id": "evt_005",
  "audio": "base64编码的PCM16数据"
}
```

### 参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `audio` | string | 是 | Base64 编码的音频字节，最大 15MB |

### 注意

- 服务端不会对此事件发送确认响应
- 使用较小的块（如 4KB-8KB）可以提高 VAD 响应速度

---

## input_audio_buffer.commit

提交音频缓冲区，创建用户消息项。

```json
{
  "type": "input_audio_buffer.commit",
  "event_id": "evt_006"
}
```

### 使用场景

- VAD 禁用时，手动触发用户输入提交
- 提交后触发输入音频转录（如果启用）

---

## input_audio_buffer.clear

清空音频缓冲区。

```json
{
  "type": "input_audio_buffer.clear",
  "event_id": "evt_007"
}
```

### 使用场景

- Push-to-Talk 场景中，开始新的录音前清空旧数据
- 取消当前输入

---

## response.create

创建响应，触发模型生成。

```json
{
  "type": "response.create",
  "event_id": "evt_008",
  "response": {
    "output_modalities": ["text"],
    "instructions": "请简洁回答。"
  }
}
```

### 参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `response` | object | 否 | 响应配置 |

### response 参数

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `output_modalities` | array | 否 | 输出模态：`["text"]`、`["audio"]` |
| `instructions` | string | 否 | 本次响应的临时指令 |
| `voice` | string | 否 | 本次响应使用的声音 |
| `tools` | array | 否 | 本次响应可用的函数 |
| `tool_choice` | string | 否 | 工具选择策略 |
| `temperature` | number | 否 | 本次响应的温度 |
| `max_response_output_tokens` | integer | 否 | 本次响应的最大输出 token |
| `conversation` | string | 否 | 对话模式：`"auto"` 或 `"none"` |
| `metadata` | object | 否 | 元数据，用于识别响应 |
| `input` | array | 否 | 自定义输入上下文 |

### 使用 conversation: none 创建带外响应

```json
{
  "type": "response.create",
  "response": {
    "conversation": "none",
    "metadata": { "type": "classification" },
    "output_modalities": ["text"],
    "instructions": "分类对话主题"
  }
}
```

### 使用自定义输入

```json
{
  "type": "response.create",
  "response": {
    "input": [
      {
        "type": "message",
        "role": "user",
        "content": [
          { "type": "input_text", "text": "独立问题" }
        ]
      }
    ]
  }
}
```

---

## response.cancel

取消正在进行的响应。

```json
{
  "type": "response.cancel",
  "event_id": "evt_009"
}
```

### 注意

- 如果没有正在进行的响应，会返回错误
- 取消后会收到 `response.cancelled` 事件

---

## 完整示例

### 多轮文本对话

```json
// 第一轮
{"type": "conversation.item.create", "item": {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "你好"}]}}
{"type": "response.create"}

// 第二轮
{"type": "conversation.item.create", "item": {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "再详细说说"}]}}
{"type": "response.create"}
```

### Push-to-Talk 音频输入

```json
// 按下按钮
{"type": "input_audio_buffer.clear"}

// 录音中...
{"type": "input_audio_buffer.append", "audio": "base64chunk1"}
{"type": "input_audio_buffer.append", "audio": "base64chunk2"}
{"type": "input_audio_buffer.append", "audio": "base64chunk3"}

// 松开按钮
{"type": "input_audio_buffer.commit"}
{"type": "response.create"}
```

### 函数调用

```json
// 定义函数
{"type": "session.update", "session": {"tools": [{"type": "function", "name": "get_weather", "description": "获取天气", "parameters": {"type": "object", "properties": {"city": {"type": "string"}}, "required": ["city"]}}]}}

// 发送消息
{"type": "conversation.item.create", "item": {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "北京天气怎么样？"}]}}
{"type": "response.create"}

// 收到函数调用后返回结果
{"type": "conversation.item.create", "item": {"type": "function_call_output", "call_id": "call_abc", "output": "{\"temp\":25}"}}
{"type": "response.create"}
```
