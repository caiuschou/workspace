import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `use mcp_client::stdio::{StdioClientTransport, StdioServerParameters};
use mcp_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure server parameters
    let params = StdioServerParameters {
        command: "cargo".to_string(),
        args: vec!["run".to_string(), "-p".to_string(), "mcp-filesystem-server".to_string()],
        env: None,
    };
    
    let transport = StdioClientTransport::new(params);
    let client = Client::new(transport);
    client.connect().await?;
    
    // Initialize
    let init_result = client.initialize("filesystem-client", "0.1.0").await?;
    println!("Connected to: {}", init_result.server_info.name);
    
    // List directory
    let result = client.call_tool("list_directory", serde_json::json!({
        "path": "."
    })).await?;
    
    println!("Directory contents: {:?}", result);
    
    Ok(())
}`;

export default async function FilesystemClientPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.filesystemClient");
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
        <CodeBlock code="cargo run -p mcp-filesystem-client" language="bash" title="Terminal" />
      </section>

      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/mcp-filesystem-client"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/mcp-filesystem-client
          </a>
        </p>
      </section>
    </div>
  );
}
