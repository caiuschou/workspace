import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const dnsProtectionExample = `use mcp_server::http::{
    host_header_validation, localhost_host_validation, DnsProtectionConfig,
};

// Allow localhost only
let router = Router::new()
    .route("/mcp", post(handle_mcp))
    .layer(localhost_host_validation());

// Custom allowed hostnames
let config = DnsProtectionConfig::new(["localhost", "127.0.0.1", "example.com"]);
let router = Router::new()
    .route("/mcp", post(handle_mcp))
    .layer(host_header_validation(config));`;

const axumConfigDns = `let config = AxumHandlerConfig {
    enable_dns_rebinding_protection: true,
    dns_protection_config: None, // Use default localhost config
    ..Default::default()
};`;

const oauthServerRoutes = `use mcp_server::auth::{create_oauth_router, OAuthRouterOptions};

let options = OAuthRouterOptions::new("https://auth.example.com")
    .with_scopes(vec!["read".to_string(), "write".to_string()]);

let oauth_router = create_oauth_router(provider, options);

let app = Router::new()
    .merge(oauth_router)
    .merge(mcp_router);`;

const bearerAuthMiddleware = `use mcp_server::auth::middleware::{BearerAuthLayer, BearerAuthOptions};

let options = BearerAuthOptions::new()
    .with_scopes(vec!["read".to_string()]);

let router = Router::new()
    .route("/mcp", post(handle_mcp))
    .layer(BearerAuthLayer::with_options(verifier, options));`;

const clientOauth = `use mcp_client::auth::{auth, AuthOptions, InMemoryOAuthClientProvider};
use mcp_core::auth::OAuthClientMetadata;

let metadata = OAuthClientMetadata {
    redirect_uris: vec!["http://localhost:8080/callback".to_string()],
    client_name: Some("My App".to_string()),
    ..Default::default()
};

let provider = InMemoryOAuthClientProvider::new(
    Some("http://localhost:8080/callback".to_string()),
    metadata,
);

let result = auth(&provider, AuthOptions::new("https://api.example.com")).await?;

// Integrate with HttpClientConfig
let config = HttpClientConfig::new("https://api.example.com")
    .auth_provider(Arc::new(provider));`;

const rfcSupport = [
  { rfc: "RFC 8414", description: "Authorization Server Metadata Discovery" },
  { rfc: "RFC 9728", description: "Protected Resource Metadata" },
  { rfc: "RFC 7591", description: "Dynamic Client Registration" },
  { rfc: "RFC 7009", description: "Token Revocation" },
  { rfc: "RFC 7636", description: "PKCE (S256)" },
];

const oauthEndpoints = [
  {
    endpoint: "/.well-known/oauth-authorization-server",
    description: "Authorization server metadata",
  },
  { endpoint: "/authorize", description: "Authorization endpoint" },
  { endpoint: "/token", description: "Token endpoint" },
  { endpoint: "/register", description: "Dynamic client registration" },
  { endpoint: "/revoke", description: "Token revocation" },
];

export default async function AuthPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("auth");
  const common = await getTranslations("common");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* DNS Rebinding Protection */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">
          {t("dnsRebindingProtection")}
        </h2>
        <p className="text-gray-400 mb-4">{t("dnsRebindingDesc")}</p>

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {common("serverUsage")}
        </h3>
        <CodeBlock code={dnsProtectionExample} language="rust" />

        <h3 className="text-xl font-semibold mt-6 mb-4">{t("viaAxumConfig")}</h3>
        <CodeBlock code={axumConfigDns} language="rust" />
      </section>

      {/* OAuth 2.1 */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("oauth21")}</h2>
        <p className="text-gray-400 mb-4">{t("oauth21Desc")}</p>

        <div className="overflow-x-auto mb-6">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  RFC
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {rfcSupport.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3">
                    <code className="text-blue-400">{item.rfc}</code>
                  </td>
                  <td className="px-4 py-3 text-gray-400">{item.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Server OAuth Routes */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("serverOAuthRoutes")}</h2>
        <CodeBlock code={oauthServerRoutes} language="rust" />

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {t("availableEndpoints")}
        </h3>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("endpoint")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {oauthEndpoints.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3">
                    <code className="text-green-400">{item.endpoint}</code>
                  </td>
                  <td className="px-4 py-3 text-gray-400">{item.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Bearer Auth Middleware */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">
          {t("bearerAuthMiddleware")}
        </h2>
        <p className="text-gray-400 mb-4">{t("bearerAuthDesc")}</p>
        <CodeBlock code={bearerAuthMiddleware} language="rust" />
      </section>

      {/* Client OAuth */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("clientOAuth")}</h2>
        <p className="text-gray-400 mb-4">{t("clientOAuthDesc")}</p>
        <CodeBlock code={clientOauth} language="rust" />
      </section>

      {/* Features */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("features")}</h2>
        <ul className="list-disc list-inside text-gray-400 space-y-2">
          <li>{t("featuresList.pkce")}</li>
          <li>{t("featuresList.tokenRefresh")}</li>
          <li>{t("featuresList.dynamicRegistration")}</li>
          <li>{t("featuresList.tokenRevocation")}</li>
          <li>{t("featuresList.scopeBased")}</li>
          <li>{t("featuresList.metadataDiscovery")}</li>
        </ul>
      </section>
    </div>
  );
}
