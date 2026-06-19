import { launchExecution } from "../api/ddockit-client.js";

const repoInput = document.getElementById("repo");
const branchInput = document.getElementById("branch");
const status = document.getElementById("status");
const workspaceLink = document.getElementById("workspace-link");
const launchButton = document.getElementById("launch");

init();

async function init() {
  const detected = await chrome.storage.session.get("detectedRepository");
  const repository = detected.detectedRepository;
  if (repository?.owner && repository?.repo) {
    repoInput.value = `${repository.owner}/${repository.repo}`;
    branchInput.value = repository.branch || "main";
  }
}

launchButton.addEventListener("click", async () => {
  const [owner, repo] = repoInput.value.split("/").map((part) => part.trim());
  const branch = branchInput.value.trim() || "main";

  if (!owner || !repo) {
    status.textContent = "Enter a valid owner/repo.";
    return;
  }

  setStatus("Launching...");
  workspaceLink.textContent = "";
  workspaceLink.removeAttribute("href");

  try {
    const data = await launchExecution({ owner, repo, branch });
    setStatus(`Execution ${data.execution_id || "started"}`);
    if (data.workspace_url) {
      workspaceLink.href = data.workspace_url;
      workspaceLink.textContent = data.workspace_url;
    }
  } catch (error) {
    setStatus(String(error));
  }
});

function setStatus(value) {
  status.textContent = value;
}
