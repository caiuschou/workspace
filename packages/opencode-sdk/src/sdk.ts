/**
 * Local SDK implementation
 * TODO: Replace with @opencode-ai/sdk when available
 */

import { detectCommand } from "./utils/command-detector.js"
import { checkServerHealth, waitForServer } from "./utils/health-check.js"
import {
  startServer,
  type ServerHandle,
} from "./utils/server-process.js"
import {
  CommandNotFoundError,
  StartupTimeoutError,
} from "./errors.js"

/**
 * Configuration for server auto-start behavior
 */
export interface ServerConfig {
  /** Whether to attempt auto-starting the server (default: true) */
  autoStart?: boolean

  /** Command to use for starting server (default: "opencode") */
  command?: string

  /** Arguments to pass when starting server */
  serverArgs?: string[]

  /** How long to wait for health check (ms) */
  healthCheckTimeout?: number

  /** How long to wait for server to be ready after starting (ms) */
  startupTimeout?: number

  /** Whether to suppress startup logs */
  silent?: boolean
}

export interface OpencodeConfig {
  hostname?: string
  port?: number
  timeout?: number
  config?: Record<string, unknown>
  /** Server auto-start configuration */
  server?: ServerConfig
}

export interface ClientConfig {
  baseUrl: string
}

export interface Session {
  id: string
  title?: string
  agent?: string
}

export interface Message {
  id: string
  role: "user" | "assistant"
  content: string
}

export interface ChatOptions {
  content: string
  files?: string[]
}

export interface SearchOptions {
  query: string
  path?: string
}

export interface FindOptions {
  pattern: string
}

export interface ReadOptions {
  path: string
}

export interface SymbolsOptions {
  query: string
}

export interface SearchResult {
  path: string
  line?: number
  content?: string
}

export interface Symbol {
  name: string
  path?: string
  kind?: string
}

/**
 * Safely parse JSON response with better error handling
 */
async function parseJSON<T>(res: Response, errorMessage: string): Promise<T> {
  const contentType = res.headers.get("content-type")

  // Check if response is actually JSON
  if (!contentType?.includes("application/json")) {
    // Try to get the response text for debugging
    const text = await res.text().catch(() => "")
    throw new Error(
      `${errorMessage}\n` +
      `Expected JSON response but received: ${contentType || "unknown content-type"}\n` +
      (text ? `Response body: ${text.slice(0, 200)}${text.length > 200 ? "..." : ""}` : "")
    )
  }

  try {
    return await res.json() as T
  } catch (err) {
    const text = await res.text().catch(() => "")
    throw new Error(
      `${errorMessage}\n` +
      `Failed to parse JSON response\n` +
      (text ? `Response body: ${text.slice(0, 200)}${text.length > 200 ? "..." : ""}` : "")
    )
  }
}

/**
 * Check response and throw detailed error if not ok
 */
async function checkResponse(res: Response, operation: string): Promise<void> {
  if (!res.ok) {
    const contentType = res.headers.get("content-type")
    let details = `HTTP ${res.status}: ${res.statusText}`

    // Try to get more details from response body
    if (contentType?.includes("application/json")) {
      try {
        const json = await res.json()
        details += `\n${JSON.stringify(json, null, 2)}`
      } catch {}
    } else {
      const text = await res.text().catch(() => "")
      if (text) {
        details += `\n${text.slice(0, 300)}${text.length > 300 ? "..." : ""}`
      }
    }

    throw new Error(`Failed to ${operation}: ${details}`)
  }
}

class SessionAPI {
  constructor(private baseUrl: string) {}

  async create(options?: { agent?: string }): Promise<Session> {
    const res = await fetch(`${this.baseUrl}/session`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(options || {}),
    })
    await checkResponse(res, "create session")
    return parseJSON<Session>(res, "Failed to parse session response")
  }

  async list(): Promise<Session[]> {
    const res = await fetch(`${this.baseUrl}/session`)
    await checkResponse(res, "list sessions")
    return parseJSON<Session[]>(res, "Failed to parse sessions response")
  }

  async chat(sessionId: string, options: ChatOptions): Promise<void> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}/chat`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(options),
    })
    await checkResponse(res, "send message")
  }

  async messages(sessionId: string): Promise<Message[]> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}/messages`)
    await checkResponse(res, "get messages")
    return parseJSON<Message[]>(res, "Failed to parse messages response")
  }

  async delete(sessionId: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}`, {
      method: "DELETE",
    })
    await checkResponse(res, "delete session")
  }

  async abort(sessionId: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}/abort`, {
      method: "POST",
    })
    await checkResponse(res, "abort session")
  }
}

