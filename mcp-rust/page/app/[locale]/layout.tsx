import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { notFound } from "next/navigation";
import { NextIntlClientProvider } from "next-intl";
import { getMessages, setRequestLocale } from "next-intl/server";
import { routing, type Locale } from "../../i18n/routing";
import Link from "next/link";
import "../globals.css";
import { LanguageSwitcher } from "../components/LanguageSwitcher";

const inter = Inter({ subsets: ["latin"] });

export function generateStaticParams() {
  return routing.locales.map((locale) => ({ locale }));
}

export const metadata: Metadata = {
  title: "MCP Rust",
  description: "Rust implementation of Model Context Protocol",
};

function NavLink({
  href,
  children,
}: {
  href: string;
  children: React.ReactNode;
}) {
  return (
    <Link
      href={href}
      className="text-gray-300 hover:text-white transition-colors"
    >
      {children}
    </Link>
  );
}

function Header({
  locale,
  messages,
}: {
  locale: string;
  messages: { nav: Record<string, string> };
}) {
  const t = messages.nav;

  return (
    <header className="bg-gray-900 border-b border-gray-800 sticky top-0 z-50">
      <nav className="container mx-auto px-4 h-16 flex items-center justify-between">
        <Link href={`/${locale}`} className="font-bold text-xl text-white">
          MCP Rust
        </Link>

        {/* Desktop Navigation */}
        <div className="hidden md:flex items-center space-x-8">
          <NavLink href={`/${locale}`}>{t.home}</NavLink>
          <NavLink href={`/${locale}/docs`}>{t.docs}</NavLink>
          <NavLink href={`/${locale}/examples`}>{t.examples}</NavLink>
          <NavLink href={`/${locale}/dev`}>{t.development}</NavLink>
          <a
            href="https://github.com/anthropics/mcp-rust"
            target="_blank"
            rel="noopener noreferrer"
            className="text-gray-300 hover:text-white transition-colors"
          >
            GitHub
          </a>
          <LanguageSwitcher />
        </div>

        {/* Mobile Navigation */}
        <div className="md:hidden flex items-center gap-2">
          <LanguageSwitcher />
          <details className="relative">
            <summary className="list-none cursor-pointer p-2">
              <svg
                className="w-6 h-6 text-gray-300"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 6h16M4 12h16M4 18h16"
                />
              </svg>
            </summary>
            <div className="absolute right-0 mt-2 w-48 bg-gray-800 rounded-lg shadow-lg border border-gray-700 py-2">
              <Link
                href={`/${locale}`}
                className="block px-4 py-2 text-gray-300 hover:bg-gray-700 hover:text-white"
              >
                {t.home}
              </Link>
              <Link
                href={`/${locale}/docs`}
                className="block px-4 py-2 text-gray-300 hover:bg-gray-700 hover:text-white"
              >
                {t.docs}
              </Link>
              <Link
                href={`/${locale}/examples`}
                className="block px-4 py-2 text-gray-300 hover:bg-gray-700 hover:text-white"
              >
                {t.examples}
              </Link>
              <Link
                href={`/${locale}/dev`}
                className="block px-4 py-2 text-gray-300 hover:bg-gray-700 hover:text-white"
              >
                {t.development}
              </Link>
              <a
                href="https://github.com/anthropics/mcp-rust"
                target="_blank"
                rel="noopener noreferrer"
                className="block px-4 py-2 text-gray-300 hover:bg-gray-700 hover:text-white"
              >
                GitHub
              </a>
            </div>
          </details>
        </div>
      </nav>
    </header>
  );
}

function Footer({
  locale,
  messages,
}: {
  locale: string;
  messages: { footer: Record<string, string> };
}) {
  const t = messages.footer;

  return (
    <footer className="bg-gray-900 border-t border-gray-800 mt-auto">
      <div className="container mx-auto px-4 py-8">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {/* Brand */}
          <div>
            <h3 className="font-bold text-lg text-white mb-2">{t.brand}</h3>
            <p className="text-gray-400 text-sm">{t.brandDescription}</p>
          </div>

          {/* Links */}
          <div>
            <h4 className="font-semibold text-white mb-3">{t.documentation}</h4>
            <ul className="space-y-2 text-sm">
              <li>
                <Link
                  href={`/${locale}/docs/quickstart`}
                  className="text-gray-400 hover:text-white"
                >
                  {t.quickStart}
                </Link>
              </li>
              <li>
                <Link
                  href={`/${locale}/docs/architecture`}
                  className="text-gray-400 hover:text-white"
                >
                  {t.architecture}
                </Link>
              </li>
              <li>
                <Link
                  href={`/${locale}/examples`}
                  className="text-gray-400 hover:text-white"
                >
                  {t.examples}
                </Link>
              </li>
            </ul>
          </div>

          {/* External Links */}
          <div>
            <h4 className="font-semibold text-white mb-3">{t.resources}</h4>
            <ul className="space-y-2 text-sm">
              <li>
                <a
                  href="https://github.com/anthropics/mcp-rust"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-gray-400 hover:text-white"
                >
                  {t.githubRepository}
                </a>
              </li>
              <li>
                <a
                  href="https://spec.modelcontextprotocol.io/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-gray-400 hover:text-white"
                >
                  {t.mcpSpecification}
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/modelcontextprotocol/typescript-sdk"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-gray-400 hover:text-white"
                >
                  {t.typescriptSdk}
                </a>
              </li>
            </ul>
          </div>
        </div>

        <div className="mt-8 pt-8 border-t border-gray-800 text-center text-gray-500 text-sm">
          {t.copyright}
        </div>
      </div>
    </footer>
  );
}

export default async function LocaleLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;

  // Validate locale
  if (!routing.locales.includes(locale as Locale)) {
    notFound();
  }

  // Enable static rendering
  setRequestLocale(locale);

  // Get messages for this locale
  const messages = await getMessages();

  return (
    <html lang={locale}>
      <body
        className={`${inter.className} bg-gray-950 text-white min-h-screen flex flex-col`}
      >
        <NextIntlClientProvider messages={messages}>
          <Header
            locale={locale}
            messages={messages as { nav: Record<string, string> }}
          />
          <main className="flex-1">{children}</main>
          <Footer
            locale={locale}
            messages={messages as { footer: Record<string, string> }}
          />
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
