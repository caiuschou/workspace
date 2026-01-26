import { createOpencode, createOpencodeClient } from "./sdk.js"

export interface ClientOptions {
  baseUrl?: string
  timeout?: number
}

export async function createClient(options: ClientOptions = {}) {
  if (options.baseUrl) {
    return {
      client: createOpencodeClient({
        baseUrl: options.baseUrl,
      }),
      server: null,
    }
  }

  return await createOpencode({
    timeout: options.timeout,
  })
}
