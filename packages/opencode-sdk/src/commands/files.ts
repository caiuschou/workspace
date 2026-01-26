import { Command } from "commander"
import chalk from "chalk"
import ora from "ora"
import { createClient } from "../client.js"
import { getBaseUrl } from "../config.js"

export const filesCommand = new Command("files")
  .description("File operations with OpenCode")

filesCommand
  .command("search <query>")
  .description("Search for text in files")
  .option("-p, --path <path>", "Path to search in")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (query: string, options) => {
    const spinner = ora("Searching...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      const results = await client.files.search({
        query,
        path: options.path,
      })

      spinner.stop()

      if (!results || results.length === 0) {
        console.log(chalk.yellow("No results found"))
        return
      }

      console.log(chalk.green(`Found ${results.length} result(s):\n`))

      for (const result of results) {
        console.log(chalk.cyan(result.path || result))
        if (result.line) {
          console.log(`  Line ${result.line}: ${result.content}`)
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

filesCommand
  .command("find <pattern>")
  .description("Find files matching a pattern")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (pattern: string, options) => {
    const spinner = ora("Finding files...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      const files = await client.files.find({
        pattern,
      })

      spinner.stop()

      if (!files || files.length === 0) {
        console.log(chalk.yellow("No files found"))
        return
      }

      console.log(chalk.green(`Found ${files.length} file(s):\n`))

      for (const file of files) {
        console.log(chalk.cyan(file))
      }
    } catch (error) {
      spinner.fail("Error occurred")
      if (error instanceof Error) {
        console.error(chalk.red(error.message))
      }
      process.exit(1)
    }
  })

filesCommand
  .command("read <path>")
  .description("Read a file")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (path: string, options) => {
    const spinner = ora("Reading file...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      const content = await client.files.read({
        path,
      })

      spinner.stop()

      console.log(content)
    } catch (error) {
      spinner.fail("Error occurred")
      if (error instanceof Error) {
        console.error(chalk.red(error.message))
      }
      process.exit(1)
    }
  })

filesCommand
  .command("symbols <query>")
  .description("Find symbols in the codebase")
  .option("-u, --url <url>", "OpenCode server URL (overrides OPENCODE_BASE_URL)")
  .action(async (query: string, options) => {
    const spinner = ora("Finding symbols...").start()

    try {
      const { client } = await createClient({ baseUrl: getBaseUrl(options.url) })
      const symbols = await client.files.symbols({
        query,
      })

      spinner.stop()

      if (!symbols || symbols.length === 0) {
        console.log(chalk.yellow("No symbols found"))
        return
      }

      console.log(chalk.green(`Found ${symbols.length} symbol(s):\n`))

      for (const symbol of symbols) {
        console.log(chalk.cyan(symbol.name || symbol))
        if (symbol.path) {
          console.log(`  Path: ${symbol.path}`)
        }
        if (symbol.kind) {
          console.log(`  Kind: ${symbol.kind}`)
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
