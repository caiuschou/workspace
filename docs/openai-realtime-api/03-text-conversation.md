# 文本对话

本文档说明如何使用 Realtime API 进行纯文本对话。

## 概述

Realtime API 支持纯文本对话模式。与 Chat Completions API 不同，Realtime API 使用 WebSocket 保持有状态连接，可以实现更实时的交互。

## 配置文本模式

首先，将会话配置为纯文本模式：

```json
{
  "type": "session.update",
  "session": {
    "modalities": ["text"],
    "instructions": "你是一个友好的助手。"
  }
}
```

## 完整对话流程

### 步骤 1：发送用户消息

使用 `conversation.item.create` 创建用户消息：

```json
{
  "type": "conversation.item.create",
  "event_id": "evt_client_001",
  "item": {
    "id": "msg_user_001",
    "type": "message",
    "role": "user",
    "content": [
      {
        "type": "input_text",
        "text": "你好，请介绍一下你自己"
      }
    ]
  }
}
```

### 步骤 2：请求生成响应

```json
{
  "type": "response.create",
  "event_id": "evt_client_002",
  "response": {
    "output_modalities": ["text"]
  }
}
```

### 步骤 3：接收流式响应

服务端会发送一系列事件：

#### 3.1 response.created

```json
{
  "type": "response.created",
  "event_id": "evt_001",
  "response": {
    "id": "resp_001",
    "status": "in_progress",
    "output": []
  }
}
```

#### 3.2 conversation.item.created

```json
{
  "type": "conversation.item.created",
  "event_id": "evt_002",
  "item": {
    "id": "msg_assist_001",
    "type": "message",
    "role": "assistant",
    "content": []
  }
}
```

#### 3.3 response.text.delta（流式增量）

**增量 1**：
```json
{
  "type": "response.text.delta",
  "event_id": "evt_003",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "delta": "你好"
}
```

**增量 2**：
```json
{
  "type": "response.text.delta",
  "event_id": "evt_004",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "delta": "！我是"
}
```

**增量 3**：
```json
{
  "type": "response.text.delta",
  "event_id": "evt_005",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "delta": "AI助手"
}
```

#### 3.4 response.text.done

```json
{
  "type": "response.text.done",
  "event_id": "evt_006",
  "response_id": "resp_001",
  "item_id": "msg_assist_001",
  "output_index": 0,
  "content_index": 0,
  "text": "你好！我是AI助手"
}
```

#### 3.5 response.done

```json
{
  "type": "response.done",
  "event_id": "evt_007",
  "response": {
    "id": "resp_001",
    "status": "completed",
    "output": [
      {
        "id": "msg_assist_001",
        "type": "message",
        "role": "assistant",
        "content": [
          {
            "type": "text",
            "text": "你好！我是AI助手"
          }
        ]
      }
    ],
    "usage": {
      "total_tokens": 42,
      "input_tokens": 18,
      "output_tokens": 24
    }
  }
}
```

## 事件流程图

```
客户端                                            服务端
   │                                                │
   ├────── conversation.item.create ──────────────>│
   │                                                │
   ├────── response.create ───────────────────────>│
   │                                                │
   │<───── response.created ───────────────────────┤
   │                                                │
   │<───── conversation.item.created ──────────────┤
   │                                                │
   │<───── response.output_item.added ─────────────┤
   │                                                │
   │<───── response.content_part.added ────────────┤
   │                                                │
   │<───── response.text.delta (多次) ─────────────┤
   │                                                │
   │<───── response.text.done ─────────────────────┤
   │                                                │
   │<───── response.content_part.done ─────────────┤
   │                                                │
   │<───── response.output_item.done ──────────────┤
   │                                                │
   │<───── response.done ──────────────────────────┤
```

## 代码示例

### Node.js

