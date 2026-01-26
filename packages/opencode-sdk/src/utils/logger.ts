/**
 * Logging utilities for CLI
 */

import { existsSync, mkdirSync, appendFileSync, writeFileSync } from "fs"
import { join, dirname } from "path"
import { homedir } from "os"

export enum LogLevel {
  DEBUG = "DEBUG",
  INFO = "INFO",
  WARN = "WARN",
  ERROR = "ERROR",
}

interface LogEntry {
  timestamp: string
  level: LogLevel
  message: string
  data?: Record<string, unknown>
}

class Logger {
  private logFile: string | null = null
  private sessionStartTime: string

  constructor() {
    this.sessionStartTime = new Date().toISOString()
  }

  /**
   * Initialize the logger with a log file in the user's home directory
   */
  init(): void {
    const logsDir = join(homedir(), ".opencode-sdk", "logs")

    // Create logs directory if it doesn't exist
    if (!existsSync(logsDir)) {
      mkdirSync(logsDir, { recursive: true })
    }

    // Create log file with timestamp
    const timestamp = new Date().toISOString().replace(/[:.]/g, "-")
    this.logFile = join(logsDir, `cli-${timestamp}.log`)

    // Write session start header
    this.writeLogHeader()
  }

  private writeLogHeader(): void {
    if (!this.logFile) return

    const header = `
================================================================================
OpenCode SDK CLI Log - Session Started: ${this.sessionStartTime}
Log File: ${this.logFile}
================================================================================

`
    writeFileSync(this.logFile, header, "utf-8")
  }

  /**
   * Write a log entry
   */
  private log(level: LogLevel, message: string, data?: Record<string, unknown>): void {
    if (!this.logFile) return

    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      data,
    }

    const logLine = `[${entry.timestamp}] [${entry.level}] ${entry.message}\n`
    appendFileSync(this.logFile, logLine, "utf-8")

    if (data) {
      try {
        const dataLine = `${JSON.stringify(data, null, 2)}\n`
        appendFileSync(this.logFile, dataLine, "utf-8")
      } catch {
        appendFileSync(this.logFile, "(Data could not be serialized)\n", "utf-8")
      }
    }
  }

  debug(message: string, data?: Record<string, unknown>): void {
    this.log(LogLevel.DEBUG, message, data)
  }

  info(message: string, data?: Record<string, unknown>): void {
    this.log(LogLevel.INFO, message, data)
  }

  warn(message: string, data?: Record<string, unknown>): void {
    this.log(LogLevel.WARN, message, data)
  }

  error(message: string, data?: Record<string, unknown>): void {
    this.log(LogLevel.ERROR, message, data)
  }

  /**
   * Log an HTTP request
   */
  logRequest(method: string, url: string, body?: unknown): void {
    this.debug("HTTP Request", {
      method,
      url,
      body: body ?? undefined,
    })
  }

  /**
   * Log an HTTP response
   */
  logResponse(url: string, status: number, statusText: string, headers: Record<string, string>, body?: string): void {
    this.debug("HTTP Response", {
      url,
      status,
      statusText,
      headers,
      body: body ? body.slice(0, 5000) + (body.length > 5000 ? "... (truncated)" : "") : undefined,
    })
  }

  /**
   * Log a full error with stack trace
   */
  logError(error: Error | unknown, context?: Record<string, unknown>): void {
    const errorData: Record<string, unknown> = { ...context }

    if (error instanceof Error) {
      errorData.name = error.name
      errorData.message = error.message
      errorData.stack = error.stack
    } else {
      errorData.value = String(error)
    }

    this.log(LogLevel.ERROR, "Error occurred", errorData)
  }

  /**
   * Get the log file path
   */
  getLogFilePath(): string | null {
    return this.logFile
  }

  /**
   * Write a divider to separate sections in the log
   */
  divider(title?: string): void {
    if (!this.logFile) return

    const line = title
      ? `\n${"=".repeat(80)}\n${title}\n${"=".repeat(80)}\n\n`
      : "\n" + "-".repeat(80) + "\n\n"

    appendFileSync(this.logFile, line, "utf-8")
  }

  /**
   * Write raw content to the log file
   */
  writeRaw(content: string): void {
    if (!this.logFile) return
    appendFileSync(this.logFile, content, "utf-8")
  }
}

// Global logger instance
const globalLogger = new Logger()

export { globalLogger as logger, Logger }
export type { LogEntry }
