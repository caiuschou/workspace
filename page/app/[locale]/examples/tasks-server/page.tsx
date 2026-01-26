import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `use mcp_server::McpServer;

// Register a long-running task
server.register_tool(
    "long_task",
    "Perform a long-running operation",
    schema,
    |params, context| async move {
        // Start async task
        let task_id = context.create_task("Processing...").await?;
        
        // Perform work in background
        tokio::spawn(async move {
            for i in 0..10 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                context.update_task_progress(task_id, i * 10).await?;
            }
            context.complete_task(task_id, result).await?;
        });
        
        // Return task reference immediately
        Ok(CallToolResult::task(task_id))
    },
)?;`;

export default async function TasksServerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.tasksServer");
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
        <CodeBlock code="cargo run -p mcp-tasks-server" language="bash" title="Terminal" />
      </section>

      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/tasks-server"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/tasks-server
          </a>
        </p>
      </section>
    </div>
  );
}
