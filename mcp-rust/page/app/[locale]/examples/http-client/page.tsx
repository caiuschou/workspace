import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `use mcp_client::http::{HttpClientConfig, HttpClientTransport};
use mcp_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HTTP transport configuration
    let config = HttpClientConfig::new("http://localhost:8080/mcp")
        .auto_reconnect(true);
    
    let transport = HttpClientTransport::new(config);
    
    // Create client
    let client = Client::new(transport);
    client.connect().await?;
    
    // Initialize connection
    let init_result = client.initialize("mcp-http-client-example", "0.1.0").await?;
    println!("Connected to: {}", init_result.server_info.name);
    
    // List available tools
    let tools = client.list_tools().await?;
    println!("Available tools:");
    for tool in &tools.tools {
        println!("  - {}: {}", tool.name, tool.description.as_deref().unwrap_or(""));
    }
    
    // Call a tool
    let result = client.call_tool("echo", serde_json::json!({
        "message": "Hello from HTTP client!"
    })).await?;
    
    println!("Tool result: {:?}", result);
    
    Ok(())
}`;

export default async function HttpClientPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.httpClient");
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
        <CodeBlock code="cargo run -p mcp-http-client" language="bash" title="Terminal" />
      </section>

      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/http-client"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/http-client
          </a>
        </p>
      </section>
    </div>
  );
}
