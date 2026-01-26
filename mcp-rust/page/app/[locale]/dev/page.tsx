import { getTranslations, setRequestLocale } from "next-intl/server";
import { StatusBadge } from "../../components";

type Status = "completed" | "in_progress" | "partial" | "not_started";

export default async function DevPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("dev");
  const common = await getTranslations("common");

  const moduleStatus = [
    {
      module: t("modules.stdioTransport.name"),
      status: "completed" as Status,
      description: t("modules.stdioTransport.description"),
    },
    {
      module: t("modules.httpSseTransport.name"),
      status: "completed" as Status,
      description: t("modules.httpSseTransport.description"),
    },
    {
      module: t("modules.websocketTransport.name"),
      status: "completed" as Status,
      description: t("modules.websocketTransport.description"),
    },
    {
      module: t("modules.legacySseTransport.name"),
      status: "completed" as Status,
      description: t("modules.legacySseTransport.description"),
    },
    {
      module: t("modules.toolsResourcesPrompts.name"),
      status: "completed" as Status,
      description: t("modules.toolsResourcesPrompts.description"),
    },
    {
      module: t("modules.tasksApi.name"),
      status: "completed" as Status,
      description: t("modules.tasksApi.description"),
    },
    {
      module: t("modules.oauthDnsProtection.name"),
      status: "completed" as Status,
      description: t("modules.oauthDnsProtection.description"),
    },
    {
      module: t("modules.samplingElicitation.name"),
      status: "completed" as Status,
      description: t("modules.samplingElicitation.description"),
    },
  ];

  const featureComparison = [
    {
      feature: "Stdio Transport (Client/Server)",
      typescript: "Default stdio transport with tests",
      rust: "completed" as Status,
    },
    {
      feature: "Streamable HTTP Transport",
      typescript: "HTTP+SSE/JSON response, session/reconnect/resumption",
      rust: "completed" as Status,
    },
    {
      feature: "HTTP+SSE Compat Transport",
      typescript: "Legacy SSE with fallback client",
      rust: "completed" as Status,
    },
    {
      feature: "WebSocket Transport",
      typescript: "websocket.ts client transport",
      rust: "completed" as Status,
    },
    {
      feature: "DNS Rebinding Protection",
      typescript: "createMcpExpressApp auto host validation",
      rust: "completed" as Status,
    },
    {
      feature: "OAuth Server Routes",
      typescript: "authorize/token/register/revoke/metadata",
      rust: "completed" as Status,
    },
    {
      feature: "OAuth Client Auth",
      typescript: "Streamable HTTP authProvider with token refresh",
      rust: "completed" as Status,
    },
    {
      feature: "Tools Registration/Call",
      typescript: "Register tools, list/call, output schema validation",
      rust: "completed" as Status,
    },
    {
      feature: "Resources Registration",
      typescript: "Register resources/templates, list/read",
      rust: "completed" as Status,
    },
    {
      feature: "Prompts Registration",
      typescript: "Register prompts, list/get",
      rust: "completed" as Status,
    },
    {
      feature: "ResourceLink",
      typescript: "Tool returns resource_link content type",
      rust: "completed" as Status,
    },
    {
      feature: "Prompt/Resource Completions",
      typescript: "Parameter completion capability",
      rust: "not_started" as Status,
    },
    {
      feature: "Logging setLevel",
      typescript: "logging/setLevel request handling",
      rust: "completed" as Status,
    },
    {
      feature: "list_changed Notifications",
      typescript: "tools/prompts/resources list changed + debounce",
      rust: "completed" as Status,
    },
    {
      feature: "Roots Capability",
      typescript: "roots/list with list_changed support",
      rust: "partial" as Status,
    },
    {
      feature: "Sampling createMessage",
      typescript: "Server requests client sampling",
      rust: "completed" as Status,
    },
    {
      feature: "Form/URL Elicitation",
      typescript: "elicitation/create form/URL mode",
      rust: "completed" as Status,
    },
    {
      feature: "Tasks API",
      typescript: "tasks/get/list/result/cancel with storage",
      rust: "completed" as Status,
    },
    {
      feature: "Completion Tool",
      typescript: "completion/complete (prompt/resource)",
      rust: "not_started" as Status,
    },
  ];

  const changelog = [
    {
      date: t("changelog.oauthDns.date"),
      title: t("changelog.oauthDns.title"),
      items: [
        "DNS rebinding protection middleware",
        "OAuth 2.1 core types",
        "Server OAuth routes (authorize/token/register/revoke/metadata)",
        "Bearer Token auth middleware",
        "Client auth middleware",
        "OAuth metadata discovery (RFC 8414, RFC 9728)",
        "PKCE support (S256)",
        "Token refresh",
        "Dynamic client registration (RFC 7591)",
      ],
    },
    {
      date: t("changelog.httpSse.date"),
      title: t("changelog.httpSse.title"),
      items: [
        "True SSE long-lived streaming with axum",
        "Bidirectional message push (SseBroadcaster)",
        "Last-Event-ID reconnection replay (EventBuffer)",
        "CORS support",
        "axum framework integration",
      ],
    },
    {
      date: t("changelog.foundation.date"),
      title: t("changelog.foundation.title"),
      items: [
        "Stdio transport (client/server)",
        "MCP core protocol (JSON-RPC)",
        "Tools/Resources/Prompts registration",
        "Tasks API",
        "list_changed notifications",
        "Logging support",
      ],
    },
  ];

  const futurePlans = [
    "Experimental task streaming helpers",
    "Server-side Roots support",
    "Completions support",
  ];

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Module Status */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-6">{t("moduleStatus")}</h2>
        <div className="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-800">
                <th className="px-6 py-4 text-left font-semibold text-gray-300">
                  {common("module")}
                </th>
                <th className="px-6 py-4 text-left font-semibold text-gray-300">
                  {common("status")}
                </th>
                <th className="px-6 py-4 text-left font-semibold text-gray-300 hidden md:table-cell">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {moduleStatus.map((item, index) => (
                <tr
                  key={index}
                  className="border-b border-gray-800 last:border-0"
                >
                  <td className="px-6 py-4 text-white font-medium">
                    {item.module}
                  </td>
                  <td className="px-6 py-4">
                    <StatusBadge status={item.status} />
                  </td>
                  <td className="px-6 py-4 text-gray-400 hidden md:table-cell">
                    {item.description}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* TypeScript SDK Comparison */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-6">
          {t("typescriptComparison")}
        </h2>
        <div className="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800">
                <th className="px-4 py-3 text-left font-semibold text-gray-300">
                  {common("feature")}
                </th>
                <th className="px-4 py-3 text-left font-semibold text-gray-300 hidden lg:table-cell">
                  TypeScript
                </th>
                <th className="px-4 py-3 text-center font-semibold text-gray-300">
                  Rust
                </th>
              </tr>
            </thead>
            <tbody>
              {featureComparison.map((item, index) => (
                <tr
                  key={index}
                  className="border-b border-gray-800 last:border-0"
                >
                  <td className="px-4 py-3 text-gray-300">{item.feature}</td>
                  <td className="px-4 py-3 text-gray-500 text-xs hidden lg:table-cell">
                    {item.typescript}
                  </td>
                  <td className="px-4 py-3 text-center">
                    <StatusBadge status={item.rust} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Changelog */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-6">{t("recentChanges")}</h2>
        <div className="space-y-6">
          {changelog.map((entry, index) => (
            <div
              key={index}
              className="bg-gray-900 rounded-xl border border-gray-800 p-6"
            >
              <div className="flex items-center gap-3 mb-4">
                <span className="text-sm text-gray-500">{entry.date}</span>
                <h3 className="text-lg font-semibold text-white">
                  {entry.title}
                </h3>
              </div>
              <ul className="list-disc list-inside text-gray-400 space-y-1">
                {entry.items.map((item, i) => (
                  <li key={i}>{item}</li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </section>

      {/* Future Plans */}
      <section>
        <h2 className="text-2xl font-semibold mb-6">{t("futurePlans")}</h2>
        <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
          <ul className="space-y-3">
            {futurePlans.map((plan, index) => (
              <li key={index} className="flex items-center gap-3">
                <span className="text-gray-500">○</span>
                <span className="text-gray-400">{plan}</span>
              </li>
            ))}
          </ul>
        </div>
      </section>
    </div>
  );
}
