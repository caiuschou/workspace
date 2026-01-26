# 音频对话

本文档说明如何使用 Realtime API 进行音频输入输出。

## 概述

Realtime API 的核心优势是**原生音频处理** - 模型直接处理音频，无需独立的 ASR（语音识别）和 TTS（语音合成），从而实现更低延迟的语音交互。

## 音频格式

### 支持的格式

| 格式 | 描述 | 采样率 | 声道 |
|------|------|--------|------|
| `pcm16` | 16-bit PCM 线性量化 | 24kHz | 单声道（推荐） |
| `g711_ulaw` | G.711 μ-law 压缩 | 8kHz | 单声道 |
| `g711_alaw` | G.711 A-law 压缩 | 8kHz | 单声道 |

### 配置音频格式

```json
{
  "type": "session.update",
  "session": {
    "input_audio_format": "pcm16",
    "output_audio_format": "pcm16"
  }
}
```

## 语音活动检测 (VAD)

### 启用 Server VAD（默认）

服务端自动检测用户说话的开始和结束：

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

### VAD 参数

| 参数 | 说明 | 默认值 | 调整建议 |
|------|------|--------|----------|
| `threshold` | 激活阈值 (0.0-1.0) | 0.5 | 嘈杂环境调高，安静环境调低 |
| `prefix_padding_ms` | 语音前包含的音频时长 | 300ms | 增加以捕获语音开头 |
| `silence_duration_ms` | 检测停止需要的静音时长 | 500ms | 减少以更快响应，但可能打断 |

### VAD 事件

当 VAD 检测到语音活动时，会发送以下事件：

#### input_audio_buffer.speech_started

```json
{
  "type": "input_audio_buffer.speech_started",
  "event_id": "evt_001",
  "audio_start_ms": 1500,
  "item_id": "msg_user_001"
}
```

#### input_audio_buffer.speech_stopped

```json
{
  "type": "input_audio_buffer.speech_stopped",
  "event_id": "evt_002",
  "audio_start_ms": 1500,
  "item_id": "msg_user_001"
}
```

## 发送音频（WebSocket）

### 方式一：流式发送（推荐用于 VAD 模式）

使用 `input_audio_buffer.append` 逐步发送音频：

```javascript
// 假设从麦克风获取音频流
const audioContext = new AudioContext({ sampleRate: 24000 });
const mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });
const source = audioContext.createMediaStreamSource(mediaStream);
const processor = audioContext.createScriptProcessor(4096, 1, 1);

processor.onaudioprocess = (e) => {
  const float32Array = e.inputBuffer.getChannelData(0);
  const pcm16 = floatTo16BitPCM(float32Array);
  const base64 = arrayBufferToBase64(pcm16);

  ws.send(JSON.stringify({
    type: 'input_audio_buffer.append',
    audio: base64
  }));
};

source.connect(processor);
processor.connect(audioContext.destination);

// 辅助函数
function floatTo16BitPCM(float32Array) {
  const buffer = new ArrayBuffer(float32Array.length * 2);
  const view = new DataView(buffer);
  for (let i = 0; i < float32Array.length; i++) {
    const s = Math.max(-1, Math.min(1, float32Array[i]));
    view.setInt16(i * 2, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
  }
  return buffer;
}

function arrayBufferToBase64(buffer) {
  let binary = '';
  const bytes = new Uint8Array(buffer);
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}
```

### 方式二：完整音频发送

使用 `conversation.item.create` 发送完整音频：

```json
{
  "type": "conversation.item.create",
  "item": {
    "type": "message",
    "role": "user",
    "content": [
      {
        "type": "input_audio",
        "audio": "base64编码的完整音频数据"
      }
    ]
  }
}
```

### 方式三：手动提交（VAD 关闭时）

当 VAD 关闭时，需要手动提交音频：

```javascript
// 1. 发送音频数据
ws.send(JSON.stringify({
  type: 'input_audio_buffer.append',
  audio: base64AudioChunk
}));

// 2. 提交音频缓冲区
ws.send(JSON.stringify({
  type: 'input_audio_buffer.commit'
}));

// 3. 触发响应
ws.send(JSON.stringify({
  type: 'response.create'
}));
```

## 接收音频（WebSocket）

### 接收音频流

监听 `response.audio.delta` 事件：

```javascript
let audioBuffer = [];
let audioContext = new AudioContext({ sampleRate: 24000 });

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'response.audio.delta') {
    // 接收音频增量
    const audioData = base64ToArrayBuffer(data.delta);
    audioBuffer.push(audioData);
  }

  if (data.type === 'response.audio.done') {
    // 音频完成，播放
    playAudio(audioBuffer);
    audioBuffer = [];
  }
};

function playAudio(buffers) {
  // 合并所有缓冲区
  const totalLength = buffers.reduce((sum, buf) => sum + buf.byteLength, 0);
  const combined = new Uint8Array(totalLength);
  let offset = 0;
  for (const buf of buffers) {
    combined.set(new Uint8Array(buf), offset);
    offset += buf.byteLength;
  }

  // 转换为 PCM16 并播放
  const int16Array = new Int16Array(combined.buffer);
  const float32Array = new Float32Array(int16Array.length);
  for (let i = 0; i < int16Array.length; i++) {
    float32Array[i] = int16Array[i] / 0x8000;
  }

  const audioBuffer = audioContext.createBuffer(1, float32Array.length, 24000);
  audioBuffer.getChannelData(0).set(float32Array);

  const source = audioContext.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(audioContext.destination);
  source.start();
}

function base64ToArrayBuffer(base64) {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}
```

### 音频转录

同时接收文本转录（如果启用）：

