# 错误处理

本文档说明 Realtime API 的错误处理机制和常见错误。

## 错误事件格式

当发生错误时，服务端会发送 `error` 事件：

```json
{
  "type": "error",
  "event_id": "evt_error_001",
  "error": {
    "type": "invalid_request_error",
    "code": "invalid_value",
    "message": "Invalid value for parameter 'type'",
    "param": "type",
    "event_id": "evt_client_001"
  }
}
```

### 错误字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `type` | string | 错误类型 |
| `code` | string | 错误代码 |
| `message` | string | 人类可读的错误消息 |
| `param` | string | 导致错误的参数 |
| `event_id` | string | 触发错误的客户端事件 ID |

## 错误类型

### invalid_request_error

请求参数无效或格式错误。

```json
{
  "type": "error",
  "error": {
    "type": "invalid_request_error",
    "code": "invalid_value",
    "message": "Invalid value for parameter 'type'",
    "param": "type"
  }
}
```

**常见原因**：
- 事件类型不存在
- 参数值超出允许范围
- 缺少必需参数

### authentication_error

认证失败。

```json
{
  "type": "error",
  "error": {
    "type": "authentication_error",
    "code": "invalid_api_key",
    "message": "Invalid API key"
  }
}
```

**常见原因**：
- API Key 无效
- API Key 已被撤销
- 未提供 API Key

### permission_error

权限不足。

```json
{
  "type": "error",
  "error": {
    "type": "permission_error",
    "code": "insufficient_permissions",
    "message": "You don't have access to this model"
  }
}
```

**常见原因**：
- API Key 无权访问指定模型
- 尝试访问不属于你的资源

### rate_limit_error

超过速率限制。

```json
{
  "type": "error",
  "error": {
    "type": "rate_limit_error",
    "code": "rate_limit_exceeded",
    "message": "Rate limit exceeded, please try again later"
  }
}
```

**处理方式**：

```javascript
if (event.error.type === 'rate_limit_error') {
  // 等待后重试
  setTimeout(() => {
    retryRequest();
  }, 60000); // 等待 60 秒
}
```

### api_error

API 内部错误。

```json
{
  "type": "error",
  "error": {
    "type": "api_error",
    "code": "internal_error",
    "message": "An internal error occurred"
  }
}
```

**处理方式**：
- 记录错误详情
- 稍后重试
- 如果持续出现，联系支持

### content_filter

内容被安全过滤器拦截。

```json
{
  "type": "error",
  "error": {
    "type": "content_filter",
    "code": "content_filtered",
    "message": "Content was filtered due to safety guidelines"
  }
}
```

## 错误处理最佳实践

### 1. 使用 event_id 追踪

为每个客户端请求设置唯一的 `event_id`：

```javascript
function sendEvent(type, data = {}) {
  const eventId = `evt_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

  const event = {
    type,
    event_id: eventId,
    ...data
  };

  ws.send(JSON.stringify(event));
  return eventId;
}
```

### 2. 错误监听器

```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'error') {
    handleError(data.error);
  }
};

function handleError(error) {
  console.error('Error:', error.message);

  switch (error.type) {
    case 'invalid_request_error':
      // 修复请求并重试
      fixAndRetry(error);
      break;

    case 'rate_limit_error':
      // 等待后重试
      scheduleRetry();
      break;

    case 'authentication_error':
      // 提示用户重新登录
      promptReauth();
      break;

    default:
      // 未知错误，记录并提示
      logError(error);
      notifyUser('发生错误，请稍后重试');
  }
}
```

### 3. 重试策略

```javascript
class RetryableRequest {
  constructor(ws, maxRetries = 3, baseDelay = 1000) {
    this.ws = ws;
    this.maxRetries = maxRetries;
    this.baseDelay = baseDelay;
    this.retryCount = 0;
  }

