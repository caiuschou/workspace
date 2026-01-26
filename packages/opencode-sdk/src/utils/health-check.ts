/**
 * Health check utilities for OpenCode server
 */

/**
 * Result of server health check
 */
export interface HealthCheckResult {
  /** Whether server is responding */
  healthy: boolean
  /** Response time in milliseconds */
  responseTime?: number
  /** Error details if unhealthy */
  error?: string
}

/**
 * Check if OpenCode server is responding at the given URL
 *
 * @param baseUrl - Server URL to check
 * @param timeout - Timeout in milliseconds (default: 3000)
 * @returns Health check result
 */
export async function checkServerHealth(
  baseUrl: string,
  timeout: number = 3000
): Promise<HealthCheckResult> {
  const startTime = performance.now()
  const controller = new AbortController()

  const timeoutId = setTimeout(() => controller.abort(), timeout)

  try {
    // Try to fetch the health endpoint or root
    const response = await fetch(`${baseUrl}/session`, {
      method: "GET",
      signal: controller.signal,
      // Don't throw on non-OK, we just want to know if server is running
    })

    const responseTime = performance.now() - startTime

    if (response.ok || response.status === 401) {
      // Server is running (401 means auth required but server is up)
      return { healthy: true, responseTime }
    }

    return {
      healthy: false,
      responseTime,
      error: `HTTP ${response.status}: ${response.statusText}`,
    }
  } catch (error) {
    const responseTime = performance.now() - startTime

    if (error instanceof Error && error.name === "AbortError") {
      return {
        healthy: false,
        responseTime: timeout,
        error: `Connection timeout after ${timeout}ms`,
      }
    }

    return {
      healthy: false,
      responseTime,
      error: error instanceof Error ? error.message : String(error),
    }
  } finally {
    clearTimeout(timeoutId)
  }
}

/**
 * Wait for server to become healthy with polling
 *
 * @param baseUrl - Server URL to check
 * @param options - Polling options
 * @returns Final health check result
 */
export async function waitForServer(
  baseUrl: string,
  options: {
    timeout?: number
    interval?: number
    signal?: AbortSignal
  } = {}
): Promise<HealthCheckResult> {
  const { timeout = 30000, interval = 500, signal } = options
  const startTime = Date.now()

  // Check for abort signal
  if (signal?.aborted) {
    return {
      healthy: false,
      error: "Operation aborted",
    }
  }

  while (Date.now() - startTime < timeout) {
    if (signal?.aborted) {
      return {
        healthy: false,
        error: "Operation aborted",
      }
    }

    const result = await checkServerHealth(baseUrl, Math.min(timeout, 3000))

    if (result.healthy) {
      return result
    }

    // Wait before next poll
    await new Promise((resolve) => setTimeout(resolve, interval))
  }

  return {
    healthy: false,
    error: `Server did not become healthy within ${timeout}ms`,
  }
}
