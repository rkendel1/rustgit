import { NextRequest, NextResponse } from "next/server";

const BACKEND_BASE =
  process.env.NODE_ENV === "development"
    ? "http://localhost:8080"
    : `https://api.${process.env.NEXT_PUBLIC_BASE_DOMAIN?.replace(/^https?:\/\//, "") ?? "trythissoftware.com"}`;

async function getWorkspacePort(id: string): Promise<number | null> {
  try {
    const res = await fetch(`${BACKEND_BASE}/workspaces/${id}`, {
      cache: "no-store",
    });
    if (!res.ok) return null;
    const ws = await res.json();
    return (ws.ports?.[0]?.port as number) ?? null;
  } catch {
    return null;
  }
}

async function handle(
  request: NextRequest,
  params: Promise<{ id: string; path?: string[] }>,
): Promise<NextResponse> {
  const { id, path } = await params;
  const port = await getWorkspacePort(id);

  if (!port) {
    return NextResponse.json(
      { error: "Workspace not found or not yet running" },
      { status: 404 },
    );
  }

  const subPath = path ? path.join("/") : "";
  const search = request.nextUrl.search;
  const upstreamUrl = `http://localhost:${port}/${subPath}${search}`;

  const forwardHeaders = new Headers();
  request.headers.forEach((value, key) => {
    if (!["host", "connection", "transfer-encoding"].includes(key.toLowerCase())) {
      forwardHeaders.set(key, value);
    }
  });
  forwardHeaders.set("host", `localhost:${port}`);

  let upstreamRes: Response;
  try {
    upstreamRes = await fetch(upstreamUrl, {
      method: request.method,
      headers: forwardHeaders,
      body:
        request.method !== "GET" && request.method !== "HEAD"
          ? new Uint8Array(await request.arrayBuffer())
          : undefined,
    });
  } catch {
    return NextResponse.json(
      { error: `Could not reach app on port ${port}` },
      { status: 502 },
    );
  }

  const contentType = upstreamRes.headers.get("content-type") ?? "";
  const proxyBase = `/api/app-proxy/${id}/`;

  if (contentType.includes("text/html")) {
    let html = await upstreamRes.text();
    // Make relative URLs resolve through our proxy
    const baseTag = `<base href="${proxyBase}">`;
    if (html.includes("<head>")) {
      html = html.replace("<head>", `<head>${baseTag}`);
    } else {
      html = baseTag + html;
    }
    return new NextResponse(html, {
      status: upstreamRes.status,
      headers: { "content-type": "text/html; charset=utf-8" },
    });
  }

  const responseHeaders = new Headers();
  responseHeaders.set("content-type", contentType || "application/octet-stream");
  const cacheControl = upstreamRes.headers.get("cache-control");
  if (cacheControl) responseHeaders.set("cache-control", cacheControl);

  return new NextResponse(upstreamRes.body, {
    status: upstreamRes.status,
    headers: responseHeaders,
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
