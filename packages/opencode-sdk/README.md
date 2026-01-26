# OpenCode SDK

> TypeScript/JavaScript SDK and CLI for OpenCode AI programming assistant

[![npm version](https://img.shields.io/npm/v/opencode-sdk-cli)](https://www.npmjs.com/package/opencode-sdk-cli)
[![Node.js Version](https://img.shields.io/node/v/opencode-sdk-cli)](https://github.com/opencode-ai/opencode)
[![License](https://img.shields.io/npm/l/opencode-sdk-cli)](MIT)

## Features

- **Auto-start server** - Automatically starts OpenCode server if installed locally
- **Health detection** - Detects existing server before starting new one
- **Type-safe** - Full TypeScript support with exported types
- **CLI included** - Command-line interface for common operations
- **Cross-platform** - Works on macOS, Linux, and Windows (WSL)

## Installation

```bash
npm install -g opencode-sdk-cli
```

Or use with Node.js 18+:

```bash
npm install opencode-sdk-cli
```

## Quick Start

### As a Library

```typescript
import { createOpencode } from "opencode-sdk-cli"

// Auto-start server if needed
const { client, server } = await createOpencode()

// Create a session
const session = await client.session.create({
  agent: "build"
})

// Send a message
await client.session.chat(session.id, {
  content: "Explain this file",
  files: ["src/index.ts"]
})

// Get response
const messages = await client.session.messages(session.id)
console.log(messages[messages.length - 1].content)

// Optional: Shutdown server when done (if SDK started it)
await server?.shutdown()
```

### As a CLI

```bash
# Send a message to OpenCode
opencode-sdk chat "Explain this code" --files src/index.ts

# List all sessions
opencode-sdk session list

# Search for text in files
opencode-sdk files search "function handleClick" --path src/

# Find files by pattern
opencode-sdk files find "**/*.ts"
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENCODE_BASE_URL` | OpenCode server URL | `http://127.0.0.1:4096` |
| `OPENCODE_AGENT` | Default agent to use | - |

### Configuration Options

```typescript
const { client, server } = await createOpencode({
  hostname: "127.0.0.1",     // Server hostname
  port: 4096,                 // Server port
  server: {
    autoStart: true,          // Auto-start server (default: true)
    command: "opencode",      // Command to start server
    healthCheckTimeout: 3000, // Health check timeout (ms)
    startupTimeout: 30000,    // Server startup timeout (ms)
    silent: true,             // Suppress startup logs
  }
})
```

## API Reference

### createOpencode()

Creates an OpenCode client with optional auto-start.

```typescript
function createOpencode(
  config?: OpencodeConfig
): Promise<{ client: OpencodeClient; server: ServerHandle | null }>
```

**Returns:**
- `client` - The OpenCode client instance
- `server` - Server handle if SDK started the server, `null` otherwise

### OpencodeClient

#### Session API

```typescript
// Create a new session
const session = await client.session.create({
  agent: "build"  // Optional: specify agent
})

// List all sessions
const sessions = await client.session.list()

// Send a message
await client.session.chat(sessionId, {
  content: "Your message here",
  files: ["path/to/file.ts"]  // Optional: attach files
})

// Get message history
const messages = await client.session.messages(sessionId)

// Delete a session
await client.session.delete(sessionId)

// Abort ongoing operation
await client.session.abort(sessionId)
```

#### Files API

```typescript
// Search for text in files
const results = await client.files.search({
  query: "function handleClick",
  path: "src/"  // Optional: restrict search path
})

// Find files by pattern
const files = await client.files.find({
  pattern: "**/*.ts"
})

// Read a file
const content = await client.files.read({
  path: "src/index.ts"
})

// Find symbols (classes, functions, etc.)
const symbols = await client.files.symbols({
  query: "handleClick"
})
```

## CLI Reference

### chat

Send a message to OpenCode.

```bash
opencode-sdk chat <message> [options]
```

**Options:**
- `-a, --agent <agent>` - Agent to use (e.g., build, code)
- `-f, --files <files...>` - Files to attach
- `-u, --url <url>` - Server URL
- `-s, --session <id>` - Use existing session

**Example:**
```bash
opencode-sdk chat "Review this code" --files src/index.ts --agent build
```

### session

Manage OpenCode sessions.

```bash
# List all sessions
opencode-sdk session list

# Create a new session
opencode-sdk session create [-a <agent>]

# Get session messages
opencode-sdk session messages <sessionId>

# Delete a session
opencode-sdk session delete <sessionId>

# Abort ongoing operation
opencode-sdk session abort <sessionId>
```

### files

File operations with OpenCode.

```bash
# Search for text in files
opencode-sdk files search <query> [-p <path>]

# Find files by pattern
opencode-sdk files find <pattern>

# Read a file
opencode-sdk files read <path>

# Find symbols
opencode-sdk files symbols <query>
```

## Error Handling

```typescript
import {
  CommandNotFoundError,
  ServerUnhealthyError,
  ServerStartupError,
  StartupTimeoutError
} from "opencode-sdk-cli"

try {
  const { client, server } = await createOpencode()
} catch (error) {
  if (error instanceof CommandNotFoundError) {
    // OpenCode not installed - error includes installation instructions
    console.error(error.message)
  } else if (error instanceof StartupTimeoutError) {
    // Server didn't start in time
    console.error("Server startup timed out")
  }
}
```

## Server Handle

When the SDK starts the server, it returns a `ServerHandle` with management methods:

```typescript
interface ServerHandle {
  pid: number              // Process ID of the server
  managed: boolean         // True if SDK started the server
  shutdown: () => Promise<void>  // Gracefully shutdown the server
}

// Usage
const { client, server } = await createOpencode()

if (server) {
  console.log(`Started server with PID: ${server.pid}`)
  // ... do work ...
  await server.shutdown()
}
```

## Development

```bash
# Clone repository
git clone https://github.com/opencode-ai/opencode-sdk
cd opencode-sdk

# Install dependencies
npm install

# Build
npm run build

# Watch mode
npm run dev

# Run CLI
npm start -- chat "hello"
```

## License

MIT

## Links

- [OpenCode Documentation](https://opencode.ai)
- [OpenCode Installation](https://opencode.ai/install)
- [Report Issues](https://github.com/opencode-ai/opencode-sdk/issues)
