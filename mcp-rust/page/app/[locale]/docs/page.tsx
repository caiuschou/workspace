import Link from "next/link";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { DocCard, CodeBlock } from "../../components";

const installCode = `[dependencies]
# Core types and protocol
mcp_core = { path = "core" }

# Server with HTTP/SSE support
mcp_server = { path = "server", features = ["axum"] }

# Client library
mcp_client = { path = "client" }`;

const runExamplesCode = `# Build all packages
cargo build

# Run HTTP server example
cargo run -p mcp-http-server

# Run HTTP client example
cargo run -p mcp-http-client

# Run all tests
cargo test --workspace`;

export default async function DocsPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("docs");

  const docLinks = [
    {
      title: t("links.quickStart.title"),
      description: t("links.quickStart.description"),
      href: `/${locale}/docs/quickstart`,
      icon: <span>🚀</span>,
    },
    {
      title: t("links.architecture.title"),
      description: t("links.architecture.description"),
      href: `/${locale}/docs/architecture`,
      icon: <span>🏗️</span>,
    },
    {
      title: t("links.httpSse.title"),
      description: t("links.httpSse.description"),
      href: `/${locale}/docs/http-sse`,
      icon: <span>📡</span>,
    },
    {
      title: t("links.websocket.title"),
      description: t("links.websocket.description"),
      href: `/${locale}/docs/websocket`,
      icon: <span>🔌</span>,
    },
    {
      title: t("links.legacySse.title"),
      description: t("links.legacySse.description"),
      href: `/${locale}/docs/legacy-sse`,
      icon: <span>📜</span>,
    },
    {
      title: t("links.auth.title"),
      description: t("links.auth.description"),
      href: `/${locale}/docs/auth`,
      icon: <span>🔐</span>,
    },
  ];

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Quick Links */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-6">{t("topics")}</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {docLinks.map((link) => (
            <DocCard
              key={link.href}
              title={link.title}
              description={link.description}
              href={link.href}
              icon={link.icon}
            />
          ))}
        </div>
      </section>

      {/* Installation */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("installation")}</h2>
        <p className="text-gray-400 mb-4">
          {t("installationDesc")}{" "}
          <code className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-300">
            Cargo.toml
          </code>
          :
        </p>
        <CodeBlock code={installCode} language="toml" title="Cargo.toml" />
      </section>

      {/* Run Examples */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("runExamples")}</h2>
        <p className="text-gray-400 mb-4">{t("runExamplesDesc")}</p>
        <CodeBlock code={runExamplesCode} language="bash" title="Terminal" />
        <p className="text-gray-400 mt-4">
          {t("seeExamplesSection")}{" "}
          <Link
            href={`/${locale}/examples`}
            className="text-blue-400 hover:text-blue-300"
          >
            {t("examplesLink")}
          </Link>{" "}
          {t("forCompleteExamples")}
        </p>
      </section>

      {/* External Links */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">{t("externalResources")}</h2>
        <ul className="space-y-3">
          <li>
            <a
              href="https://spec.modelcontextprotocol.io/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-400 hover:text-blue-300"
            >
              {t("mcpSpecification")}
            </a>
            <span className="text-gray-500 ml-2">{t("mcpSpecificationDesc")}</span>
          </li>
          <li>
            <a
              href="https://github.com/modelcontextprotocol/typescript-sdk"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-400 hover:text-blue-300"
            >
              {t("typescriptSdk")}
            </a>
            <span className="text-gray-500 ml-2">{t("typescriptSdkDesc")}</span>
          </li>
          <li>
            <a
              href="https://github.com/anthropics/mcp-rust"
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-400 hover:text-blue-300"
            >
              {t("githubRepository")}
            </a>
            <span className="text-gray-500 ml-2">{t("githubRepositoryDesc")}</span>
          </li>
        </ul>
      </section>
    </div>
  );
}
