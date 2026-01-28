# Zoekt MCP

MCP server for [Zoekt](https://github.com/sourcegraph/zoekt) code search. Exposes `zoekt_search` and `zoekt_list` as tools. Supports **stdio**, **SSE** (Streamable HTTP), and **WebSocket** transports.

## Transports

| Mode | Invocation | Use case |
|------|------------|----------|
| **stdio** | `zoekt-mcp` or `zoekt-mcp --stdio` | Subprocess/pipe (e.g. Cursor, Claude Desktop) |
| **HTTP** | `zoekt-mcp --http [ADDR]` | Network: Streamable HTTP/SSE at `/mcp`, WebSocket at `/ws`. Default bind: `0.0.0.0:8080` |

## Configuration

- **ZOEKT_BASE_URL**: Zoekt-webserver base URL (default `http://127.0.0.1:6070`).  
  The Zoekt instance must have RPC enabled (zoekt-webserver `-rpc`).
- **ZOEKT_USERNAME** / **ZOEKT_PASSWORD**: Optional HTTP Basic auth. When both are set, every request to Zoekt includes `Authorization: Basic base64(user:password)`.

## Build & run

From workspace root:

```bash
cargo build -p zoekt-mcp
```

- Stdio (default):

  ```bash
  ZOEKT_BASE_URL=http://127.0.0.1:6070 ./target/debug/zoekt-mcp
  ```

- HTTP (SSE + WebSocket):

  ```bash
  ZOEKT_BASE_URL=http://127.0.0.1:6070 ./target/debug/zoekt-mcp --http 0.0.0.0:8080
  ```

  - Streamable HTTP/SSE: `POST/GET/DELETE http://HOST:8080/mcp`
  - WebSocket: `ws://HOST:8080/ws` (subprotocol `mcp`)

## Tools

- **zoekt_search**: Code search. Args: `q` (required), `repo_ids`, `num_context_lines`, `max_doc_display_count`.
- **zoekt_list**: List repositories. Args: `q` (repo query, use `""` for all).

Query syntax: [Zoekt query_syntax](https://github.com/sourcegraph/zoekt/blob/main/doc/query_syntax.md).
