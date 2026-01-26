/**
 * Server process management utilities
 */

import { spawn } from "child_process"
import { checkServerHealth } from "./health-check.js"

/**
 * Represents a managed server process
 */
export interface ServerHandle {
  /** Process ID of the server */
  pid: number
  /** Whether this SDK instance started the server */
  managed: boolean
  /** Shut down the server if we started it */
  shutdown: () => Promise<void>
}

/**
 * Options for starting the server
 */
export interface StartServerOptions {
  /** Hostname for the server */
  hostname?: string
  /** Port for the server */
  port?: number
  /** Suppress stdout/stderr output */
  silent?: boolean
  /** Callback for stdout data */
  onStdout?: (data: string) => void
  /** Callback for stderr data */
  onStderr?: (data: string) => void
}

/**
 * Start the OpenCode server as a detached process
 *
 * @param command - Command to execute (e.g., "opencode")
 * @param args - Arguments to pass (e.g., ["serve", "--port", "4096"])
 * @param options - Spawn options
 * @returns Server handle with shutdown method
 */
export async function startServer(
  command: string,
  args: string[],
  options: StartServerOptions = {}
): Promise<ServerHandle> {
  const { silent = false, onStdout, onStderr } = options

  return new Promise((resolve, reject) => {
    // Spawn the process in detached mode
    // This allows it to continue running independently
    const childProcess = spawn(command, args, {
      detached: true,
      stdio: silent ? ["ignore", "ignore", "ignore"] : ["ignore", "pipe", "pipe"],
      windowsHide: true,
    })

    let stdoutOutput = ""
    let stderrOutput = ""

    if (!silent) {
      if (childProcess.stdout) {
        childProcess.stdout.on("data", (data) => {
          const text = data.toString()
          stdoutOutput += text
          onStdout?.(text)
        })
      }

      if (childProcess.stderr) {
        childProcess.stderr.on("data", (data) => {
          const text = data.toString()
          stderrOutput += text
          onStderr?.(text)
        })
      }
    }

    // Unref the process so it doesn't keep the parent alive
    childProcess.unref()

    // Handle process exit (this happens quickly when daemonizing)
    childProcess.on("exit", (code, signal) => {
      if (code !== 0 && code !== null) {
        reject(
          new Error(
            `Server process exited with code ${code}${stderrOutput ? `\n${stderrOutput}` : ""}`
          )
        )
      }
    })

    // Handle spawn errors
    childProcess.on("error", (error) => {
      reject(new Error(`Failed to spawn server process: ${error.message}`))
    })

    // If we get here, the process was spawned successfully
    // The actual server may still be starting up
    const handle: ServerHandle = {
      pid: childProcess.pid ?? 0,
      managed: true,
      shutdown: () => shutdownServer(childProcess.pid ?? 0, silent),
    }

    // Small delay to let the process initialize
    setTimeout(() => resolve(handle), 100)
  })
}

/**
 * Terminate a server process by PID
 *
 * @param pid - Process ID to terminate
 * @param signal - Signal to send (default: "SIGTERM")
 * @param timeout - Time before force kill (default: 5000ms)
 */
export async function stopServer(
  pid: number,
  signal: NodeJS.Signals = "SIGTERM",
  timeout: number = 5000
): Promise<void> {
  if (pid <= 0) {
    return
  }

  try {
    // Try graceful shutdown first
    process.kill(pid, signal)

    // Wait for process to exit, then force kill if needed
    const startTime = Date.now()
    while (Date.now() - startTime < timeout) {
      try {
        // Check if process still exists
        process.kill(pid, 0)
        // Process still exists, wait a bit
        await new Promise((resolve) => setTimeout(resolve, 100))
      } catch {
        // Process doesn't exist anymore, we're done
        return
      }
    }

    // Force kill if still running
    try {
      process.kill(pid, "SIGKILL")
    } catch {
      // Already dead
    }
  } catch (error) {
    // Process may have already exited
    if (
      error instanceof Error &&
      !error.message.includes("ESRCH") &&
      !error.message.includes("process doesn't exist")
    ) {
      throw error
    }
  }
}

/**
 * Shutdown a server handle
 */
async function shutdownServer(
  pid: number,
  silent: boolean
): Promise<void> {
  if (pid <= 0) {
    return
  }

  try {
    await stopServer(pid, "SIGTERM", 5000)
  } catch (error) {
    if (!silent) {
      console.error(`Warning: Failed to gracefully shutdown server: ${error}`)
    }
    // Try force kill
    try {
      await stopServer(pid, "SIGKILL", 1000)
    } catch {
      // Ignore
    }
  }
}

/**
 * Wait for server to be ready after starting
 *
 * @param baseUrl - Server URL to check
 * @param timeout - Maximum time to wait (default: 30000ms)
 */
export async function waitForServerReady(
  baseUrl: string,
  timeout: number = 30000
): Promise<void> {
  const startTime = Date.now()
  const interval = 500

  while (Date.now() - startTime < timeout) {
    const result = await checkServerHealth(baseUrl, 2000)
    if (result.healthy) {
      return
    }
    await new Promise((resolve) => setTimeout(resolve, interval))
  }

  throw new Error(`Server did not become ready within ${timeout}ms`)
}
