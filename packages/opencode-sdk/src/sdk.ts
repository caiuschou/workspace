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
import { logger } from "./utils/logger.js"

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
 * Safely parse JSON response with better error handling and logging
 */
async function parseJSON<T>(res: Response, errorMessage: string, url: string): Promise<T> {
  const contentType = res.headers.get("content-type")
  const headers: Record<string, string> = {}
  res.headers.forEach((value, key) => {
    headers[key] = value
  })

  // Get response body for logging
  const text = await res.text().catch(() => "")

  // Log the response
  logger.logResponse(url, res.status, res.statusText, headers, text)

  // Check if response is actually JSON
  if (!contentType?.includes("application/json")) {
    logger.error("Non-JSON response received", {
      url,
      contentType,
      bodyPreview: text.slice(0, 1000),
    })

    // Write full HTML response to log file
    if (text && text.startsWith("<")) {
      logger.divider("FULL HTML RESPONSE")
      logger.writeRaw("\n" + text + "\n")
      logger.divider()
    }

    throw new Error(
      `${errorMessage}\n` +
      `Expected JSON response but received: ${contentType || "unknown content-type"}\n` +
      (text ? `Response body: ${text.slice(0, 200)}${text.length > 200 ? "..." : ""}` : "")
    )
  }

  try {
    return JSON.parse(text) as T
  } catch (err) {
    logger.error("Failed to parse JSON", {
      url,
      parseError: String(err),
      bodyPreview: text.slice(0, 1000),
    })

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
async function checkResponse(res: Response, operation: string, url: string): Promise<void> {
  if (!res.ok) {
    const contentType = res.headers.get("content-type")
    const headers: Record<string, string> = {}
    res.headers.forEach((value, key) => {
      headers[key] = value
    })

    let details = `HTTP ${res.status}: ${res.statusText}`

    // Try to get more details from response body
    let body = ""
    if (contentType?.includes("application/json")) {
      try {
        const json = await res.json()
        body = JSON.stringify(json, null, 2)
        details += `\n${body}`
      } catch {}
    } else {
      body = await res.text().catch(() => "")
      if (body) {
        details += `\n${body.slice(0, 300)}${body.length > 300 ? "..." : ""}`
      }
    }

    // Log the error response
    logger.logResponse(url, res.status, res.statusText, headers, body)
    logger.error(`HTTP error during ${operation}`, {
      url,
      status: res.status,
      statusText: res.statusText,
      body: body.slice(0, 5000),
    })

    // Write full response to log if it's HTML (likely an error page)
    if (body && body.startsWith("<")) {
      logger.divider("FULL HTML ERROR RESPONSE")
      logger.writeRaw("\n" + body + "\n")
      logger.divider()
    }

    throw new Error(`Failed to ${operation}: ${details}`)
  }
}

class SessionAPI {
  constructor(private baseUrl: string) {}

  async create(options?: { agent?: string }): Promise<Session> {
    const url = `${this.baseUrl}/session`
    logger.logRequest("POST", url, options)

    const res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(options || {}),
    })

    await checkResponse(res, "create session", url)
    return parseJSON<Session>(res, "Failed to parse session response", url)
  }

  async list(): Promise<Session[]> {
    const url = `${this.baseUrl}/session`
    logger.logRequest("GET", url)

    const res = await fetch(url)
    await checkResponse(res, "list sessions", url)
    return parseJSON<Session[]>(res, "Failed to parse sessions response", url)
  }

  async chat(sessionId: string, options: ChatOptions): Promise<void> {
    const url = `${this.baseUrl}/session/${sessionId}/chat`
    logger.logRequest("POST", url, options)

    const res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(options),
    })

    await checkResponse(res, "send message", url)
  }

  async messages(sessionId: string): Promise<Message[]> {
    const url = `${this.baseUrl}/session/${sessionId}/messages`
    logger.logRequest("GET", url)

    const res = await fetch(url)
    await checkResponse(res, "get messages", url)
    return parseJSON<Message[]>(res, "Failed to parse messages response", url)
  }

  async delete(sessionId: string): Promise<void> {
    const url = `${this.baseUrl}/session/${sessionId}`
    logger.logRequest("DELETE", url)

    const res = await fetch(url, {
      method: "DELETE",
    })

    await checkResponse(res, "delete session", url)
  }

  async abort(sessionId: string): Promise<void> {
    const url = `${this.baseUrl}/session/${sessionId}/abort`
    logger.logRequest("POST", url)

    const res = await fetch(url, {
      method: "POST",
    })

    await checkResponse(res, "abort session", url)
  }
}

