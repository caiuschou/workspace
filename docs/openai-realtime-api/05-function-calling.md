# 函数调用

本文档说明如何在 Realtime API 中使用函数调用（Function Calling）。

## 概述

函数调用允许模型在对话中调用你定义的函数，从而扩展模型的能力。工作流程：

1. 定义可用的函数
2. 模型根据用户输入决定是否调用函数
3. 客户端执行函数
4. 将结果返回给模型
5. 模型基于结果生成回复

## 定义函数

### 在会话级别定义

```json
{
  "type": "session.update",
  "session": {
    "tools": [
      {
        "type": "function",
        "name": "get_weather",
        "description": "获取指定城市的当前天气",
        "parameters": {
          "type": "object",
          "properties": {
            "city": {
              "type": "string",
              "description": "城市名称，例如：北京、上海"
            },
            "unit": {
              "type": "string",
              "enum": ["celsius", "fahrenheit"],
              "description": "温度单位"
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

### 在响应级别定义

```json
{
  "type": "response.create",
  "response": {
    "tools": [
      {
        "type": "function",
        "name": "get_weather",
        "description": "获取指定城市的当前天气",
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
    ]
  }
}
```

## tool_choice 参数

控制模型如何选择工具：

| 值 | 说明 |
|----|------|
| `"auto"` | 模型自动决定是否调用（默认） |
| `"none"` | 不调用任何函数 |
| `{"type": "function", "name": "func_name"}` | 强制调用指定函数 |
| `{"type": "function", "name": {"type": "function", "name": "func_a"}` | 强制调用指定函数（新格式） |

```json
{
  "type": "session.update",
  "session": {
    "tools": [...],
    "tool_choice": "auto"
  }
}
```

## 完整流程示例

### 步骤 1：配置函数

```json
{
  "type": "session.update",
  "session": {
    "tools": [
      {
        "type": "function",
        "name": "get_weather",
        "description": "获取指定城市的当前天气",
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
      },
      {
        "type": "function",
        "name": "get_time",
        "description": "获取当前时间",
        "parameters": {
          "type": "object",
          "properties": {
            "timezone": {
              "type": "string",
              "description": "时区，例如：Asia/Shanghai"
            }
          },
          "required": []
        }
      }
    ],
    "tool_choice": "auto"
  }
}
```

### 步骤 2：用户发送消息

```json
{
  "type": "conversation.item.create",
  "item": {
    "type": "message",
    "role": "user",
    "content": [
      {
        "type": "input_text",
        "text": "北京今天天气怎么样？"
      }
    ]
  }
}
```

### 步骤 3：请求响应

```json
{
  "type": "response.create"
}
```

### 步骤 4：接收函数调用请求

服务端返回 `response.done`，包含函数调用信息：

```json
{
  "type": "response.done",
  "event_id": "evt_001",
  "response": {
    "id": "resp_001",
    "status": "completed",
    "output": [
      {
        "id": "fc_001",
        "type": "function_call",
        "name": "get_weather",
        "call_id": "call_abc123",
        "arguments": "{\"city\":\"北京\"}"
      }
    ]
  }
}
```

### 步骤 5：执行函数并返回结果

客户端执行函数后，将结果返回给模型：

```json
{
  "type": "conversation.item.create",
  "item": {
    "type": "function_call_output",
    "call_id": "call_abc123",
    "output": "{\"temperature\":25,\"condition\":\"晴\",\"humidity\":45}"
  }
}
```

### 步骤 6：请求最终响应

```json
{
  "type": "response.create"
}
```

### 步骤 7：接收最终回复

```json
{
  "type": "response.done",
  "response": {
    "id": "resp_002",
    "status": "completed",
    "output": [
      {
        "type": "message",
        "role": "assistant",
        "content": [
          {
            "type": "text",
            "text": "北京今天天气晴朗，气温 25°C，湿度 45%。"
          }
        ]
      }
    ]
  }
}
```

## 代码示例

### Node.js 完整示例

```javascript
const WebSocket = require('ws');

// 定义可用的函数
const functions = {
  get_weather: async (args) => {
    const { city } = JSON.parse(args);
    // 模拟 API 调用
    return JSON.stringify({
      city,
      temperature: 25,
      condition: '晴',
      humidity: 45
    });
  },

  get_time: async (args) => {
    const { timezone } = JSON.parse(args);
    return JSON.stringify({
      time: new Date().toLocaleString('zh-CN', { timeZone: timezone || 'Asia/Shanghai' })
    });
  }
};

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
  // 配置会话和函数
  ws.send(JSON.stringify({
    type: 'session.update',
    session: {
      modalities: ['text'],
      tools: [
        {
          type: 'function',
          name: 'get_weather',
          description: '获取指定城市的当前天气',
          parameters: {
            type: 'object',
            properties: {
              city: {
                type: 'string',
                description: '城市名称'
              }
            },
            required: ['city']
          }
        },
        {
          type: 'function',
          name: 'get_time',
          description: '获取当前时间',
          parameters: {
            type: 'object',
            properties: {
              timezone: {
                type: 'string',
                description: '时区'
              }
            },
            required: []
          }
        }
      ],
      tool_choice: 'auto'
    }
  }));
});

