import Link from "next/link";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { FeatureCard } from "../components";
import { StatusBadge } from "../components";

export default async function Home({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("home");
  const common = await getTranslations("common");

  const statusData = [
    { module: t("statusData.stdioTransport"), status: "completed" as const },
    { module: t("statusData.httpSseTransport"), status: "completed" as const },
    {
      module: t("statusData.websocketTransport"),
      status: "completed" as const,
    },
    { module: t("statusData.legacySseTransport"), status: "completed" as const },
    {
      module: t("statusData.toolsResourcesPrompts"),
      status: "completed" as const,
    },
    { module: t("statusData.tasksApi"), status: "completed" as const },
    {
      module: t("statusData.oauthDnsProtection"),
      status: "completed" as const,
    },
    {
      module: t("statusData.samplingElicitation"),
      status: "completed" as const,
    },
  ];

  return (
    <div>
      {/* Hero Section */}
      <section className="py-20 px-4">
        <div className="container mx-auto text-center">
          <h1 className="text-5xl md:text-6xl font-bold mb-6 bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
            {t("title")}
          </h1>
          <p className="text-xl text-gray-400 max-w-2xl mx-auto mb-8">
            {t("subtitle")}
          </p>
          <div className="flex flex-wrap justify-center gap-4">
            <Link
              href={`/${locale}/docs`}
              className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors"
            >
              {common("getStarted")}
            </Link>
            <Link
              href={`/${locale}/docs/quickstart`}
              className="px-6 py-3 bg-gray-800 hover:bg-gray-700 text-white font-semibold rounded-lg transition-colors"
            >
              {common("quickStart")}
            </Link>
            <a
              href="https://github.com/anthropics/mcp-rust"
              target="_blank"
              rel="noopener noreferrer"
              className="px-6 py-3 border border-gray-700 hover:border-gray-600 text-gray-300 hover:text-white font-semibold rounded-lg transition-colors"
            >
              GitHub
            </a>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-16 px-4 bg-gray-900/50">
        <div className="container mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">
            {t("features")}
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <FeatureCard
              icon={<span>🔌</span>}
              title={t("multipleTransports.title")}
              description={t("multipleTransports.description")}
            />
            <FeatureCard
              icon={<span>🛠️</span>}
              title={t("completeMcpSupport.title")}
              description={t("completeMcpSupport.description")}
            />
            <FeatureCard
              icon={<span>🔐</span>}
              title={t("securityBuiltIn.title")}
              description={t("securityBuiltIn.description")}
            />
            <FeatureCard
              icon={<span>📡</span>}
              title={t("realTimeStreaming.title")}
              description={t("realTimeStreaming.description")}
            />
            <FeatureCard
              icon={<span>⚡</span>}
              title={t("asyncAwaitNative.title")}
              description={t("asyncAwaitNative.description")}
            />
            <FeatureCard
              icon={<span>📦</span>}
              title={t("modularDesign.title")}
              description={t("modularDesign.description")}
            />
          </div>
        </div>
      </section>

      {/* Status Section */}
      <section className="py-16 px-4">
        <div className="container mx-auto max-w-3xl">
          <h2 className="text-3xl font-bold text-center mb-4">
            {t("currentStatus")}
          </h2>
          <p className="text-gray-400 text-center mb-8">{t("trackingParity")}</p>
          <div className="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-800">
                  <th className="px-6 py-4 text-left font-semibold text-gray-300">
                    {common("module")}
                  </th>
                  <th className="px-6 py-4 text-right font-semibold text-gray-300">
                    {common("status")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {statusData.map((item, index) => (
                  <tr
                    key={index}
                    className="border-b border-gray-800 last:border-0"
                  >
                    <td className="px-6 py-4 text-gray-400">{item.module}</td>
                    <td className="px-6 py-4 text-right">
                      <StatusBadge status={item.status} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="text-center mt-6">
            <Link
              href={`/${locale}/dev`}
              className="text-blue-400 hover:text-blue-300 font-medium"
            >
              {t("viewDetailedProgress")}
            </Link>
          </div>
        </div>
      </section>

      {/* Quick Install Section */}
      <section className="py-16 px-4 bg-gray-900/50">
        <div className="container mx-auto max-w-3xl">
          <h2 className="text-3xl font-bold text-center mb-8">
            {t("quickInstall")}
          </h2>
          <div className="bg-gray-950 rounded-xl border border-gray-800 overflow-hidden">
            <div className="px-4 py-2 bg-gray-900 border-b border-gray-800 text-sm text-gray-400">
              Cargo.toml
            </div>
            <pre className="p-4 overflow-x-auto">
              <code className="text-sm text-gray-300">{`[dependencies]
# Core types and protocol
mcp_core = { path = "core" }

# Server with HTTP/SSE support
mcp_server = { path = "server", features = ["axum"] }

# Client library
mcp_client = { path = "client" }`}</code>
            </pre>
          </div>
          <div className="text-center mt-6">
            <Link
              href={`/${locale}/docs/quickstart`}
              className="text-blue-400 hover:text-blue-300 font-medium"
            >
              {t("seeInstallationGuide")}
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}
