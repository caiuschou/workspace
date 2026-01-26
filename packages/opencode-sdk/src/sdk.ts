/**
 * Local SDK implementation
 * TODO: Replace with @opencode-ai/sdk when available
 */

export interface OpencodeConfig {
  hostname?: string
  port?: number
  timeout?: number
  config?: Record<string, unknown>
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

class SessionAPI {
  constructor(private baseUrl: string) {}

  async create(options?: { agent?: string }): Promise<Session> {
    const res = await fetch(`${this.baseUrl}/session`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(options || {}),
    })
    if (!res.ok) throw new Error(`Failed to create session: ${res.statusText}`)
    return res.json() as unknown as Session
  }

  async list(): Promise<Session[]> {
    const res = await fetch(`${this.baseUrl}/session`)
    if (!res.ok) throw new Error(`Failed to list sessions: ${res.statusText}`)
    return res.json() as unknown as Session[]
  }

  async chat(sessionId: string, options: ChatOptions): Promise<void> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}/chat`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(options),
    })
    if (!res.ok) throw new Error(`Failed to send message: ${res.statusText}`)
  }

  async messages(sessionId: string): Promise<Message[]> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}/messages`)
    if (!res.ok) throw new Error(`Failed to get messages: ${res.statusText}`)
    return res.json() as unknown as Message[]
  }

  async delete(sessionId: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}`, {
      method: "DELETE",
    })
    if (!res.ok) throw new Error(`Failed to delete session: ${res.statusText}`)
  }

  async abort(sessionId: string): Promise<void> {
    const res = await fetch(`${this.baseUrl}/session/${sessionId}/abort`, {
      method: "POST",
    })
    if (!res.ok) throw new Error(`Failed to abort session: ${res.statusText}`)
  }
}

class FilesAPI {
  constructor(private baseUrl: string) {}

  async search(options: SearchOptions): Promise<SearchResult[]> {
    const params = new URLSearchParams({ query: options.query })
    if (options.path) params.set("path", options.path)
    const res = await fetch(`${this.baseUrl}/files/search?${params}`)
    if (!res.ok) throw new Error(`Failed to search: ${res.statusText}`)
    return res.json() as unknown as SearchResult[]
  }

  async find(options: FindOptions): Promise<string[]> {
    const params = new URLSearchParams({ pattern: options.pattern })
    const res = await fetch(`${this.baseUrl}/files/find?${params}`)
    if (!res.ok) throw new Error(`Failed to find files: ${res.statusText}`)
    return res.json() as unknown as string[]
  }

  async read(options: ReadOptions): Promise<string> {
    const params = new URLSearchParams({ path: options.path })
    const res = await fetch(`${this.baseUrl}/files/read?${params}`)
    if (!res.ok) throw new Error(`Failed to read file: ${res.statusText}`)
    return res.text()
  }

  async symbols(options: SymbolsOptions): Promise<Symbol[]> {
    const params = new URLSearchParams({ query: options.query })
    const res = await fetch(`${this.baseUrl}/files/symbols?${params}`)
    if (!res.ok) throw new Error(`Failed to find symbols: ${res.statusText}`)
    return res.json() as unknown as Symbol[]
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

export async function createOpencode(config?: OpencodeConfig): Promise<{
  client: OpencodeClient
  server: unknown
}> {
  const hostname = config?.hostname || "127.0.0.1"
  const port = config?.port || 4096
  const baseUrl = `http://${hostname}:${port}`

  // TODO: Start server process here when implementing full SDK
  const client = createOpencodeClient({ baseUrl })

  return { client, server: null }
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