class FilesAPI {
  constructor(private baseUrl: string) {}

  async search(options: SearchOptions): Promise<SearchResult[]> {
    const params = new URLSearchParams({ query: options.query })
    if (options.path) params.set("path", options.path)
    const url = `${this.baseUrl}/files/search?${params}`
    logger.logRequest("GET", url)

    const res = await fetch(url)
    await checkResponse(res, "search files", url)
    return parseJSON<SearchResult[]>(res, "Failed to parse search response", url)
  }

  async find(options: FindOptions): Promise<string[]> {
    const params = new URLSearchParams({ pattern: options.pattern })
    const url = `${this.baseUrl}/files/find?${params}`
    logger.logRequest("GET", url)

    const res = await fetch(url)
    await checkResponse(res, "find files", url)
    return parseJSON<string[]>(res, "Failed to parse files response", url)
  }

  async read(options: ReadOptions): Promise<string> {
    const params = new URLSearchParams({ path: options.path })
    const url = `${this.baseUrl}/files/read?${params}`
    logger.logRequest("GET", url)

    const res = await fetch(url)
    await checkResponse(res, "read file", url)
    return res.text()
  }

  async symbols(options: SymbolsOptions): Promise<Symbol[]> {
    const params = new URLSearchParams({ query: options.query })
    const url = `${this.baseUrl}/files/symbols?${params}`
    logger.logRequest("GET", url)

    const res = await fetch(url)
    await checkResponse(res, "find symbols", url)
    return parseJSON<Symbol[]>(res, "Failed to parse symbols response", url)
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

  logger.info("createOpencode called", {
    hostname,
    port,
    baseUrl,
    autoStart: serverConfig.autoStart !== false,
  })

  // 1. If autoStart is explicitly disabled, just return client
  if (serverConfig.autoStart === false) {
    logger.info("Auto-start disabled, returning client without server")
    return {
      client: createOpencodeClient({ baseUrl }),
      server: null,
    }
  }

  // 2. Check if opencode command is available
  logger.debug("Checking for opencode command...")
  const detection = await detectCommand(serverConfig.command ?? "opencode")

  if (!detection.available) {
    logger.error("opencode command not found", { detection })
    throw new CommandNotFoundError(serverConfig.command ?? "opencode")
  }

  logger.info("opencode command found", {
    path: detection.path,
    version: detection.version,
  })

  // 3. Check if server is already running
  logger.debug("Checking server health...")
  const health = await checkServerHealth(
    baseUrl,
    serverConfig.healthCheckTimeout
  )

  if (health.healthy) {
    logger.info("Server is already running", {
      baseUrl,
      responseTime: health.responseTime,
    })
    // Server is already running, just return client
    return {
      client: createOpencodeClient({ baseUrl }),
      server: null,
    }
  }

  logger.info("Server not running, starting it...", {
    healthError: health.error,
  })

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

  logger.info("Starting opencode server", {
    command,
    args,
  })

  const server = await startServer(command, args, {
    hostname,
    port,
    silent: serverConfig.silent ?? true,
  })

  logger.info("Server process started", {
    pid: server.pid,
  })

  // 5. Wait for server to be ready
  const startupTimeout = serverConfig.startupTimeout ?? 30000
  logger.info("Waiting for server to be ready...", { timeout: startupTimeout })

  const readyResult = await waitForServer(baseUrl, {
    timeout: startupTimeout,
  })

  if (!readyResult.healthy) {
    logger.error("Server failed to become healthy", {
      baseUrl,
      startupTimeout,
      healthResult: readyResult,
    })
    // Server didn't start properly, try to clean up
    await server.shutdown().catch(() => {})
    throw new StartupTimeoutError(baseUrl, startupTimeout)
  }

  logger.info("Server is ready", {
    baseUrl,
    responseTime: readyResult.responseTime,
  })

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