class FilesAPI {
  constructor(private baseUrl: string) {}

  async search(options: SearchOptions): Promise<SearchResult[]> {
    const params = new URLSearchParams({ query: options.query })
    if (options.path) params.set("path", options.path)
    const res = await fetch(`${this.baseUrl}/files/search?${params}`)
    await checkResponse(res, "search files")
    return parseJSON<SearchResult[]>(res, "Failed to parse search response")
  }

  async find(options: FindOptions): Promise<string[]> {
    const params = new URLSearchParams({ pattern: options.pattern })
    const res = await fetch(`${this.baseUrl}/files/find?${params}`)
    await checkResponse(res, "find files")
    return parseJSON<string[]>(res, "Failed to parse files response")
  }

  async read(options: ReadOptions): Promise<string> {
    const params = new URLSearchParams({ path: options.path })
    const res = await fetch(`${this.baseUrl}/files/read?${params}`)
    await checkResponse(res, "read file")
    return res.text()
  }

  async symbols(options: SymbolsOptions): Promise<Symbol[]> {
    const params = new URLSearchParams({ query: options.query })
    const res = await fetch(`${this.baseUrl}/files/symbols?${params}`)
    await checkResponse(res, "find symbols")
    return parseJSON<Symbol[]>(res, "Failed to parse symbols response")
  }
}

export class OpencodeClient {
  public session: SessionAPI
  public files: FilesAPI

  constructor(config: ClientConfig) {
    this.session = new SessionAPI(config.baseUrl)
    this.files = new FilesAPI(config.baseUrl)
  }
}

export function createOpencodeClient(config: ClientConfig): OpencodeClient {
  return new OpencodeClient(config)
}

/**
 * Create an OpenCode client with optional auto-start
 *
 * @param config - SDK configuration
 * @returns Client and optional server handle
 *
 * @example
 * ```typescript
 * // Auto-start server if needed
 * const { client, server } = await createOpencode()
 *
 * // Connect to existing server only
 * const { client } = await createOpencode({ server: { autoStart: false } })
 *
 * // Shutdown server when done (if we started it)
 * await server?.shutdown()
 * ```
 */
export async function createOpencode(
  config?: OpencodeConfig
): Promise<{
  client: OpencodeClient
  server: ServerHandle | null
}> {
  const hostname = config?.hostname ?? "127.0.0.1"
  const port = config?.port ?? 4096
  const baseUrl = `http://${hostname}:${port}`
  const serverConfig = config?.server ?? {}

  // 1. If autoStart is explicitly disabled, just return client
  if (serverConfig.autoStart === false) {
    return {
      client: createOpencodeClient({ baseUrl }),
      server: null,
    }
  }

  // 2. Check if opencode command is available
  const detection = await detectCommand(serverConfig.command ?? "opencode")
  if (!detection.available) {
    throw new CommandNotFoundError(serverConfig.command ?? "opencode")
  }

  // 3. Check if server is already running
  const health = await checkServerHealth(
    baseUrl,
    serverConfig.healthCheckTimeout
  )
  if (health.healthy) {
    // Server is already running, just return client
    return {
      client: createOpencodeClient({ baseUrl }),
      server: null,
    }
  }

  // 4. Start the server
  const command = detection.path ?? serverConfig.command ?? "opencode"
  const args = [
    "serve",
    "--port",
    String(port),
    "--hostname",
    hostname,
    ...(serverConfig.serverArgs ?? []),
  ]

  const server = await startServer(command, args, {
    hostname,
    port,
    silent: serverConfig.silent ?? true,
  })

  // 5. Wait for server to be ready
  const startupTimeout = serverConfig.startupTimeout ?? 30000
  const readyResult = await waitForServer(baseUrl, {
    timeout: startupTimeout,
  })

  if (!readyResult.healthy) {
    // Server didn't start properly, try to clean up
    await server.shutdown().catch(() => {})
    throw new StartupTimeoutError(baseUrl, startupTimeout)
  }

  return {
    client: createOpencodeClient({ baseUrl }),
    server,
  }
}

export class OpenCodeError extends Error {
  constructor(
    message: string,
    public status?: number
  ) {
    super(message)
    this.name = "OpenCodeError"
  }
}

// Re-export types
export type { ServerHandle }
