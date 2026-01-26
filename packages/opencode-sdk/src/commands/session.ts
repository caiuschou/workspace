import { Command } from "commander"
import chalk from "chalk"
import ora from "ora"
import { createClient } from "../client.js"
import { getBaseUrl, shouldUseAutoStart } from "../config.js"
import { handleCommandError } from "../utils/error-handler.js"

export const sessionCommand = new Command("session")
  .description("Manage OpenCode sessions")

sessionCommand
  .command("list")
  .description("List all sessions")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (options) => {
    const spinner = ora("Fetching sessions...").start()

    try {
      const useAutoStart = shouldUseAutoStart(options.url)
      const baseUrl = useAutoStart ? undefined : getBaseUrl(options.url)

      const { client, server } = await createClient({
        baseUrl,
        autoStart: useAutoStart,
      })

      if (server) {
        spinner.text = "OpenCode server started"
      }

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

      if (server) {
        await server.shutdown()
      }
    } catch (error) {
      spinner.fail("Error occurred")
      handleCommandError(error, options.url)
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
      const useAutoStart = shouldUseAutoStart(options.url)
      const baseUrl = useAutoStart ? undefined : getBaseUrl(options.url)

      const { client, server } = await createClient({
        baseUrl,
        autoStart: useAutoStart,
      })

      if (server) {
        spinner.text = "OpenCode server started"
      }

      const session = await client.session.create({
        agent: options.agent,
      })

      spinner.succeed("Session created")
      console.log(chalk.green(`Session ID: ${session.id}`))

      if (server) {
        await server.shutdown()
      }
    } catch (error) {
      spinner.fail("Error occurred")
      handleCommandError(error, options.url)
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
      const useAutoStart = shouldUseAutoStart(options.url)
      const baseUrl = useAutoStart ? undefined : getBaseUrl(options.url)

      const { client, server } = await createClient({
        baseUrl,
        autoStart: useAutoStart,
      })

      if (server) {
        spinner.text = "OpenCode server started"
      }

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

      if (server) {
        await server.shutdown()
      }
    } catch (error) {
      spinner.fail("Error occurred")
      handleCommandError(error, options.url)
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
      const useAutoStart = shouldUseAutoStart(options.url)
      const baseUrl = useAutoStart ? undefined : getBaseUrl(options.url)

      const { client, server } = await createClient({
        baseUrl,
        autoStart: useAutoStart,
      })

      if (server) {
        spinner.text = "OpenCode server started"
      }

      await client.session.delete(sessionId)

      spinner.succeed("Session deleted")

      if (server) {
        await server.shutdown()
      }
    } catch (error) {
      spinner.fail("Error occurred")
      handleCommandError(error, options.url)
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
      const useAutoStart = shouldUseAutoStart(options.url)
      const baseUrl = useAutoStart ? undefined : getBaseUrl(options.url)

      const { client, server } = await createClient({
        baseUrl,
        autoStart: useAutoStart,
      })

      if (server) {
        spinner.text = "OpenCode server started"
      }

      await client.session.abort(sessionId)

      spinner.succeed("Operation aborted")

      if (server) {
        await server.shutdown()
      }
    } catch (error) {
      spinner.fail("Error occurred")
      handleCommandError(error, options.url)
      process.exit(1)
    }
  })
