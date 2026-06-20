"use client";

import { useMemo, useState } from "react";
import styles from "./page.module.css";

type Action = {
  id: string;
  label: string;
  method: "GET" | "POST" | "DELETE";
  path: string;
  body?: (ctx: ContextState) => unknown;
};

type ActionGroup = {
  title: string;
  description: string;
  actions: Action[];
};

type ContextState = {
  orgId: string;
  userId: string;
  anonUserId: string;
  owner: string;
  repo: string;
  branch: string;
  executionId: string;
  repoId: string;
};

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL ?? "https://api.trythissoftware.com";

const ACTION_GROUPS: ActionGroup[] = [
  {
    title: "Dual surface + portal contracts",
    description: "Portal shell, navigation, and extension/portal shared contracts.",
    actions: [
      { id: "dual-contract", label: "Dual surface contract", method: "GET", path: "/api/v1/dual-surface/contract" },
      { id: "portal-nav", label: "Portal navigation", method: "GET", path: "/api/v1/surfaces/portal/navigation" },
      { id: "portal-ui", label: "Portal UI contract", method: "GET", path: "/api/v1/surfaces/portal/ui" },
      { id: "extension-actions", label: "Extension actions", method: "GET", path: "/api/v1/surfaces/extension/actions" },
      { id: "extension-ui", label: "Extension UI", method: "GET", path: "/api/v1/surfaces/extension/ui" },
    ],
  },
  {
    title: "Authentication",
    description: "Login/logout/me and OAuth callback payload contracts.",
    actions: [
      {
        id: "auth-login",
        label: "Auth login",
        method: "POST",
        path: "/auth/login",
        body: (ctx) => ({
          org_id: ctx.orgId,
          role: "Owner",
          user: {
            user_id: ctx.userId,
            email: "portal@trythissoftware.com",
            name: "Portal User",
            auth_provider: "github",
          },
        }),
      },
      { id: "auth-me", label: "Auth me", method: "GET", path: "/auth/me" },
      {
        id: "auth-logout",
        label: "Auth logout",
        method: "POST",
        path: "/auth/logout",
        body: (ctx) => ({ user_id: ctx.userId, org_id: ctx.orgId }),
      },
      {
        id: "oauth-github",
        label: "GitHub OAuth callback",
        method: "GET",
        path: "/auth/github/callback?code=portal-demo-code&state=portal-demo-state",
      },
      {
        id: "oauth-google",
        label: "Google OAuth callback",
        method: "GET",
        path: "/auth/google/callback?code=portal-demo-code&state=portal-demo-state",
      },
    ],
  },
  {
    title: "Execution lifecycle",
    description: "Start, inspect, logs, restart, stop, migrate, and claim execution identity.",
    actions: [
      {
        id: "execution-start",
        label: "Start execution",
        method: "POST",
        path: "/api/v1/executions",
        body: (ctx) => ({
          org_id: ctx.orgId,
          user_id: ctx.userId,
          anon_user_id: ctx.anonUserId,
          repo_url: `https://github.com/${ctx.owner}/${ctx.repo}`,
          branch: ctx.branch,
        }),
      },
      { id: "execution-status", label: "Execution status", method: "GET", path: "/api/v1/executions/{id}" },
      { id: "execution-logs", label: "Execution logs", method: "GET", path: "/api/v1/executions/{id}/logs" },
      {
        id: "execution-restart",
        label: "Restart execution",
        method: "POST",
        path: "/api/v1/executions/{id}/restart",
      },
      { id: "execution-stop", label: "Stop execution", method: "POST", path: "/api/v1/executions/{id}/stop" },
      {
        id: "execution-migrate",
        label: "Migrate execution",
        method: "POST",
        path: "/api/v1/executions/{id}/migrate",
        body: () => ({ target: "cloud" }),
      },
      {
        id: "execution-claim",
        label: "Claim execution",
        method: "POST",
        path: "/api/v1/executions/{id}/claim",
        body: (ctx) => ({
          anon_user_id: ctx.anonUserId,
          user_id: ctx.userId,
          org_id: ctx.orgId,
        }),
      },
      {
        id: "execution-history",
        label: "Execution history",
        method: "GET",
        path: "/executions/{id}/history",
      },
    ],
  },
  {
    title: "Badges, seed, and repository intelligence",
    description: "Portal badge generator, seed flow, and repository runtime intelligence endpoints.",
    actions: [
      {
        id: "badge-generate",
        label: "Generate badge",
        method: "POST",
        path: "/api/badge/generate",
        body: (ctx) => ({
          repo_url: `https://github.com/${ctx.owner}/${ctx.repo}`,
          branch: ctx.branch,
        }),
      },
      { id: "seed-launch", label: "Badge seed launch", method: "GET", path: "/seed/{owner}/{repo}" },
      { id: "badge-svg", label: "Badge SVG", method: "GET", path: "/badge/{owner}/{repo}.svg" },
      {
        id: "healed-badge-svg",
        label: "Healed badge SVG",
        method: "GET",
        path: "/badge/healed/{owner}/{repo}.svg",
      },
      {
        id: "image-compile",
        label: "Execution image compile",
        method: "POST",
        path: "/execution-image/compile",
        body: (ctx) => ({ repo_url: `https://github.com/${ctx.owner}/${ctx.repo}`, branch: ctx.branch }),
      },
      { id: "warm-pool", label: "Warm pool status", method: "GET", path: "/warm-pool/status" },
      { id: "repo-history", label: "Repository history", method: "GET", path: "/repositories/{repo_id}/history" },
    ],
  },
  {
    title: "Billing",
    description: "Usage, summary, and invoice service endpoints.",
    actions: [
      { id: "billing-usage", label: "Billing usage", method: "GET", path: "/billing/usage?org_id={org_id}" },
      { id: "billing-summary", label: "Billing summary", method: "GET", path: "/billing/summary" },
      {
        id: "billing-invoice",
        label: "Billing invoice",
        method: "POST",
        path: "/billing/invoice",
        body: (ctx) => ({ org_id: ctx.orgId }),
      },
    ],
  },
];

