#!/usr/bin/env node
import { Command } from "commander"
import { config } from "dotenv"
import { resolve } from "path"
import { existsSync } from "fs"
import { fileURLToPath } from "url"
import { dirname } from "path"

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

// Load .env file from current directory or project root
const envPaths = [
  resolve(process.cwd(), ".env"),
  resolve(__dirname, "../.env"),
]

for (const envPath of envPaths) {
  if (existsSync(envPath)) {
    config({ path: envPath })
    break
  }
}

import { chatCommand } from "./commands/chat.js"
import { sessionCommand } from "./commands/session.js"
import { filesCommand } from "./commands/files.js"

const program = new Command()

program
  .name("opencode-sdk")
  .description("CLI tool for executing tasks with OpenCode SDK")
  .version("0.1.0")

program.addCommand(chatCommand)
program.addCommand(sessionCommand)
program.addCommand(filesCommand)

program.parse()
