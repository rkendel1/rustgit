"use client";

import { useMemo, useState } from "react";

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL ??
  (process.env.NODE_ENV === "development"
    ? "http://localhost:8080"
    : "https://api.trythissoftware.com");

type RepoContext = {
  owner: string;
  repo: string;
  repoUrl: string;
};

type AnalyzeResponse = {
  repo_url?: string;
  fingerprint_id?: string;
  frameworks?: string[];
  services?: string[];
};

type RunResponse = {
  execution_id?: string;
  workspace_id?: string;
  workspace_url?: string;
  status?: string;
};

function createAnonymousId(prefix: string): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `${prefix}-${crypto.randomUUID()}`;
  }
  if (typeof crypto !== "undefined" && typeof crypto.getRandomValues === "function") {
    const bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    const hex = Array.from(bytes, (value) => value.toString(16).padStart(2, "0")).join("");
    return `${prefix}-${hex}`;
  }
  return `${prefix}-${Date.now()}`;
}

function parseRepositoryInput(input: string): RepoContext | null {
  const trimmed = input.trim();
  if (!trimmed) {
    return null;
  }

  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    try {
      const url = new URL(trimmed);
      if (url.hostname !== "github.com" && url.hostname !== "www.github.com") {
        return null;
      }
      const segments = url.pathname
        .replace(/\.git$/, "")
        .split("/")
        .filter(Boolean);
      if (segments.length < 2) {
        return null;
      }
      const owner = segments[0];
      const repo = segments[1];
      return {
        owner,
        repo,
        repoUrl: `https://github.com/${owner}/${repo}`,
      };
    } catch {
      return null;
    }
  }

  const segments = trimmed
    .replace(/\.git$/, "")
    .split("/")
    .map((segment) => segment.trim())
    .filter(Boolean);
  if (segments.length !== 2) {
    return null;
  }

  const [owner, repo] = segments;
  return {
    owner,
    repo,
    repoUrl: `https://github.com/${owner}/${repo}`,
  };
}

