# OpenAI Realtime API 文档

OpenAI Realtime API 是一个基于 WebSocket 的实时对话 API，支持低延迟的文本和音频交互。

## 特性

- **实时双向通信** - 基于 WebSocket，支持客户端和服务端之间的双向实时通信
- **多模态支持** - 支持文本、音频输入和输出
- **原生语音处理** - 无需独立的 ASR/TTS，模型直接处理音频
- **函数调用** - 支持在对话中调用自定义函数
- **语音活动检测 (VAD)** - 自动检测用户说话开始和结束
- **会话状态管理** - 有状态的会话，自动维护对话历史

## WebSocket vs WebRTC

| 特性 | WebSocket | WebRTC |
|------|-----------|--------|
| 适用场景 | 服务端到服务端 | 浏览器/客户端应用 |
| 音频处理 | 手动处理 Base64 编码 | 自动媒体流处理 |
| 网络适应性 | 需要自行实现 | 自动处理网络波动 |
| 音频播放 | 需要自己实现 | 浏览器原生支持 |
| 推荐用途 | 后端服务、电话集成 | Web 应用、移动应用 |

## 支持的模型

- `gpt-4o-realtime-preview-2024-12-17` (默认)
- `gpt-4o-realtime-preview`

## 支持的声音

`alloy`, `ash`, `ballad`, `coral`, `echo`, `sage`, `shimmer`, `verse`, `marin`, `cedar`

推荐使用 `marin` 或 `cedar` 获得最佳质量。

## 音频格式

| 格式 | 描述 |
|------|------|
| `pcm16` | 16-bit PCM, 24kHz, 单声道（推荐） |
| `g711_ulaw` | G.711 μ-law |
| `g711_alaw` | G.711 A-law |

## 文档导航

- [连接与认证](./01-connection.md) - WebSocket 连接方式和认证
- [会话管理](./02-session.md) - 会话配置和生命周期
- [文本对话](./03-text-conversation.md) - 纯文本对话完整流程
- [音频对话](./04-audio-conversation.md) - 音频输入输出和 VAD
- [函数调用](./05-function-calling.md) - 函数调用配置和流程
- [客户端事件](./06-client-events.md) - 客户端事件完整参考
- [服务端事件](./07-server-events.md) - 服务端事件完整参考
- [错误处理](./08-error-handling.md) - 错误处理和常见问题

## 快速开始

### 1. 建立连接

```javascript
const ws = new WebSocket('wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17', [
  'realtime',
  `openai-insecure-api-key.${apiKey}`
]);

ws.onopen = () => {
  console.log('连接已建立');
};
```

### 2. 配置会话

```javascript
ws.send(JSON.stringify({
  type: 'session.update',
  session: {
    modalities: ['text'],
    instructions: '你是一个友好的助手。'
  }
}));
```

### 3. 发送消息

```javascript
// 创建用户消息
ws.send(JSON.stringify({
  type: 'conversation.item.create',
  item: {
    type: 'message',
    role: 'user',
    content: [{ type: 'input_text', text: '你好' }]
  }
}));

// 请求响应
ws.send(JSON.stringify({
  type: 'response.create'
}));
```

### 4. 处理响应

```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'response.text.delta') {
    console.log('增量:', data.delta);
  }

  if (data.type === 'response.done') {
    console.log('完成:', data.response.output);
  }
};
```

## 会话限制

- 最大会话时长：**60 分钟**
- 单次音频块最大：**15 MB**

## 相关资源

- [OpenAI Realtime API 官方文档](https://platform.openai.com/docs/guides/realtime)
- [OpenAI Realtime API 参考](https://platform.openai.com/docs/api-reference/realtime)
