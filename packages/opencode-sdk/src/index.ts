// Core client exports
export { createClient, type ClientOptions } from "./client.js"

// SDK exports
export {
  createOpencode,
  createOpencodeClient,
  OpencodeClient,
  OpenCodeError,
  type OpencodeConfig,
  type ServerConfig,
  type ClientConfig,
  type ServerHandle,
  // Session types
  type Session,
  type Message,
  type ChatOptions,
  // Files types
  type SearchOptions,
  type FindOptions,
  type ReadOptions,
  type SymbolsOptions,
  type SearchResult,
  type Symbol,
} from "./sdk.js"

// Error types
export {
  CommandNotFoundError,
  ServerUnhealthyError,
  ServerStartupError,
  StartupTimeoutError,
} from "./errors.js"

// Utility exports
export {
  detectCommand,
  getInstallationInstructions,
  type CommandDetectionResult,
} from "./utils/command-detector.js"

export {
  checkServerHealth,
  waitForServer,
  type HealthCheckResult,
} from "./utils/health-check.js"

export {
  startServer,
  stopServer,
  waitForServerReady,
  type ServerHandle as ProcessServerHandle,
  type StartServerOptions,
} from "./utils/server-process.js"

// CLI commands
export { chatCommand } from "./commands/chat.js"
export { sessionCommand } from "./commands/session.js"
export { filesCommand } from "./commands/files.js"
