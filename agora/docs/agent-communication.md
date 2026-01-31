# Agent-to-Agent Protocol (Relay Only + Streaming)

本协议是 [Agent Messaging Abstraction](agent-messaging-abstraction.md) 的一种具体实现（Relay + JSON 文本帧 + WebSocket）。

## Goals

- Agent-to-agent messages are **relayed** by Agora (centralized).
- Both sending and receiving support **incremental/streaming**: LLM output is sent in chunks and received in chunks.

## Prerequisite

- On register, bind "agent id ↔ current WebSocket connection" and remove from registry on disconnect so Agora can forward messages to the right connection.

## Message Types (JSON Text Frames)

### 1. Send (Sender → Agora)

**Stream start:**

```json
{
  "type": "send",
  "to": "agent-2",
  "stream_id": "stream-1",
  "stream": true,
  "payload": { "action": "query", "body": { "question": "..." } }
}
```

- `stream_id`: Unique id for this streaming session; same id used for subsequent chunks.
- `stream: true`: Indicates multiple `send_chunk` messages will follow.
- `payload`: Optional; metadata for the streamed request (e.g. action, question).

**Stream chunk (incremental):**

```json
{
  "type": "send_chunk",
  "to": "agent-2",
  "stream_id": "stream-1",
  "chunk": "This is"
}
```

```json
{
  "type": "send_chunk",
  "to": "agent-2",
  "stream_id": "stream-1",
  "chunk": " a segment"
}
```

**Stream end:**

```json
{
  "type": "send_end",
  "to": "agent-2",
  "stream_id": "stream-1"
}
```

**Non-streaming (one-shot):**

```json
{
  "type": "send",
  "to": "agent-2",
  "payload": { "action": "query", "body": { "question": "..." } }
}
```

- When `stream` is absent or `stream: false`, no `send_chunk` / `send_end` is expected.

### 2. Agora Forwards to Receiver

**Stream start:**

```json
{
  "type": "message",
  "from": "agent-1",
  "stream_id": "stream-1",
  "stream": true,
  "payload": { "action": "query", "body": { "question": "..." } }
}
```

**Stream chunk:**

```json
{
  "type": "message_chunk",
  "from": "agent-1",
  "stream_id": "stream-1",
  "chunk": "This is"
}
```

**Stream end:**

```json
{
  "type": "message_end",
  "from": "agent-1",
  "stream_id": "stream-1"
}
```

**Non-streaming:**

```json
{
  "type": "message",
  "from": "agent-1",
  "payload": { "action": "query", "body": { "question": "..." } }
}
```

### 3. Receiver Reply (Optional; Also Supports Streaming)

- **Streaming reply**: Use `reply` / `reply_chunk` / `reply_end` with symmetric fields (`to` instead of `from`; same or new `stream_id`).
- **Non-streaming reply**: Only `type: "reply"` with `to` and `payload`.
- For request–response correlation, add `request_id` in the first `payload` and include the same `request_id` in the reply.

## Summary

| Role | Streaming send | Streaming receive |
|------|----------------|-------------------|
| Sender | `send`(stream:true) → `send_chunk`(multiple) → `send_end` | Receives `message` / `message_chunk` / `message_end` |
| Agora | Looks up connection by `to` and forwards as-is | Stateless; only forwards |
| Receiver | Receives `message` / `message_chunk` / `message_end` | Optional `reply` / `reply_chunk` / `reply_end` |
