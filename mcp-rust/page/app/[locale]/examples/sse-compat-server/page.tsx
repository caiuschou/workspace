import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, create_router,
    LegacySseConfig, LegacySseState, create_legacy_sse_router,
};

// Create Streamable HTTP router (new protocol)
let streamable_config = AxumHandlerConfig {
    endpoint_path: "/mcp".to_string(),
    ..Default::default()
};
let streamable_state = Arc::new(AxumHandlerState::new(Arc::clone(&mcp_server), streamable_config));
let streamable_router = create_router(streamable_state);

// Create legacy SSE router (old protocol)
let legacy_config = LegacySseConfig {
    endpoint_path: "/sse".to_string(),
    message_path: "/message".to_string(),
};
let legacy_state = Arc::new(LegacySseState::new(Arc::clone(&mcp_server), legacy_config));
let legacy_router = create_legacy_sse_router(legacy_state);

// Merge both routers
let app = streamable_router.merge(legacy_router);`;

export default async function SseCompatServerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.sseCompatServer");
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
        <CodeBlock code="cargo run -p mcp-sse-compat-server" language="bash" title="Terminal" />
      </section>

      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/sse-compat-server"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/sse-compat-server
          </a>
        </p>
      </section>
    </div>
  );
}
