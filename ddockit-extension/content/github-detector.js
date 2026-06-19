function parseGithubRepository(pathname) {
  const segments = pathname.split("/").filter(Boolean);
  if (segments.length < 2) {
    return null;
  }

  const [owner, repo] = segments;
  const invalid = new Set(["settings", "orgs", "users", "marketplace", "features", "topics", "search", "new"]);
  if (invalid.has(owner)) {
    return null;
  }

  return {
    owner,
    repo,
    branch: new URLSearchParams(window.location.search).get("branch") || "main",
    url: window.location.href
  };
}

const repositoryContext = parseGithubRepository(window.location.pathname);
if (repositoryContext) {
  chrome.runtime.sendMessage({
    type: "DDOCKIT_DETECTED_REPOSITORY",
    payload: repositoryContext
  });

  window.__ddockitRepositoryContext = repositoryContext;
}
