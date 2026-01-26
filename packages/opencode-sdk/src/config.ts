const DEFAULT_BASE_URL = "http://127.0.0.1:4096"

export function getBaseUrl(url?: string): string {
  return url || process.env.OPENCODE_BASE_URL || DEFAULT_BASE_URL
}

export function getDefaultAgent(): string {
  return process.env.OPENCODE_AGENT || ""
}

/**
 * Check if we should use auto-start for the server.
 * Returns true only if no custom URL is provided (via option or env).
 */
export function shouldUseAutoStart(customUrl?: string): boolean {
  // If user provided a URL via option, don't auto-start (might be remote)
  if (customUrl) {
    return false
  }
  // If OPENCODE_BASE_URL is set, don't auto-start (user configured it)
  if (process.env.OPENCODE_BASE_URL) {
    return false
  }
  // Otherwise, use auto-start for the default local server
  return true
}

export { DEFAULT_BASE_URL }
