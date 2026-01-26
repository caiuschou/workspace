/**
 * Command detection utilities
 */

import { spawn } from "child_process"

/**
 * Result of command availability check
 */
export interface CommandDetectionResult {
  /** Whether the command is available */
  available: boolean
  /** Full path to the command (if found) */
  path?: string
  /** Version string (if detectable) */
  version?: string
}

/**
 * Check if a command is available in PATH
 *
 * Uses `which` on Unix/macOS and `where` on Windows.
 *
 * @param commandName - Name of command to check (default: "opencode")
 * @returns Detection result with availability status and path
 */
export async function detectCommand(
  commandName: string = "opencode"
): Promise<CommandDetectionResult> {
  const isWindows = process.platform === "win32"
  const whichCommand = isWindows ? "where" : "which"

  return new Promise((resolve) => {
    const process = spawn(whichCommand, [commandName], {
      stdio: ["ignore", "pipe", "pipe"],
      windowsHide: true,
    })

    let stdout = ""
    let stderr = ""

    process.stdout?.on("data", (data) => {
      stdout += data.toString()
    })

    process.stderr?.on("data", (data) => {
      stderr += data.toString()
    })

    process.on("close", (code) => {
      if (code === 0 && stdout.trim()) {
        // Command found
        const lines = stdout.trim().split(/\r?\n/)
        const commandPath = lines[0]?.trim()
        resolve({
          available: true,
          path: commandPath,
        })
      } else {
        // Command not found
        resolve({ available: false })
      }
    })

    process.on("error", () => {
      // Spawn failed (e.g., which command not found)
      resolve({ available: false })
    })
  })
}

/**
 * Get platform-specific installation instructions
 */
export function getInstallationInstructions(): string {
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