```javascript
let transcript = '';

if (data.type === 'response.audio_transcript.delta') {
  transcript += data.delta;
  console.log('转录增量:', data.delta);
}

if (data.type === 'response.audio_transcript.done') {
  console.log('完整转录:', transcript);
  transcript = '';
}
```

## 禁用 VAD（Push-to-Talk）

适用于按键说话场景：

```javascript
// 1. 禁用 VAD
ws.send(JSON.stringify({
  type: 'session.update',
  session: {
    turn_detection: null
  }
}));

// 2. 按下按钮开始录音
button.onmousedown = () => {
  ws.send(JSON.stringify({ type: 'input_audio_buffer.clear' }));
  startRecording();
};

// 3. 松开按钮停止并发送
button.onmouseup = async () => {
  const audioData = stopRecording();

  // 发送音频
  for (const chunk of chunkAudio(audioData)) {
    ws.send(JSON.stringify({
      type: 'input_audio_buffer.append',
      audio: chunk
    }));
  }

  // 提交并请求响应
  ws.send(JSON.stringify({ type: 'input_audio_buffer.commit' }));
  ws.send(JSON.stringify({ type: 'response.create' }));
};
```

## 音频中断

### 检测用户打断

```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'input_audio_buffer.speech_started') {
    // 用户开始说话，停止当前音频播放
    stopCurrentAudio();

    // 响应会被自动取消
  }
};
```

### 手动取消响应

```javascript
ws.send(JSON.stringify({
  type: 'response.cancel'
}));
```

### 截断未播放的音频

```javascript
ws.send(JSON.stringify({
  type: 'conversation.item.truncate',
  item_id: 'msg_assist_001',
  content_index': 0,
  audio_end_ms: 1500  // 保留前 1.5 秒
}));
```

## 完整示例：语音对话

```javascript
class RealtimeVoiceChat {
  constructor(apiKey) {
    this.ws = new WebSocket(
      'wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-12-17',
      ['realtime', `openai-insecure-api-key.${apiKey}`]
    );

    this.audioContext = new AudioContext({ sampleRate: 24000 });
    this.audioQueue = [];
    this.isPlaying = false;

    this.setupWebSocket();
    this.setupAudioInput();
  }

  setupWebSocket() {
    this.ws.onopen = () => {
      // 配置会话
      this.ws.send(JSON.stringify({
        type: 'session.update',
        session: {
          modalities: ['text', 'audio'],
          input_audio_format: 'pcm16',
          output_audio_format: 'pcm16',
          voice: 'marin',
          turn_detection: {
            type: 'server_vad',
            threshold: 0.5,
            silence_duration_ms: 500
          }
        }
      }));
    };

    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);

      switch (data.type) {
        case 'response.audio.delta':
          this.audioQueue.push(data.delta);
          if (!this.isPlaying) {
            this.playNextAudio();
          }
          break;

        case 'input_audio_buffer.speech_started':
          // 用户开始说话，停止播放
          this.stopAudio();
          break;

        case 'response.audio_transcript.delta':
          console.log('AI:', data.delta);
          break;
      }
    };
  }

  async setupAudioInput() {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    const source = this.audioContext.createMediaStreamSource(stream);
    const processor = this.audioContext.createScriptProcessor(4096, 1, 1);

    processor.onaudioprocess = (e) => {
      const float32Array = e.inputBuffer.getChannelData(0);
      const pcm16 = this.floatTo16BitPCM(float32Array);
      const base64 = this.arrayBufferToBase64(pcm16);

      this.ws.send(JSON.stringify({
        type: 'input_audio_buffer.append',
        audio: base64
      }));
    };

    source.connect(processor);
    processor.connect(this.audioContext.destination);
  }

  floatTo16BitPCM(float32Array) {
    const buffer = new ArrayBuffer(float32Array.length * 2);
    const view = new DataView(buffer);
    for (let i = 0; i < float32Array.length; i++) {
      const s = Math.max(-1, Math.min(1, float32Array[i]));
      view.setInt16(i * 2, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
    }
    return buffer;
  }

  arrayBufferToBase64(buffer) {
    let binary = '';
    const bytes = new Uint8Array(buffer);
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
  }

  async playNextAudio() {
    if (this.audioQueue.length === 0) {
      this.isPlaying = false;
      return;
    }

    this.isPlaying = true;
    const base64 = this.audioQueue.shift();
    const arrayBuffer = this.base64ToArrayBuffer(base64);

    const audioBuffer = this.audioContext.createBuffer(
      1,
      arrayBuffer.byteLength / 2,
      24000
    );

    const int16Array = new Int16Array(arrayBuffer);
    const float32Array = audioBuffer.getChannelData(0);

    for (let i = 0; i < int16Array.length; i++) {
      float32Array[i] = int16Array[i] / 0x8000;
    }

    const source = this.audioContext.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(this.audioContext.destination);
    source.onended = () => this.playNextAudio();
    source.start();
  }

  base64ToArrayBuffer(base64) {
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
  }

  stopAudio() {
    this.audioQueue = [];
    this.isPlaying = false;
  }
}

// 使用
const chat = new RealtimeVoiceChat('sk-...');
```

## WebRTC 音频处理

如果使用 WebRTC 而非 WebSocket，音频处理更简单：

```javascript
const pc = new RTCPeerConnection();

// 播放远程音频
const audioEl = document.createElement('audio');
audioEl.autoplay = true;
pc.ontrack = (e) => {
  audioEl.srcObject = e.streams[0];
};

// 添加本地音频
const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
pc.addTrack(stream.getTracks()[0]);
```

WebRTC 会自动处理音频编解码和网络传输。
