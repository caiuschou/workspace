import { setRequestLocale } from "next-intl/server";

export default async function ExamplesLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);

  return <div className="container mx-auto px-4 py-8">{children}</div>;
}
