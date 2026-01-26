import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `use mcp_server::McpServer;
use mcp_core::types::LoggingLevel;

// Enable logging capability
server_options.capabilities = Some(ServerCapabilities {
    logging: Some(LoggingCapabilities {}),
    ..Default::default()
});

// Send log messages
server.log(LoggingLevel::Info, "Server started", None).await?;
server.log(LoggingLevel::Debug, "Processing request", Some(json!({
    "request_id": "123"
}))).await?;
server.log(LoggingLevel::Error, "Operation failed", None).await?;`;

export default async function LoggingServerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.loggingServer");
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
        <CodeBlock code="cargo run -p mcp-logging-server" language="bash" title="Terminal" />
      </section>

      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/logging-server"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/logging-server
          </a>
        </p>
      </section>
    </div>
  );
}