export default function Home() {
  const [repository, setRepository] = useState("");
  const [branch, setBranch] = useState("main");
  const [analyzing, setAnalyzing] = useState(false);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [analyzeResult, setAnalyzeResult] = useState<AnalyzeResponse | null>(null);
  const [analyzedRepoUrl, setAnalyzedRepoUrl] = useState<string | null>(null);
  const [runResult, setRunResult] = useState<RunResponse | null>(null);

  const parsedRepo = useMemo(() => parseRepositoryInput(repository), [repository]);
  const canRun =
    !running &&
    Boolean(parsedRepo) &&
    Boolean(analyzeResult) &&
    analyzedRepoUrl === parsedRepo?.repoUrl;

  async function handleAnalyze() {
    if (!parsedRepo) {
      setError("Enter a GitHub repository as owner/repo or full URL.");
      return;
    }

    setError(null);
    setRunResult(null);
    setAnalyzing(true);
    try {
      const response = await fetch("/api/proxy/api/v1/repositories/analyze", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ repo_url: parsedRepo.repoUrl }),
      });

      const responseText = await response.text();
      if (!response.ok) {
        throw new Error(
          `Analyze failed (${response.status}): ${responseText || "no response body"}`,
        );
      }
      const body = JSON.parse(responseText) as AnalyzeResponse;
      setAnalyzeResult(body);
      setAnalyzedRepoUrl(parsedRepo.repoUrl);
    } catch (caught) {
      setAnalyzeResult(null);
      setAnalyzedRepoUrl(null);
      setError(caught instanceof Error ? caught.message : "Analyze request failed.");
    } finally {
      setAnalyzing(false);
    }
  }

  async function handleRun() {
    if (!parsedRepo) {
      setError("Enter a GitHub repository first.");
      return;
    }

    setError(null);
    setRunning(true);
    try {
      const response = await fetch("/api/proxy/api/v1/executions", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          org_id: null,
          user_id: null,
          anon_user_id: createAnonymousId("anon-portal"),
          anon_session_id: createAnonymousId("portal-session"),
          device_fingerprint: "portal-home",
          repo_url: parsedRepo.repoUrl,
          branch: branch.trim() || "main",
          commit: null,
        }),
      });

      const responseText = await response.text();
      if (!response.ok) {
        throw new Error(`Run failed (${response.status}): ${responseText || "no response body"}`);
      }
      const body = JSON.parse(responseText) as RunResponse;
      setRunResult(body);
    } catch (caught) {
      setRunResult(null);
      setError(caught instanceof Error ? caught.message : "Run request failed.");
    } finally {
      setRunning(false);
    }
  }

  return (
    <main
      style={{
        minHeight: "100vh",
        display: "grid",
        placeItems: "center",
        padding: "2rem 1rem",
      }}
    >
      <section
        style={{
          width: "100%",
          maxWidth: "42rem",
          border: "1px solid #dbeafe",
          borderRadius: "0.75rem",
          padding: "1.5rem",
          background: "#ffffff",
          color: "#0f172a",
        }}
      >
        <h1 style={{ marginBottom: "0.75rem" }}>TryThisSoftware Portal</h1>
        <p style={{ marginBottom: "0.75rem" }}>
          Add a repository, analyze it, then run it.
        </p>

        <section style={{ display: "grid", gap: "0.75rem" }}>
          <label htmlFor="repo" style={{ fontWeight: 600 }}>
            Repository
          </label>
          <input
            id="repo"
            value={repository}
            onChange={(event) => {
              setRepository(event.target.value);
              setAnalyzeResult(null);
              setAnalyzedRepoUrl(null);
              setRunResult(null);
              setError(null);
            }}
            placeholder="owner/repo or https://github.com/owner/repo"
            style={{
              border: "1px solid #cbd5e1",
              borderRadius: "0.5rem",
              padding: "0.6rem 0.75rem",
            }}
          />

          <label htmlFor="branch" style={{ fontWeight: 600 }}>
            Branch
          </label>
          <input
            id="branch"
            value={branch}
            onChange={(event) => setBranch(event.target.value)}
            placeholder="main"
            style={{
              border: "1px solid #cbd5e1",
              borderRadius: "0.5rem",
              padding: "0.6rem 0.75rem",
            }}
          />

          <div style={{ display: "flex", gap: "0.75rem", flexWrap: "wrap" }}>
            <button
              type="button"
              onClick={handleAnalyze}
              disabled={analyzing}
              style={{
                background: "#1d4ed8",
                color: "#ffffff",
                border: 0,
                borderRadius: "0.5rem",
                padding: "0.6rem 0.9rem",
                cursor: analyzing ? "wait" : "pointer",
              }}
            >
              {analyzing ? "Analyzing..." : "1) Analyze"}
            </button>
            <button
              type="button"
              onClick={handleRun}
              disabled={!canRun}
              style={{
                background: "#0f766e",
                color: "#ffffff",
                border: 0,
                borderRadius: "0.5rem",
                padding: "0.6rem 0.9rem",
                cursor: canRun ? "pointer" : "not-allowed",
              }}
            >
              {running ? "Running..." : "2) Run"}
            </button>
          </div>
        </section>

        {error ? (
          <p style={{ marginTop: "0.9rem", color: "#b91c1c" }} role="alert">
            {error}
          </p>
        ) : null}

        {analyzeResult ? (
          <section
            style={{
              marginTop: "1rem",
              borderTop: "1px solid #e2e8f0",
              paddingTop: "1rem",
            }}
          >
            <h2 style={{ marginBottom: "0.5rem" }}>Analysis</h2>
            <p style={{ marginBottom: "0.35rem" }}>
              Repository: <code>{analyzeResult.repo_url ?? "n/a"}</code>
            </p>
            <p style={{ marginBottom: "0.35rem" }}>
              Fingerprint: <code>{analyzeResult.fingerprint_id ?? "pending"}</code>
            </p>
            <p style={{ marginBottom: "0.35rem" }}>
              Frameworks: {(analyzeResult.frameworks ?? []).join(", ") || "n/a"}
            </p>
            <p>
              Services: {(analyzeResult.services ?? []).join(", ") || "n/a"}
            </p>
          </section>
        ) : null}

        {runResult ? (
          <section
            style={{
              marginTop: "1rem",
              borderTop: "1px solid #e2e8f0",
              paddingTop: "1rem",
            }}
          >
            <h2 style={{ marginBottom: "0.5rem" }}>Execution</h2>
            <p style={{ marginBottom: "0.35rem" }}>
              Execution ID: <code>{runResult.execution_id ?? "n/a"}</code>
            </p>
            <p style={{ marginBottom: "0.35rem" }}>
              Workspace ID: <code>{runResult.workspace_id ?? "n/a"}</code>
            </p>
            <p style={{ marginBottom: "0.35rem" }}>Status: {runResult.status ?? "starting"}</p>
            {runResult.workspace_url ? (
              <p>
                Workspace URL: <a href={runResult.workspace_url}>{runResult.workspace_url}</a>
              </p>
            ) : null}
          </section>
        ) : null}

        <section
          style={{
            marginTop: "1.25rem",
            borderTop: "1px solid #e2e8f0",
            paddingTop: "1rem",
            color: "#334155",
          }}
        >
          <p style={{ marginBottom: "0.4rem" }}>
            API base: <code>{API_BASE_URL}</code>
          </p>
          <p>
            Health check: <a href="/api/health">/api/health</a>
          </p>
        </section>
      </section>
    </main>
  );
}