  async send(event) {
    const eventId = event.event_id || this.generateEventId();
    event.event_id = eventId;

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Request timeout'));
      }, 30000);

      const handler = (data) => {
        if (data.event_id === eventId || data.error?.event_id === eventId) {
          clearTimeout(timeout);
          ws.removeEventListener('message', handler);

          if (data.error) {
            this.handleError(data.error, event, resolve, reject);
          } else {
            resolve(data);
          }
        }
      };

      ws.addEventListener('message', handler);
      ws.send(JSON.stringify(event));
    });
  }

  handleError(error, originalEvent, resolve, reject) {
    const retryableErrors = ['rate_limit_error', 'api_error'];

    if (retryableErrors.includes(error.type) && this.retryCount < this.maxRetries) {
      this.retryCount++;
      const delay = this.baseDelay * Math.pow(2, this.retryCount - 1);

      setTimeout(() => {
        this.send(originalEvent).then(resolve).catch(reject);
      }, delay);
    } else {
      reject(error);
    }
  }

  generateEventId() {
    return `evt_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

// 使用
const request = new RetryableRequest(ws);

try {
  const response = await request.send({
    type: 'conversation.item.create',
    item: { type: 'message', role: 'user', content: [...] }
  });
  console.log('Success:', response);
} catch (error) {
  console.error('Failed after retries:', error);
}
```

## 常见错误场景

### 场景 1：无效的事件类型

**错误**：
```json
{
  "type": "error",
  "error": {
    "type": "invalid_request_error",
    "message": "Unknown event type: 'unknown.event'",
    "param": "type"
  }
}
```

**解决**：检查事件类型拼写是否正确。

### 场景 2：语音已被输出后更改声音

**错误**：
```json
{
  "type": "error",
  "error": {
    "type": "invalid_request_error",
    "message": "Cannot change voice after audio has been output",
    "param": "voice"
  }
}
```

**解决**：一旦模型输出过音频，`voice` 字段无法更改。需要重新建立连接。

### 场景 3：音频缓冲区为空时提交

**错误**：
```json
{
  "type": "error",
  "error": {
    "type": "invalid_request_error",
    "message": "Cannot commit empty audio buffer"
  }
}
```

**解决**：确保在提交前已发送音频数据。

### 场景 4：不存在的对话项 ID

**错误**：
```json
{
  "type": "error",
  "error": {
    "type": "invalid_request_error",
    "message": "Item not found: msg_999",
    "param": "item_id"
  }
}
```

**解决**：检查对话项 ID 是否正确。

### 场景 5：函数调用 call_id 不匹配

**错误**：
```json
{
  "type": "error",
  "error": {
    "type": "invalid_request_error",
    "message": "Function call with call_id 'call_abc' not found",
    "param": "call_id"
  }
}
```

**解决**：确保返回函数结果时使用正确的 `call_id`。

## 响应状态错误

### response.failed

响应生成失败时发送 `response.failed` 事件：

```json
{
  "type": "response.failed",
  "event_id": "evt_001",
  "response": {
    "id": "resp_001",
    "status": "failed",
    "status_details": {
      "type": "content_filter",
      "reason": "Content was filtered"
    }
  }
}
```

### 状态详情类型

| 类型 | 说明 |
|------|------|
| `content_filter` | 内容被安全过滤器拦截 |
| `max_output_tokens` | 达到最大输出 token 限制 |
| `rate_limit` | 超过速率限制 |

## 连接错误

### WebSocket 连接失败

```javascript
ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = (event) => {
  console.log('WebSocket closed:', event.code, event.reason);

  switch (event.code) {
    case 1000:
      // 正常关闭
      console.log('Connection closed normally');
      break;

    case 1006:
      // 异常关闭（网络问题）
      console.log('Connection lost, reconnecting...');
      reconnect();
      break;

    case 4001:
      // 认证失败
      console.log('Authentication failed');
      break;

    default:
      console.log('Unexpected close code:', event.code);
  }
};
```

### 重连策略

```javascript
class ReconnectingWebSocket {
  constructor(url, protocols, options = {}) {
    this.url = url;
    this.protocols = protocols;
    this.reconnectDelay = options.reconnectDelay || 1000;
    this.maxReconnectDelay = options.maxReconnectDelay || 30000;
    this.currentReconnectDelay = this.reconnectDelay;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = options.maxReconnectAttempts || Infinity;

    this.connect();
  }

  connect() {
    this.ws = new WebSocket(this.url, this.protocols);

    this.ws.onopen = () => {
      console.log('Connected');
      this.reconnectAttempts = 0;
      this.currentReconnectDelay = this.reconnectDelay;
    };

    this.ws.onclose = (event) => {
      if (this.reconnectAttempts < this.maxReconnectAttempts) {
        this.scheduleReconnect();
      }
    };

    // 传递其他事件
    this.ws.onmessage = this.onmessage;
    this.ws.onerror = this.onerror;
  }

  scheduleReconnect() {
    this.reconnectAttempts++;
    console.log(`Reconnecting in ${this.currentReconnectDelay}ms...`);

    setTimeout(() => {
      this.connect();
      this.currentReconnectDelay = Math.min(
        this.currentReconnectDelay * 2,
        this.maxReconnectDelay
      );
    }, this.currentReconnectDelay);
  }

  send(data) {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(data);
    } else {
      console.warn('WebSocket not open, message not sent');
    }
  }

  close() {
    this.reconnectAttempts = this.maxReconnectAttempts;
    this.ws.close();
  }
}

// 使用
const ws = new ReconnectingWebSocket(
  'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
  ['realtime', `openai-insecure-api-key.${apiKey}`],
  { reconnectDelay: 1000, maxReconnectDelay: 30000 }
);
```

## 调试技巧

### 1. 记录所有事件

```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(`[${data.type}]`, data);

  // 处理事件...
};
```

### 2. 记录所有发送的事件

```javascript
const originalSend = ws.send.bind(ws);
ws.send = (data) => {
  console.log('[SEND]', data);
  originalSend(data);
};
```

### 3. 使用事件 ID 追踪

```javascript
const pendingRequests = new Map();

function sendEvent(event) {
  const eventId = event.event_id || generateEventId();
  event.event_id = eventId;

  pendingRequests.set(eventId, {
    event,
    timestamp: Date.now(),
    status: 'pending'
  });

  ws.send(JSON.stringify(event));
  return eventId;
}

function markRequestComplete(eventId, status, result) {
  const request = pendingRequests.get(eventId);
  if (request) {
    request.status = status;
    request.result = result;
    request.completedAt = Date.now();
    request.duration = request.completedAt - request.timestamp;
    console.log('Request completed:', request);
  }
}
```

## 常见问题排查

| 问题 | 可能原因 | 解决方法 |
|------|----------|----------|
| 连接立即关闭 | API Key 无效 | 检查 API Key |
| 收到 401 错误 | 认证失败 | 检查认证方式 |
| 收到 429 错误 | 超过速率限制 | 等待后重试 |
| 音频无法播放 | 格式不匹配 | 检查音频格式配置 |
| 响应为空 | 内容被过滤 | 检查输入内容 |
| VAD 不工作 | 音频质量太低 | 调整 threshold 或使用 push-to-talk |
