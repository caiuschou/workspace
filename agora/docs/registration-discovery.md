# LLM Agent Registration & Discovery (Minimal Design · WebSocket)

## Goals

- Agents connect over WebSocket and register themselves so other clients can discover them.
- Discoverers send a "list agents" request to the same WebSocket endpoint; the server returns the registered agent list.

## Design (Minimal)

| Item | Approach |
|------|----------|
| Transport | All over WebSocket, JSON text frames. |
| Storage | In-memory `HashMap`, key = agent `id`; cleared on restart. |
| Register | Client sends `register` message; server writes to memory and replies. |
| Discovery | Client sends `list` message; server replies with current agent list. |
| Endpoint | Single `ws://host:port/ws`. |

## Message Format (JSON Text Frames)

### 1. Register (Agent → Server)

Client sends:

```json
{
  "type": "register",
  "id": "agent-1",
  "name": "Code Assistant",
  "endpoint": "http://127.0.0.1:3000"
}
```

- `id`: Unique identifier; resubmission with same id is treated as update.
- `name`: Display name.
- `endpoint`: Optional; agent’s service URL.

Server replies:

```json
{
  "type": "registered",
  "id": "agent-1"
}
```

### 2. Discovery (Any Client → Server)

Client sends:

```json
{
  "type": "list"
}
```

Server replies:

```json
{
  "type": "agents",
  "agents": [
    {
      "id": "agent-1",
      "name": "Code Assistant",
      "endpoint": "http://127.0.0.1:3000",
      "registered_at": "2025-01-31T12:00:00Z"
    }
  ]
}
```

## Data Model (Minimal)

- In memory: `HashMap<String, AgentRecord>`, key = `id`.
- `AgentRecord`: `id`, `name`, `endpoint`, `registered_at` (ISO8601).

## Connection & Lifecycle

- Registration and discovery share the same WebSocket address; message `type` distinguishes them.
- Current minimal implementation does not bind "connection ↔ agent": the same connection may send `list` then `register`, or only one; disconnect does not remove the agent from the registry (optional extension: bind agent to connection id and remove on disconnect).

## Future Extensions (Not in Current Minimal Implementation)

- Unregister on disconnect: bind agent to the current connection on register; remove from registry when the connection closes.
- Heartbeat: agents send `ping`/`pong` or `refresh` periodically; remove if not refreshed within timeout.
- Capability tags: e.g. `capabilities: ["code", "search"]` for discovery by capability.
- Persistence: SQLite or file so registrations survive restart.
