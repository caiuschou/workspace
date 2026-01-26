/**
 * Error handling utilities for CLI commands
 */

import chalk from "chalk"
import { getBaseUrl } from "../config.js"

/**
 * Handle command errors with user-friendly messages
 */
export function handleCommandError(error: unknown, customUrl?: string): void {
  const baseUrl = getBaseUrl(customUrl)

  if (error instanceof Error) {
    // Check for common fetch errors
    if (error.message.includes("fetch failed") || error.message.includes("ECONNREFUSED")) {
      console.error(chalk.red("\n✖ Failed to connect to OpenCode server"))
      console.error(chalk.yellow(`\nTried to connect to: ${baseUrl}`))
      console.error(chalk.gray("\nPossible solutions:"))
      console.error(chalk.gray("  1. Make sure the OpenCode server is running:"))
      console.error(chalk.gray("     opencode serve"))
      console.error(chalk.gray("  2. Or specify a different URL with --url option"))
      console.error(chalk.gray("  3. Or set OPENCODE_BASE_URL environment variable"))
      return
    }

    // Check for timeout errors
    if (error.message.includes("timeout") || error.message.includes("ETIMEDOUT")) {
      console.error(chalk.red("\n✖ Connection timeout"))
      console.error(chalk.yellow(`\nThe server at ${baseUrl} took too long to respond`))
      console.error(chalk.gray("\nPossible solutions:"))
      console.error(chalk.gray("  1. Check if the server is running properly"))
      console.error(chalk.gray("  2. Check your network connection"))
      console.error(chalk.gray("  3. Try again with a different URL using --url"))
      return
    }

    // Check for OpenCode custom errors
    if (error.name === "CommandNotFoundError") {
      console.error(chalk.red(`\n${error.message}`))
      return
    }

    if (error.name === "ServerUnhealthyError" || error.name === "StartupTimeoutError") {
      console.error(chalk.red(`\n${error.message}`))
      console.error(chalk.gray("\nPossible solutions:"))
      console.error(chalk.gray("  1. Make sure the OpenCode server is running"))
      console.error(chalk.gray("  2. Try: opencode serve"))
      console.error(chalk.gray("  3. Check the server logs for errors"))
      return
    }

    // Generic error with more context
    console.error(chalk.red(`\n✖ ${error.message}`))
    console.error(chalk.gray(`\nServer URL: ${baseUrl}`))
    if (error.stack) {
      console.error(chalk.gray("\nStack trace:"))
      console.error(chalk.gray(error.stack.split("\n").slice(1, 4).join("\n")))
    }
  } else {
    console.error(chalk.red(`\n✖ Unknown error: ${String(error)}`))
  }
}
