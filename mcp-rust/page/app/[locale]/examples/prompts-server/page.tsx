import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `use mcp_server::McpServer;

// Register prompts with dynamic message generation
server.register_prompt(
    "greeting",
    "Generate a greeting message",
    vec![PromptArgument {
        name: "name".to_string(),
        description: Some("Name of the person to greet".to_string()),
        required: true,
    }],
    |args| async move {
        let name = args.get("name").unwrap_or(&"World".to_string());
        Ok(GetPromptResult {
            description: Some("A friendly greeting".to_string()),
            messages: vec![PromptMessage {
                role: Role::User,
                content: PromptContent::Text(TextContent::new(
                    format!("Please greet {} warmly!", name)
                )),
            }],
        })
    },
)?;`;

export default async function PromptsServerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.promptsServer");
  const common = await getTranslations("common");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{common("basicExample")}</h2>
        <CodeBlock code={mainCode} language="rust" title="src/main.rs" />
      </section>

      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{common("running")}</h2>
        <CodeBlock code="cargo run -p mcp-prompts-server" language="bash" title="Terminal" />
      </section>

      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/prompts-server"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/prompts-server
          </a>
        </p>
      </section>
    </div>
  );
}
