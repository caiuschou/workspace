# Security Considerations

## Network Security

- **TLS by Default**: Uses `rustls-tls` (no OpenSSL dependency)
- **Localhost Binding**: Default hostname `127.0.0.1` prevents external access
- **No Credential Storage**: SDK doesn't persist secrets

## Process Security

- **Null Stdio**: Spawned server has detached stdio (no sensitive output leakage)
- **Graceful Shutdown**: Uses `SIGTERM` (not `SIGKILL`) allowing cleanup

## Input Validation

- Path validation via `to_str()` checks (reject non-UTF-8 paths)
- JSON deserialization with explicit type bounds

---

# Performance Considerations

## Connection Reuse

`reqwest::Client` maintains a connection pool internally. Creating multiple `Client` instances is lightweight; they share the underlying pool.

## Streaming vs Polling

The SDK uses SSE streaming (not polling) for response waiting:

| Approach | Latency | Server Load | Implementation |
|----------|---------|-------------|----------------|
| ~~Polling~~ | High | High (repeated requests) | Not used |
| **SSE Streaming** | Low | Low (single connection) | Current |

## Memory Efficiency

- Streaming callbacks receive `&str` references, no intermediate allocations
- Large responses are not fully buffered; process incrementally via `on_text`
