const LEGACY_PATH_FALLBACKS: Record<string, string> = {
  "api/badges/generate": "api/badge/generate",
};

export function resolveLegacyFallbackPath(path: string): string | null {
  const normalized = path.replace(/^\/+/, "").toLowerCase();
  return LEGACY_PATH_FALLBACKS[normalized] ?? null;
}
