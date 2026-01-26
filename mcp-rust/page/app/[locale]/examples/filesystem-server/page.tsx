import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `// Filesystem server provides file operations via stdio transport
// Supports: read_file, write_file, list_directory, etc.

use mcp_server::stdio::StdioServerTransport;
use mcp_server::McpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = McpServer::new(server_info, server_options);
    
    // Register filesystem tools
    server.register_tool("read_file", /* ... */);
    server.register_tool("write_file", /* ... */);
    server.register_tool("list_directory", /* ... */);
    
    // Start stdio transport
    let transport = StdioServerTransport::new();
    transport.serve(server).await?;
    
    Ok(())
}`;

export default async function FilesystemServerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.filesystemServer");
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
        <CodeBlock code="cargo run -p mcp-filesystem-server" language="bash" title="Terminal" />
      </section>

      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/mcp-filesystem-server"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/mcp-filesystem-server
          </a>
        </p>
      </section>
    </div>
  );
}
