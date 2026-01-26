import { Command } from "commander"
import chalk from "chalk"
import ora from "ora"
import { createClient } from "../client.js"
import { getBaseUrl } from "../config.js"

export const sessionCommand = new Command("session")
  .description("Manage OpenCode sessions")

sessionCommand
  .command("list")
  .description("List all sessions")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (options) => {
    const spinner = ora("Fetching sessions...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      const sessions = await client.session.list()

      spinner.stop()

      if (sessions.length === 0) {
        console.log(chalk.yellow("No sessions found"))
        return
      }

      console.log(chalk.green(`Found ${sessions.length} session(s):\n`))

      for (const session of sessions) {
        console.log(chalk.cyan(`ID: ${session.id}`))
        if (session.title) {
          console.log(`  Title: ${session.title}`)
        }
        console.log("")
      }
    } catch (error) {
      spinner.fail("Error occurred")
      if (error instanceof Error) {
        console.error(chalk.red(error.message))
      }
      process.exit(1)
    }
  })

sessionCommand
  .command("create")
  .description("Create a new session")
  .option("-a, --agent <agent>", "Agent to use")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (options) => {
    const spinner = ora("Creating session...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      const session = await client.session.create({
        agent: options.agent,
      })

      spinner.succeed("Session created")
      console.log(chalk.green(`Session ID: ${session.id}`))
    } catch (error) {
      spinner.fail("Error occurred")
      if (error instanceof Error) {
        console.error(chalk.red(error.message))
      }
      process.exit(1)
    }
  })

sessionCommand
  .command("messages <sessionId>")
  .description("Get messages from a session")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (sessionId: string, options) => {
    const spinner = ora("Fetching messages...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      const messages = await client.session.messages(sessionId)

      spinner.stop()

      if (messages.length === 0) {
        console.log(chalk.yellow("No messages in this session"))
        return
      }

      console.log(chalk.green(`Found ${messages.length} message(s):\n`))

      for (const msg of messages) {
        const role = msg.role === "user" ? chalk.blue("User") : chalk.green("Assistant")
        console.log(`${role}:`)
        console.log(msg.content)
        console.log("")
      }
    } catch (error) {
      spinner.fail("Error occurred")
      if (error instanceof Error) {
        console.error(chalk.red(error.message))
      }
      process.exit(1)
    }
  })

sessionCommand
  .command("delete <sessionId>")
  .description("Delete a session")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (sessionId: string, options) => {
    const spinner = ora("Deleting session...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      await client.session.delete(sessionId)

      spinner.succeed("Session deleted")
    } catch (error) {
      spinner.fail("Error occurred")
      if (error instanceof Error) {
        console.error(chalk.red(error.message))
      }
      process.exit(1)
    }
  })

sessionCommand
  .command("abort <sessionId>")
  .description("Abort an ongoing operation in a session")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (sessionId: string, options) => {
    const spinner = ora("Aborting operation...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      await client.session.abort(sessionId)

      spinner.succeed("Operation aborted")
    } catch (error) {
      spinner.fail("Error occurred")
      if (error instanceof Error) {
        console.error(chalk.red(error.message))
      }
      process.exit(1)
    }
  })
