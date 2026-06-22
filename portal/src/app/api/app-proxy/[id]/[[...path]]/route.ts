import { NextRequest, NextResponse } from "next/server";

const BACKEND_BASE =
  process.env.NODE_ENV === "development"
    ? "http://localhost:8080"
    : `https://api.${process.env.NEXT_PUBLIC_BASE_DOMAIN?.replace(/^https?:\/\//, "") ?? "trythissoftware.com"}`;

const MAX_PROBE_TIMEOUT_MS = 500;

type WorkspaceInfo = {
  state?: string;
  framework?: string;
};

type WorkspaceRuntime = {
  pid?: number;
  alive?: boolean;
  exit_code?: number | null;
  requested_port?: number;
  actual_port?: number | null;
  listening?: boolean;
  http_ready?: boolean;
  last_probe?: string;
  stdout?: string[];
  stderr?: string[];
};

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" }[c] ?? c));
}

async function getWorkspace(id: string): Promise<WorkspaceInfo | null> {
  try {
    const res = await fetch(`${BACKEND_BASE}/workspaces/${id}`, { cache: "no-store" });
    if (!res.ok) return null;
    return (await res.json()) as WorkspaceInfo;
  } catch {
    return null;
  }
}

async function getRuntime(id: string): Promise<WorkspaceRuntime | null> {
  try {
    const res = await fetch(`${BACKEND_BASE}/workspaces/${id}/runtime`, { cache: "no-store" });
    if (!res.ok) return null;
    return (await res.json()) as WorkspaceRuntime;
  } catch {
    return null;
  }
}

function startupHtml(id: string, workspace: WorkspaceInfo, runtime: WorkspaceRuntime): string {
  const logs = [...(runtime.stdout ?? []), ...(runtime.stderr ?? [])].slice(-20);
  const safeLogs = logs
    .map((line) => escapeHtml(line))
    .join("\n");
  const details = [
    `Workspace: ${escapeHtml(id)}`,
    `Framework: ${escapeHtml(workspace.framework ?? "unknown")}`,
    `Status: ${escapeHtml(workspace.state ?? "Initializing")}`,
    `PID: ${escapeHtml(String(runtime.pid ?? "unknown"))}`,
    `Probe: ${escapeHtml(runtime.last_probe ?? "connection refused")}`,
  ]
    .map((line) => `<div>${line}</div>`)
    .join("");
  return `<!doctype html><html><head><meta charset="utf-8"/><meta http-equiv="refresh" content="1"><title>Starting Workspace</title></head><body style="font-family: ui-monospace, SFMono-Regular, Menlo, monospace; padding: 24px;"><h2>🚀 Starting Workspace</h2>${details}<hr/><pre style="white-space: pre-wrap;">${safeLogs}</pre></body></html>`;
}

async function handle(
  request: NextRequest,
  params: Promise<{ id: string; path?: string[] }>,
): Promise<NextResponse> {
  const { id, path } = await params;
  const workspace = await getWorkspace(id);
  if (!workspace) {
    return NextResponse.json({ error: "Workspace not found" }, { status: 404 });
  }
  const runtime = await getRuntime(id);
  const port = runtime?.actual_port ?? null;
  const isReady = Boolean(runtime?.alive && runtime?.http_ready && port);

  if (!isReady || !port) {
    const payload = {
      workspaceId: id,
      framework: workspace.framework ?? "unknown",
      state: workspace.state ?? "Initializing",
      pid: runtime?.pid ?? null,
      requestedPort: runtime?.requested_port ?? null,
      actualPort: runtime?.actual_port ?? null,
      processAlive: runtime?.alive ?? false,
      httpReady: runtime?.http_ready ?? false,
      lastProbe: runtime?.last_probe ?? "connection refused",
      logs: [...(runtime?.stdout ?? []), ...(runtime?.stderr ?? [])].slice(-20),
    };

    if (request.method === "GET") {
      return new NextResponse(startupHtml(id, workspace, runtime ?? {}), {
        status: 202,
        headers: { "content-type": "text/html; charset=utf-8", "cache-control": "no-store" },
      });
    }
    return NextResponse.json(payload, { status: 202 });
  }

  const subPath = path ? path.join("/") : "";
  const upstreamUrl = `http://127.0.0.1:${port}/${subPath}${request.nextUrl.search}`;
  const forwardHeaders = new Headers();
  request.headers.forEach((value, key) => {
    if (!["host", "connection", "transfer-encoding"].includes(key.toLowerCase())) {
      forwardHeaders.set(key, value);
    }
  });
  forwardHeaders.set("host", `127.0.0.1:${port}`);
  const body =
    request.method !== "GET" && request.method !== "HEAD"
      ? new Uint8Array(await request.arrayBuffer())
      : undefined;
  const upstreamRes = await fetch(upstreamUrl, {
    method: request.method,
    headers: forwardHeaders,
    body,
    signal: AbortSignal.timeout(MAX_PROBE_TIMEOUT_MS),
  });
  return new NextResponse(upstreamRes.body, {
    status: upstreamRes.status,
    headers: upstreamRes.headers,
  });
}

export async function GET(
  req: NextRequest,
  ctx: { params: Promise<{ id: string; path?: string[] }> },
) {
  return handle(req, ctx.params);
}

export async function POST(
  req: NextRequest,
  ctx: { params: Promise<{ id: string; path?: string[] }> },
) {
  return handle(req, ctx.params);
}

export async function PUT(
  req: NextRequest,
  ctx: { params: Promise<{ id: string; path?: string[] }> },
) {
  return handle(req, ctx.params);
}

export async function DELETE(
  req: NextRequest,
  ctx: { params: Promise<{ id: string; path?: string[] }> },
) {
  return handle(req, ctx.params);
}
