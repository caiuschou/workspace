export function getBaseUrl(url?: string): string {
  return url || process.env.OPENCODE_BASE_URL || "http://127.0.0.1:4096"
}

export function getDefaultAgent(): string {
  return process.env.OPENCODE_AGENT || ""
}
