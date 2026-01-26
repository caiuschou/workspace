import { Command } from "commander"
import chalk from "chalk"
import ora from "ora"
import { createClient } from "../client.js"
import { getBaseUrl, shouldUseAutoStart } from "../config.js"
import { handleCommandError } from "../utils/error-handler.js"

export const chatCommand = new Command("chat")
  .description("Send a message to OpenCode and get a response")
  .argument("<message>", "Message to send")
  .option("-a, --agent <agent>", "Agent to use (e.g., build, code)")
  .option("-f, --files <files...>", "Files to attach to the message")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .option("-s, --session <id>", "Existing session ID to use")
  .action(async (message: string, options) => {
    const spinner = ora("Connecting to OpenCode...").start()

    try {
      const useAutoStart = shouldUseAutoStart(options.url)
      const baseUrl = useAutoStart ? undefined : getBaseUrl(options.url)

      const { client, server } = await createClient({
        baseUrl,
        autoStart: useAutoStart,
      })

      // If we started a server, update spinner
      if (server) {
        spinner.text = "OpenCode server started"
      }

      spinner.text = "Creating session..."

      let sessionId = options.session

      if (!sessionId) {
        const session = await client.session.create({
          agent: options.agent,
        })
        sessionId = session.id
        spinner.text = `Session created: ${sessionId}`
      }

      spinner.text = "Sending message..."

      await client.session.chat(sessionId, {
        content: message,
        files: options.files,
      })

      spinner.text = "Waiting for response..."

      const messages = await client.session.messages(sessionId)
      const lastMessage = messages[messages.length - 1]

      spinner.stop()

      console.log(chalk.green("\nResponse:"))
      console.log(lastMessage?.content || "No response received")
      console.log(chalk.gray(`\nSession ID: ${sessionId}`))

      // Shutdown server if we started it
      if (server) {
        await server.shutdown()
      }
    } catch (error) {
      spinner.fail("Error occurred")
      handleCommandError(error, options.url)
      process.exit(1)
    }
  })
