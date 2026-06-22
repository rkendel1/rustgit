export function buildAppUrl(
  workspaceId: string | null,
  route: string | undefined,
): string | null {
  if (!workspaceId) return null;
  const path = route?.startsWith("/") ? route : "/";
  return `/api/app-proxy/${workspaceId}${path}`;
}
