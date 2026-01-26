import { getTranslations, setRequestLocale } from "next-intl/server";
import { DocCard } from "../../components";

export default async function ExamplesPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examples");

  const serverExamples = [
    {
      title: t("servers.httpServer.title"),
      description: t("servers.httpServer.description"),
      href: `/${locale}/examples/http-server`,
      icon: <span>🌐</span>,
    },
    {
      title: t("servers.websocketServer.title"),
      description: t("servers.websocketServer.description"),
      href: `/${locale}/examples/websocket-server`,
      icon: <span>🔌</span>,
    },
    {
      title: t("servers.filesystemServer.title"),
      description: t("servers.filesystemServer.description"),
      href: `/${locale}/examples/filesystem-server`,
      icon: <span>📁</span>,
    },
    {
      title: t("servers.promptsServer.title"),
      description: t("servers.promptsServer.description"),
      href: `/${locale}/examples/prompts-server`,
      icon: <span>💬</span>,
    },
    {
      title: t("servers.tasksServer.title"),
      description: t("servers.tasksServer.description"),
      href: `/${locale}/examples/tasks-server`,
      icon: <span>📋</span>,
    },
    {
      title: t("servers.loggingServer.title"),
      description: t("servers.loggingServer.description"),
      href: `/${locale}/examples/logging-server`,
      icon: <span>📝</span>,
    },
    {
      title: t("servers.sseCompatServer.title"),
      description: t("servers.sseCompatServer.description"),
      href: `/${locale}/examples/sse-compat-server`,
      icon: <span>📜</span>,
    },
  ];

  const clientExamples = [
    {
      title: t("clients.httpClient.title"),
      description: t("clients.httpClient.description"),
      href: `/${locale}/examples/http-client`,
      icon: <span>📡</span>,
    },
    {
      title: t("clients.websocketClient.title"),
      description: t("clients.websocketClient.description"),
      href: `/${locale}/examples/websocket-client`,
      icon: <span>🔗</span>,
    },
    {
      title: t("clients.filesystemClient.title"),
      description: t("clients.filesystemClient.description"),
      href: `/${locale}/examples/filesystem-client`,
      icon: <span>📂</span>,
    },
  ];

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Server Examples */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-6">{t("serverExamples")}</h2>
        <p className="text-gray-400 mb-4">{t("serverExamplesDesc")}</p>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {serverExamples.map((example) => (
            <DocCard
              key={example.href}
              title={example.title}
              description={example.description}
              href={example.href}
              icon={example.icon}
            />
          ))}
        </div>
      </section>

      {/* Client Examples */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-6">{t("clientExamples")}</h2>
        <p className="text-gray-400 mb-4">{t("clientExamplesDesc")}</p>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {clientExamples.map((example) => (
            <DocCard
              key={example.href}
              title={example.title}
              description={example.description}
              href={example.href}
              icon={example.icon}
            />
          ))}
        </div>
      </section>

      {/* Quick Start */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">{t("runningExamples")}</h2>
        <p className="text-gray-400 mb-4">{t("runningExamplesDesc")}</p>
        <div className="rounded-lg overflow-hidden bg-gray-950 border border-gray-800">
          <div className="px-4 py-2 bg-gray-900 border-b border-gray-800 text-sm text-gray-400">
            Terminal
          </div>
          <pre className="p-4 overflow-x-auto">
            <code className="text-sm text-gray-300">{`# Build all examples
cargo build --examples

# Run a specific server example
cargo run -p mcp-http-server

# Run a specific client example (in another terminal)
cargo run -p mcp-http-client`}</code>
          </pre>
        </div>
      </section>
    </div>
  );
}
