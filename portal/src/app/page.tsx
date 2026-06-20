const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL ??
  (process.env.NODE_ENV === "development"
    ? "http://localhost:8080"
    : "https://api.trythissoftware.com");

export default function Home() {
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
          Welcome to the deployed portal home page.
        </p>
        <p style={{ marginBottom: "0.75rem" }}>
          API base: <code>{API_BASE_URL}</code>
        </p>
        <p>
          Health check: <a href="/api/health">/api/health</a>
        </p>
        <section
          style={{
            marginTop: "1.25rem",
            borderTop: "1px solid #e2e8f0",
            paddingTop: "1rem",
          }}
        >
          <h2 style={{ marginBottom: "0.5rem" }}>Repository Intelligence</h2>
          <p style={{ marginBottom: "0.5rem" }}>
            Ask Repository: <code>POST /api/repositories/{`{id}`}/ask</code>
          </p>
          <p style={{ marginBottom: "0.5rem" }}>Recent Questions: Coming from EIDB knowledge history.</p>
          <ul style={{ margin: 0, paddingLeft: "1.25rem" }}>
            <li>Can this repository run?</li>
            <li>Why did the last build fail?</li>
            <li>What runtime performs best?</li>
            <li>How can this be healed?</li>
            <li>What changed since the last successful run?</li>
          </ul>
        </section>
      </section>
    </main>
  );
}
