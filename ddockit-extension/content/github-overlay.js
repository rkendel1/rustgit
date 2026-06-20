const BUTTON_ID = "ddockit-run-button";
const ASK_BUTTON_ID = "ddockit-ask-repository-button";

function ensureButton() {
  if (document.getElementById(BUTTON_ID) && document.getElementById(ASK_BUTTON_ID)) {
    return;
  }

  const toolbar = document.querySelector(".file-navigation") || document.querySelector("#repository-container-header");
  if (!toolbar) {
    return;
  }

  const runButton = document.createElement("button");
  runButton.id = BUTTON_ID;
  runButton.type = "button";
  runButton.className = "ddockit-button";
  runButton.textContent = "Run with TryThisSoftware";
  runButton.addEventListener("click", async () => {
    const payload = window.__ddockitRepositoryContext;
    if (!payload?.owner || !payload?.repo) {
      console.warn("TryThisSoftware repository context unavailable on this page.");
      return;
    }
    await chrome.runtime.sendMessage({ type: "DDOCKIT_OPEN_SIDEPANEL" });
    await chrome.runtime.sendMessage({ type: "DDOCKIT_DETECTED_REPOSITORY", payload });
  });

  const askButton = document.createElement("button");
  askButton.id = ASK_BUTTON_ID;
  askButton.type = "button";
  askButton.className = "ddockit-button";
  askButton.style.marginLeft = "0.5rem";
  askButton.textContent = "Ask Repository";
  askButton.addEventListener("click", async () => {
    await chrome.runtime.sendMessage({ type: "DDOCKIT_OPEN_SIDEPANEL" });
  });

  toolbar.appendChild(runButton);
  toolbar.appendChild(askButton);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", ensureButton, { once: true });
} else {
  ensureButton();
}