function toDisplayText(payload: unknown): string {
  if (typeof payload === "string") {
    return payload;
  }
  return JSON.stringify(payload, null, 2);
}

function resolveTemplate(path: string, ctx: ContextState): string {
  return path
    .replaceAll("{id}", encodeURIComponent(ctx.executionId))
    .replaceAll("{owner}", encodeURIComponent(ctx.owner))
    .replaceAll("{repo}", encodeURIComponent(ctx.repo))
    .replaceAll("{org_id}", encodeURIComponent(ctx.orgId))
    .replaceAll("{repo_id}", encodeURIComponent(ctx.repoId));
}

export default function Home() {
  const [ctx, setCtx] = useState<ContextState>({
    orgId: "org-demo",
    userId: "user-demo",
    anonUserId: "anon-demo",
    owner: "vercel",
    repo: "next.js",
    branch: "main",
    executionId: "exec-demo",
    repoId: "repo-demo",
  });
  const [activeAction, setActiveAction] = useState<string>("None");
  const [status, setStatus] = useState<string>("Idle");
  const [responseText, setResponseText] = useState<string>("No response yet.");

  const summary = useMemo(
    () => ({
      groups: ACTION_GROUPS.length,
      actions: ACTION_GROUPS.reduce((count, group) => count + group.actions.length, 0),
      apiBase: API_BASE_URL,
    }),
    [],
  );

  async function runAction(action: Action) {
    setActiveAction(action.label);
    setStatus("Loading...");

    const resolvedPath = resolveTemplate(action.path, ctx);
    const url = new URL(`/api/proxy${resolvedPath}`, window.location.origin);

    const requestInit: RequestInit = {
      method: action.method,
      headers: {
        "Content-Type": "application/json",
      },
    };

    if (action.method !== "GET" && action.body) {
      requestInit.body = JSON.stringify(action.body(ctx));
    }

    try {
      const response = await fetch(url, requestInit);
      const contentType = response.headers.get("content-type") ?? "";
      const payload = contentType.includes("application/json")
        ? await response.json()
        : await response.text();

      if (action.id === "execution-start" && typeof payload === "object" && payload) {
        const executionId = (payload as { execution_id?: string }).execution_id;
        if (executionId) {
          setCtx((previous) => ({ ...previous, executionId }));
        }
      }

      setStatus(`${response.status} ${response.statusText}`.trim());
      setResponseText(toDisplayText(payload));
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown request error";
      setStatus("Request failed");
      setResponseText(message);
    }
  }

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <h1>TryThisSoftware Portal</h1>
        <p>Next.js management portal integrated with all backend service contracts.</p>
      </header>

      <section className={styles.summaryGrid}>
        <article className={styles.card}>
          <h2>Service Groups</h2>
          <strong>{summary.groups}</strong>
        </article>
        <article className={styles.card}>
          <h2>Integrated Actions</h2>
          <strong>{summary.actions}</strong>
        </article>
        <article className={styles.card}>
          <h2>API Base</h2>
          <strong className={styles.apiBase}>{summary.apiBase}</strong>
        </article>
      </section>

      <section className={styles.contextPanel}>
        <h2>Portal context</h2>
        <div className={styles.inputGrid}>
          {[
            ["Org ID", "orgId"],
            ["User ID", "userId"],
            ["Anon User ID", "anonUserId"],
            ["Owner", "owner"],
            ["Repository", "repo"],
            ["Branch", "branch"],
            ["Execution ID", "executionId"],
            ["Repository ID", "repoId"],
          ].map(([label, key]) => (
            <label key={key} className={styles.field}>
              <span>{label}</span>
              <input
                value={ctx[key as keyof ContextState]}
                onChange={(event) =>
                  setCtx((previous) => ({
                    ...previous,
                    [key]: event.target.value,
                  }))
                }
              />
            </label>
          ))}
        </div>
      </section>

      <section className={styles.groups}>
        {ACTION_GROUPS.map((group) => (
          <article key={group.title} className={styles.groupCard}>
            <h2>{group.title}</h2>
            <p>{group.description}</p>
            <div className={styles.actions}>
              {group.actions.map((action) => (
                <button key={action.id} onClick={() => runAction(action)}>
                  <span>{action.label}</span>
                  <small>
                    {action.method} {action.path}
                  </small>
                </button>
              ))}
            </div>
          </article>
        ))}
      </section>

      <section className={styles.responsePanel}>
        <h2>Latest response</h2>
        <p>
          <strong>Action:</strong> {activeAction}
        </p>
        <p>
          <strong>Status:</strong> {status}
        </p>
        <pre>{responseText}</pre>
      </section>
    </div>
  );
}
