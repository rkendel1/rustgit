export function rewriteLocationHeader(
  location: string,
  endpoint: string,
  proxyOrigin: string,
  workspaceId: string,
): string {
  const absoluteLocation = new URL(location, endpoint);
  const rewritten = new URL(
    `/api/app-proxy/${workspaceId}${absoluteLocation.pathname}`,
    proxyOrigin,
  );
  rewritten.search = absoluteLocation.search;
  rewritten.hash = absoluteLocation.hash;
  return rewritten.toString();
}
