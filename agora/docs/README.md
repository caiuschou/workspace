# Agora Docs

Independent WebSocket server. Supports agent registration, discovery, and agent-to-agent communication relayed via Agora (including streaming).

## Contents

| Doc | Description |
|-----|-------------|
| [Registration & Discovery](registration-discovery.md) | LLM agents register themselves and discover other agents over WebSocket (minimal design). |
| [Agent Messaging Abstraction](agent-messaging-abstraction.md) | Abstract layer for agent-to-agent messaging: roles, addressing, one-shot/streaming, request–response correlation. |
| [Scenario: Assistant → Email Agent](scenario-assistant-email.md) | User message to assistant agent, forwarded to email send/receive agent; minimal design. |
| [Agent-to-Agent Protocol](agent-communication.md) | Concrete message protocol (Relay + Streaming) for agent-to-agent communication; one implementation of the abstraction. |
| [langgraph-rust Agent Messaging Design](langgraph-agent-messaging-design.md) | Simple design for agents to have messaging ability (inbound/outbound envelopes, request_id) using langgraph-rust StateGraph. |

## Implementation

Workspace contains two crates:

| Crate | Description |
|-------|-------------|
| `agora-protocol` | Shared types and protocols |
| `agora-server` | WebSocket service, see `agora-server/src/main.rs` |

Run: `cargo run -p agora-server`. Parse JSON `type` on `/ws` and implement:

- Registration & discovery: `register`, `list` (see [registration-discovery.md](registration-discovery.md))
- Relay: `send` / `send_chunk` / `send_end` (see [agent-communication.md](agent-communication.md))
