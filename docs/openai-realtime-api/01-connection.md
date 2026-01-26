# 连接与认证

本文档说明如何建立与 OpenAI Realtime API 的 WebSocket 连接。

## WebSocket 连接地址

### OpenAI 官方

```
wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17
```

### Azure OpenAI

```
wss://<resource>.openai.azure.com/openai/realtime?api-version=2024-10-01-preview&deployment=<deployment-name>
```

## 认证方式

### 方式一：通过子协议传递 API Key（推荐）

```javascript
const apiKey = 'sk-...';

const ws = new WebSocket(
  'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
  ['realtime', `openai-insecure-api-key.${apiKey}`]
);
```

### 方式二：通过查询参数传递

```javascript
const ws = new WebSocket(
  `wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17&api-key=${apiKey}`
);
```

### 方式三：通过 HTTP 请求头传递

某些 WebSocket 客户端库支持自定义请求头：

```javascript
const ws = new WebSocket(
  'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
  {
    headers: {
      'Authorization': `Bearer ${apiKey}`,
      'OpenAI-Beta': 'realtime=v1'
    }
  }
);
```

## 完整连接示例

### Node.js (ws 库)

```javascript
const WebSocket = require('ws');

const apiKey = 'sk-...';

const ws = new WebSocket(
  'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
  {
    headers: {
      'Authorization': `Bearer ${apiKey}`,
      'OpenAI-Beta': 'realtime=v1'
    }
  }
);

ws.on('open', () => {
  console.log('WebSocket 连接已建立');
  // 连接成功后会自动收到 session.created 事件
});

ws.on('message', (data) => {
  const event = JSON.parse(data);
  console.log('收到事件:', event.type);
});

ws.on('error', (error) => {
  console.error('WebSocket 错误:', error);
});

ws.on('close', () => {
  console.log('WebSocket 连接已关闭');
});
```

### 浏览器原生 WebSocket

```javascript
const apiKey = 'sk-...';

const ws = new WebSocket(
  'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
  ['realtime', `openai-insecure-api-key.${apiKey}`]
);

ws.onopen = () => {
  console.log('连接已建立');
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('收到事件:', data.type);

  // 首个事件应该是 session.created
  if (data.type === 'session.created') {
    console.log('会话已创建:', data.session);
  }
};

ws.onerror = (error) => {
  console.error('连接错误:', error);
};

ws.onclose = () => {
  console.log('连接已关闭');
};
```

### Python (websockets 库)

```python
import asyncio
import json
import websockets

API_KEY = "sk-..."

async def connect():
    uri = "wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17"
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "OpenAI-Beta": "realtime=v1"
    }

    async with websockets.connect(uri, extra_headers=headers) as ws:
        print("连接已建立")

        while True:
            message = await ws.recv()
            event = json.loads(message)
            print(f"收到事件: {event['type']}")

            if event['type'] == 'session.created':
                print(f"会话已创建: {event['session']['id']}")

asyncio.run(connect())
```

## 连接后的首个事件

连接成功后，服务端会立即发送 `session.created` 事件：

```json
{
  "type": "session.created",
  "event_id": "evt_abc123",
  "session": {
    "id": "sess_xxx",
    "object": "realtime.session",
    "model": "gpt-4o-realtime-preview-2024-12-17",
    "modalities": ["text", "audio"],
    "instructions": "A helpful AI assistant.",
    "voice": "alloy",
    "input_audio_format": "pcm16",
    "output_audio_format": "pcm16",
    "turn_detection": {
      "type": "server_vad",
      "threshold": 0.5,
      "prefix_padding_ms": 300,
      "silence_duration_ms": 500
    },
    "temperature": 0.8,
    "max_response_output_tokens": null
  }
}
```

## 连接参数

### URL 查询参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `model` | string | 是 | 使用的模型名称 |
| `api-key` | string | 否* | API 密钥（如未通过其他方式传递） |

* 如果通过子协议或请求头传递了 API Key，则不需要此参数。

## 心跳保持

OpenAI Realtime API 使用 WebSocket 的 ping/pong 机制来保持连接。大多数 WebSocket 库会自动处理。

如果需要手动实现，可以定期发送 ping 帧：

```javascript
// 每 30 秒发送一次 ping
setInterval(() => {
  if (ws.readyState === WebSocket.OPEN) {
    ws.ping();
  }
}, 30000);
```

## 连接限制

| 限制项 | 值 |
|--------|-----|
| 最大会话时长 | 60 分钟 |
| 并发连接数 | 取决于 API 限制 |
| 空闲超时 | 建议保持心跳 |

## 错误处理

连接失败时可能收到的错误：

| 错误类型 | 说明 |
|----------|------|
| 401 Unauthorized | API Key 无效 |
| 403 Forbidden | 无权访问该模型 |
| 429 Too Many Requests | 超过速率限制 |
| 500 Server Error | 服务器内部错误 |

详细错误处理请参考 [错误处理文档](./08-error-handling.md)。