```javascript
const WebSocket = require('ws');

const ws = new WebSocket(
  'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
  {
    headers: {
      'Authorization': `Bearer ${apiKey}`,
      'OpenAI-Beta': 'realtime=v1'
    }
  }
);

let fullText = '';

ws.on('open', () => {
  // 配置会话
  ws.send(JSON.stringify({
    type: 'session.update',
    session: {
      modalities: ['text'],
      instructions: '你是一个友好的助手。'
    }
  }));
});

ws.on('message', (data) => {
  const event = JSON.parse(data);

  switch (event.type) {
    case 'session.updated':
      // 会话配置完成，发送消息
      ws.send(JSON.stringify({
        type: 'conversation.item.create',
        item: {
          type: 'message',
          role: 'user',
          content: [{ type: 'input_text', text: '你好' }]
        }
      }));

      ws.send(JSON.stringify({
        type: 'response.create'
      }));
      break;

    case 'response.text.delta':
      // 追加增量文本
      fullText += event.delta;
      console.log('增量:', event.delta);
      break;

    case 'response.text.done':
      console.log('完整文本:', event.text);
      break;

    case 'response.done':
      console.log('响应完成');
      console.log('Token 使用:', event.response.usage);
      break;
  }
});
```

### 浏览器

```javascript
const ws = new WebSocket(
  'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
  ['realtime', `openai-insecure-api-key.${apiKey}`]
);

let responseText = '';

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'response.text.delta') {
    responseText += data.delta;
    // 实时更新 UI
    updateUI(responseText);
  }

  if (data.type === 'response.done') {
    console.log('最终回复:', responseText);
    responseText = '';
  }
};

function sendMessage(text) {
  ws.send(JSON.stringify({
    type: 'conversation.item.create',
    item: {
      type: 'message',
      role: 'user',
      content: [{ type: 'input_text', text }]
    }
  }));

  ws.send(JSON.stringify({
    type: 'response.create'
  }));
}
```

### Python

```python
import asyncio
import json
import websockets

async def text_conversation():
    uri = "wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17"
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "OpenAI-Beta": "realtime=v1"
    }

    async with websockets.connect(uri, extra_headers=headers) as ws:
        # 等待会话创建
        await ws.recv()  # session.created

        # 配置会话
        await ws.send(json.dumps({
            "type": "session.update",
            "session": {
                "modalities": ["text"],
                "instructions": "你是一个友好的助手。"
            }
        }))

        await ws.recv()  # session.updated

        # 发送消息
        await ws.send(json.dumps({
            "type": "conversation.item.create",
            "item": {
                "type": "message",
                "role": "user",
                "content": [{"type": "input_text", "text": "你好"}]
            }
        }))

        await ws.send(json.dumps({
            "type": "response.create"
        }))

        # 接收响应
        full_text = ""
        while True:
            message = await ws.recv()
            event = json.loads(message)

            if event["type"] == "response.text.delta":
                full_text += event["delta"]
                print(f"增量: {event['delta']}")

            elif event["type"] == "response.done":
                print(f"完整: {full_text}")
                break

asyncio.run(text_conversation())
```

## 响应状态

| 状态 | 说明 |
|------|------|
| `in_progress` | 响应生成中 |
| `completed` | 响应完成 |
| `cancelled` | 响应被取消（用户打断） |
| `incomplete` | 响应未完成（如达到 token 限制） |
| `failed` | 响应失败 |

## 多轮对话

由于会话是有状态的，只需持续发送新的 `conversation.item.create` 和 `response.create` 即可实现多轮对话：

```javascript
// 第一轮
sendMessage("你好");
// ... 收到回复

// 第二轮（会自动包含历史对话）
sendMessage("再详细说说");
// ... 收到回复
```

## 清除对话历史

如需清除对话历史，可以发送删除事件：

```json
{
  "type": "conversation.item.delete",
  "item_id": "msg_user_001"
}
```

或使用 `response.create` 的 `input` 参数创建带自定义上下文的响应：

```json
{
  "type": "response.create",
  "response": {
    "input": []
  }
}
```

空数组会忽略所有历史上下文。
