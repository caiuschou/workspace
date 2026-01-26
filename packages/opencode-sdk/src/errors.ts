/**
 * Custom error types for OpenCode SDK
 */

/**
 * Base error for all OpenCode SDK errors
 */
export class OpenCodeError extends Error {
  constructor(message: string, public code?: string) {
    super(message)
    this.name = "OpenCodeError"
  }
}

/**
 * Error when opencode command is not found in PATH
 */
export class CommandNotFoundError extends OpenCodeError {
  constructor(command: string = "opencode") {
    super(
      `OpenCode command '${command}' not found in PATH.\n\n${getInstallationInstructions()}`,
      "COMMAND_NOT_FOUND"
    )
    this.name = "CommandNotFoundError"
  }
}

/**
 * Error when server health check fails
 */
export class ServerUnhealthyError extends OpenCodeError {
  constructor(url: string, reason?: string) {
    super(
      `OpenCode server at ${url} is not responding` +
        (reason ? `: ${reason}` : ""),
      "SERVER_UNHEALTHY"
    )
    this.name = "ServerUnhealthyError"
  }
}

/**
 * Error when server fails to start
 */
export class ServerStartupError extends OpenCodeError {
  constructor(url: string, output?: string) {
    super(
      `Failed to start OpenCode server at ${url}` +
        (output ? `\n\nServer output:\n${output}` : ""),
      "SERVER_STARTUP_FAILED"
    )
    this.name = "ServerStartupError"
  }
}

/**
 * Error when startup timeout is exceeded
 */
export class StartupTimeoutError extends OpenCodeError {
  constructor(url: string, timeout: number) {
    super(
      `OpenCode server at ${url} did not start within ${timeout}ms`,
      "STARTUP_TIMEOUT"
    )
    this.name = "StartupTimeoutError"
  }
}

/**
 * Get platform-specific installation instructions
 */
function getInstallationInstructions(): string {
  const platform = process.platform

  const instructions: Record<string, string> = {
    win32: `Install OpenCode using one of these methods:
  1. npm:      npm install -g opencode
  2. pnpm:     pnpm add -g opencode
  3. Download: https://opencode.ai/install`,

    darwin: `Install OpenCode using one of these methods:
  1. Homebrew: brew install opencode-ai/tap/opencode
  2. npm:      npm install -g opencode
  3. curl:     curl -fsSL https://opencode.ai/install | bash`,

    linux: `Install OpenCode using one of these methods:
  1. npm:      npm install -g opencode
  2. curl:     curl -fsSL https://opencode.ai/install | bash
  3. Download: https://opencode.ai/install`,
  }

  return instructions[platform] || instructions.linux
}
