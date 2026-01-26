import { createOpencode, createOpencodeClient, type ServerConfig } from "./sdk.js"

export interface ClientOptions {
  baseUrl?: string
  timeout?: number
  /** Enable auto-start of OpenCode server (default: true) */
  autoStart?: boolean
  /** Server auto-start configuration */
  server?: ServerConfig
}

export async function createClient(options: ClientOptions = {}) {
  // Determine if we should use auto-start:
  // - If autoStart is explicitly false, don't auto-start
  // - If baseUrl is provided, assume it's a remote server and don't auto-start
  // - Otherwise, use auto-start to detect/start local server
  const shouldAutoStart = options.autoStart !== false && !options.baseUrl

  if (shouldAutoStart) {
    // Use createOpencode which will:
    // 1. Check if server is already running
    // 2. Start server if not running (via 'opencode serve')
    // 3. Return both client and server handle
    return await createOpencode({
      timeout: options.timeout,
      server: options.server,
    })
  }

  // Either autoStart is disabled or baseUrl is provided (remote server)
  return {
    client: createOpencodeClient({
      baseUrl: options.baseUrl || "http://127.0.0.1:4096",
    }),
    server: null,
  }
}