ws.on('message', async (data) => {
  const event = JSON.parse(data);

  if (event.type === 'session.updated') {
    // 会话配置完成，发送测试消息
    sendMessage('北京今天天气怎么样？');
  }

  if (event.type === 'response.done') {
    const output = event.response.output;

    // 检查是否是函数调用
    for (const item of output) {
      if (item.type === 'function_call') {
        console.log('调用函数:', item.name);
        console.log('参数:', item.arguments);

        // 执行函数
        const result = await functions[item.name](item.arguments);
        console.log('函数结果:', result);

        // 返回结果
        ws.send(JSON.stringify({
          type: 'conversation.item.create',
          item: {
            type: 'function_call_output',
            call_id: item.call_id,
            output: result
          }
        }));

        // 请求最终响应
        ws.send(JSON.stringify({
          type: 'response.create'
        }));
      } else if (item.type === 'message' && item.role === 'assistant') {
        console.log('助手回复:', item.content[0].text);
      }
    }
  }
});

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

### 浏览器示例

```javascript
const functions = {
  get_weather: async (args) => {
    const { city } = JSON.parse(args);
    // 调用你的后端 API
    const response = await fetch(`/api/weather?city=${city}`);
    const data = await response.json();
    return JSON.stringify(data);
  }
};

const ws = new WebSocket(
  'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
  ['realtime', `openai-insecure-api-key.${apiKey}`]
);

ws.onmessage = async (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'response.done') {
    for (const item of data.response.output) {
      if (item.type === 'function_call') {
        // 执行函数
        const result = await functions[item.name](item.arguments);

        // 返回结果
        ws.send(JSON.stringify({
          type: 'conversation.item.create',
          item: {
            type: 'function_call_output',
            call_id: item.call_id,
            output: result
          }
        }));

        ws.send(JSON.stringify({
          type: 'response.create'
        }));
      }
    }
  }
};
```

## 函数调用事件流

```
客户端                                            服务端
   │                                                │
   ├────── session.update (定义函数) ─────────────>│
   │                                                │
   ├────── conversation.item.create ─────────────>│
   │                                                │
   ├────── response.create ───────────────────────>│
   │                                                │
   │<───── response.created ───────────────────────┤
   │                                                │
   │<───── response.done (包含 function_call) ─────┤
   │                                                │
   │  [客户端执行函数]                               │
   │                                                │
   ├────── conversation.item.create (返回结果) ───>│
   │                                                │
   ├────── response.create ───────────────────────>│
   │                                                │
   │<───── response.done (最终回复) ───────────────┤
```

## 并发函数调用

模型可以同时调用多个函数：

```json
{
  "type": "response.done",
  "response": {
    "output": [
      {
        "type": "function_call",
        "name": "get_weather",
        "call_id": "call_001",
        "arguments": "{\"city\":\"北京\"}"
      },
      {
        "type": "function_call",
        "name": "get_weather",
        "call_id": "call_002",
        "arguments": "{\"city\":\"上海\"}"
      }
    ]
  }
}
```

需要为每个调用返回结果：

```javascript
for (const item of output) {
  if (item.type === 'function_call') {
    const result = await functions[item.name](item.arguments);
    ws.send(JSON.stringify({
      type: 'conversation.item.create',
      item: {
        type: 'function_call_output',
        call_id: item.call_id,
        output: result
      }
    }));
  }
}

ws.send(JSON.stringify({ type: 'response.create' }));
```

## 函数调用参数 schema

使用 JSON Schema 定义参数：

```json
{
  "type": "object",
  "properties": {
    "location": {
      "type": "object",
      "description": "位置信息",
      "properties": {
        "city": {
          "type": "string",
          "description": "城市"
        },
        "country": {
          "type": "string",
          "description": "国家"
        }
      },
      "required": ["city"]
    },
    "unit": {
      "type": "string",
      "enum": ["celsius", "fahrenheit"],
      "description": "温度单位"
    }
  },
  "required": ["location"]
}
```

## 注意事项

1. **call_id 必须匹配**：返回函数结果时，`call_id` 必须与调用时的 `call_id` 一致
2. **output 必须是字符串**：函数返回值需要序列化为 JSON 字符串
3. **超时处理**：函数执行可能需要时间，需要处理超时和错误
4. **安全性**：函数调用可能执行敏感操作，需要验证和权限控制
