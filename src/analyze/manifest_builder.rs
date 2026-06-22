use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Result, RuntimeError};

const ENVIRONMENT_CANDIDATES: [&str; 6] = [
    ".env",
    ".env.local",
    ".env.example",
    ".env.local.example",
    ".env.template",
    ".sample.env",
];
const ENVIRONMENT_CODE_CANDIDATES: [&str; 7] = [
    "docker-compose.yml",
    "docker-compose.yaml",
    "Dockerfile",
    "vite.config.ts",
    "vite.config.js",
    "next.config.js",
    "next.config.mjs",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyzeManifest {
    pub framework: String,
    pub execution: ManifestExecution,
    #[serde(rename = "executionConfidence")]
    pub execution_confidence: ManifestExecutionConfidence,
    pub docker: ManifestDocker,
    #[serde(rename = "packageManager")]
    pub package_manager: Option<String>,
    #[serde(rename = "startCommand")]
    pub start_command: Option<String>,
    #[serde(rename = "buildCommand")]
    pub build_command: Option<String>,
    #[serde(rename = "environmentVariables")]
    pub environment_variables: Vec<ManifestEnvironmentVariable>,
    #[serde(rename = "preferredRuntime")]
    pub preferred_runtime: String,
    #[serde(rename = "recommendedCommand")]
    pub recommended_command: Option<String>,
    #[serde(rename = "nodeVersion")]
    pub node_version: Option<String>,
    #[serde(rename = "autoHealsApplied")]
    pub auto_heals_applied: Vec<String>,
    pub workspace: ManifestWorkspace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestExecution {
    pub preferred: String,
    pub fallback: String,
    pub confidence: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestExecutionConfidence {
    pub score: u8,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestDocker {
    pub dockerfile: bool,
    pub compose: bool,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEnvironmentVariable {
    pub name: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestWorkspace {
    #[serde(rename = "requiresDocker")]
    pub requires_docker: bool,
    #[serde(rename = "requiresSecrets")]
    pub requires_secrets: bool,
}

impl AnalyzeManifest {
    #[allow(clippy::too_many_arguments)]
    pub fn synthesize(
        root: &Path,
        framework: &str,
        runtime: &str,
        package_manager: Option<&str>,
        build_command: Option<&str>,
        start_command: Option<&str>,
        dev_command: Option<&str>,
        confidence: u8,
    ) -> Self {
        let dockerfile = root.join("Dockerfile").exists() || root.join("dockerfile").exists();
        let compose = root.join("docker-compose.yml").exists()
            || root.join("docker-compose.yaml").exists()
            || root.join("compose.yml").exists()
            || root.join("compose.yaml").exists();
        let preferred =
            if dockerfile || compose || root.join(".devcontainer/devcontainer.json").exists() {
                "docker".to_string()
            } else {
                package_manager
                    .filter(|value| !value.is_empty())
                    .unwrap_or(runtime)
                    .to_string()
            };
        let fallback = package_manager
            .filter(|value| !value.is_empty())
            .unwrap_or(runtime)
            .to_string();
        let command = if compose {
            "docker compose up".to_string()
        } else {
            "docker".to_string()
        };
        let start_command = infer_start_command(
            root,
            package_manager,
            start_command,
            dev_command,
            runtime,
            framework,
        );
        let (recommended_command, auto_heals_applied) =
            start_command.as_deref().map_or((None, vec![]), |command| {
                let (healed, heals) = apply_safe_command_heals(command, runtime, framework);
                (Some(healed), heals)
            });
        let build_command = build_command
            .map(ToString::to_string)
            .or_else(|| package_manager.map(default_build_command));
        let environment_variables = discover_environment_variables(root);
        let requires_secrets = environment_variables.iter().any(|entry| entry.required);
        let node_version = infer_node_version(root);
        let mut confidence_reasons = Vec::new();
        if dockerfile || compose || root.join(".devcontainer/devcontainer.json").exists() {
            confidence_reasons.push("Docker detected".to_string());
        }
        if !environment_variables.is_empty() {
            confidence_reasons.push("Environment resolved".to_string());
        }
        if package_manager.is_some() {
            confidence_reasons.push("Package manager verified".to_string());
        }
        if recommended_command.is_some() {
            confidence_reasons.push("Start command verified".to_string());
        }
        if node_version.is_some() {
            confidence_reasons.push("Node version verified".to_string());
        }
        Self {
            framework: framework.to_string(),
            execution: ManifestExecution {
                preferred: preferred.clone(),
                fallback,
                confidence,
            },
            execution_confidence: ManifestExecutionConfidence {
                score: confidence,
                reasons: confidence_reasons,
            },
            docker: ManifestDocker {
                dockerfile,
                compose,
                command,
            },
            package_manager: package_manager.map(ToString::to_string),
            start_command: recommended_command.clone(),
            build_command,
            environment_variables,
            preferred_runtime: preferred.clone(),
            recommended_command,
            node_version,
            auto_heals_applied,
            workspace: ManifestWorkspace {
                requires_docker: dockerfile || compose,
                requires_secrets,
            },
        }
    }
}

fn apply_safe_command_heals(
    command: &str,
    runtime: &str,
    framework: &str,
) -> (String, Vec<String>) {
    let lower = command.to_ascii_lowercase();
    let should_inject_web_flags =
        runtime == "node" || runtime == "bun" || framework.eq_ignore_ascii_case("vite");
    if !should_inject_web_flags {
        return (command.to_string(), vec![]);
    }

    let mut healed = command.trim().to_string();
    let mut heals = Vec::new();

    if !lower.contains("--host") && !lower.contains("hostname") {
        healed.push_str(" --host 0.0.0.0");
        heals.push("hostInjection".to_string());
    }
    if !lower.contains("--port") {
        healed.push_str(" --port {PORT}");
        heals.push("portInjection".to_string());
    }
    (healed, heals)
}

fn infer_node_version(root: &Path) -> Option<String> {
    for file in [".nvmrc", ".node-version"] {
        if let Ok(content) = fs::read_to_string(root.join(file)) {
            let version = content.trim().trim_start_matches('v').to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }
    }
    if let Ok(package_json) = fs::read_to_string(root.join("package.json")) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&package_json) {
            if let Some(engine) = value
                .get("engines")
                .and_then(|engines| engines.get("node"))
                .and_then(serde_json::Value::as_str)
            {
                let version = engine
                    .chars()
                    .filter(|ch| ch.is_ascii_digit() || *ch == '.')
                    .collect::<String>();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
    }
    if let Ok(dockerfile) = fs::read_to_string(root.join("Dockerfile")) {
        for line in dockerfile.lines() {
            let trimmed = line.trim().to_ascii_lowercase();
            if trimmed.starts_with("from node:") {
                let candidate = trimmed
                    .trim_start_matches("from node:")
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_start_matches('v')
                    .to_string();
                if !candidate.is_empty() {
                    return Some(candidate);
                }
            }
        }
    }
    if let Ok(readme) = fs::read_to_string(root.join("README.md")) {
        let lower = readme.to_ascii_lowercase();
        if let Some(index) = lower.find("node ") {
            let suffix = &lower[index + 5..];
            let version = suffix
                .chars()
                .skip_while(|ch| !ch.is_ascii_digit())
                .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
                .collect::<String>();
            if !version.is_empty() {
                return Some(version);
            }
        }
    }
    None
}

pub fn write_manifest(root: &Path, manifest: &AnalyzeManifest) -> Result<()> {
    let payload = serde_json::to_string_pretty(manifest)
        .map_err(|e| RuntimeError::CommandFailed(format!("manifest_serialization_failed: {e}")))?;
    fs::write(root.join(".execution.json"), &payload)?;
    let ddockit_dir = root.join(".ddockit");
    fs::create_dir_all(&ddockit_dir)?;
    fs::write(ddockit_dir.join("manifest.json"), payload)?;
    Ok(())
}

fn default_build_command(package_manager: &str) -> String {
    match package_manager {
        "pnpm" => "pnpm run build".to_string(),
        "yarn" => "yarn build".to_string(),
        "bun" => "bun run build".to_string(),
        "cargo" => "cargo build".to_string(),
        "go" => "go build ./...".to_string(),
        "pip" | "uv" | "poetry" | "pipenv" => "python -m build".to_string(),
        _ => "npm run build".to_string(),
    }
}

fn infer_start_command(
    root: &Path,
    package_manager: Option<&str>,
    start_command: Option<&str>,
    dev_command: Option<&str>,
    runtime: &str,
    framework: &str,
) -> Option<String> {
    if root.join("docker-compose.yml").exists() || root.join("docker-compose.yaml").exists() {
        return Some("docker compose up".to_string());
    }
    if root.join("Dockerfile").exists() || root.join("dockerfile").exists() {
        return Some("docker".to_string());
    }
    if root.join(".devcontainer/devcontainer.json").exists() {
        return Some("devcontainer up --workspace-folder .".to_string());
    }
    if let Ok(procfile) = fs::read_to_string(root.join("Procfile")) {
        if let Some(line) = procfile.lines().find(|line| line.contains(':')) {
            if let Some((_, command)) = line.split_once(':') {
                let command = command.trim();
                if !command.is_empty() {
                    return Some(command.to_string());
                }
            }
        }
    }
    if let Some(command) = start_command.filter(|value| !value.trim().is_empty()) {
        return Some(command.to_string());
    }
    if let Some(command) = dev_command.filter(|value| !value.trim().is_empty()) {
        return Some(command.to_string());
    }
    if let Ok(readme) = fs::read_to_string(root.join("README.md")) {
        for marker in [
            "pnpm run dev",
            "npm run dev",
            "yarn dev",
            "bun run dev",
            "cargo run",
            "python main.py",
        ] {
            if readme.contains(marker) {
                return Some(marker.to_string());
            }
        }
    }
    if root.join("fly.toml").exists() {
        return Some("fly launch".to_string());
    }
    Some(match package_manager.unwrap_or(runtime) {
        "pnpm" => "pnpm run dev -- --host 0.0.0.0 --port {PORT}".to_string(),
        "yarn" => "yarn dev".to_string(),
        "bun" => "bun run dev".to_string(),
        "cargo" => "cargo run".to_string(),
        "go" => "go run .".to_string(),
        "python" | "pip" | "uv" | "poetry" | "pipenv" => "python main.py".to_string(),
        _ => {
            if framework.eq_ignore_ascii_case("vite") {
                "npm run dev -- --host 0.0.0.0 --port {PORT}".to_string()
            } else {
                "npm run dev".to_string()
            }
        }
    })
}

fn discover_environment_variables(root: &Path) -> Vec<ManifestEnvironmentVariable> {
    let mut names = BTreeSet::new();
    for file in ENVIRONMENT_CANDIDATES {
        let path = root.join(file);
        if let Ok(content) = fs::read_to_string(path) {
            names.extend(extract_env_keys_from_text(&content));
        }
    }
    for file in ENVIRONMENT_CODE_CANDIDATES {
        let path = root.join(file);
        if let Ok(content) = fs::read_to_string(path) {
            names.extend(extract_env_keys_from_text(&content));
        }
    }
    let mut variables = names
        .into_iter()
        .filter(|name| !name.is_empty())
        .map(|name| ManifestEnvironmentVariable {
            required: true,
            name,
        })
        .collect::<Vec<_>>();
    variables.sort_by(|left, right| left.name.cmp(&right.name));
    variables
}

fn extract_env_keys_from_text(content: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            let key = key.trim();
            if is_env_key(key) {
                keys.insert(key.to_string());
            }
        }
    }
    collect_dot_notation(content, "process.env.", &mut keys);
    collect_dot_notation(content, "import.meta.env.", &mut keys);
    collect_quoted_notation(content, "os.getenv(", &mut keys);
    collect_quoted_notation(content, "std::env::var(", &mut keys);
    collect_quoted_notation(content, "getenv(", &mut keys);
    collect_template_env(content, &mut keys);
    keys
}

fn is_env_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(is_env_key_char)
}

fn is_env_key_char(ch: char) -> bool {
    ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_'
}

fn collect_dot_notation(content: &str, prefix: &str, keys: &mut BTreeSet<String>) {
    let mut rest = content;
    while let Some(index) = rest.find(prefix) {
        let candidate = &rest[index + prefix.len()..];
        let key: String = candidate
            .chars()
            .take_while(|ch| is_env_key_char(*ch))
            .collect();
        if is_env_key(&key) {
            keys.insert(key);
        }
        rest = &candidate[1.min(candidate.len())..];
    }
}

fn collect_quoted_notation(content: &str, prefix: &str, keys: &mut BTreeSet<String>) {
    let mut rest = content;
    while let Some(index) = rest.find(prefix) {
        let candidate = &rest[index + prefix.len()..];
        let Some(quote) = candidate.chars().next() else {
            break;
        };
        if quote != '"' && quote != '\'' {
            rest = &candidate[1.min(candidate.len())..];
            continue;
        }
        if let Some(end_index) = candidate[1..].find(quote) {
            let key = &candidate[1..1 + end_index];
            if is_env_key(key) {
                keys.insert(key.to_string());
            }
            rest = &candidate[1 + end_index + 1..];
        } else {
            break;
        }
    }
}

fn collect_template_env(content: &str, keys: &mut BTreeSet<String>) {
    let mut rest = content;
    while let Some(index) = rest.find("${") {
        let candidate = &rest[index + 2..];
        let key: String = candidate
            .chars()
            .take_while(|ch| is_env_key_char(*ch))
            .collect();
        if is_env_key(&key) {
            keys.insert(key);
        }
        rest = &candidate[1.min(candidate.len())..];
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{infer_node_version, AnalyzeManifest};

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{ts}"))
    }

    #[test]
    fn synthesize_applies_host_and_port_auto_heals() {
        let root = unique_temp_dir("rustgit-manifest-heals");
        fs::create_dir_all(&root).expect("create root");
        fs::write(
            root.join("package.json"),
            r#"{"scripts":{"dev":"vite"},"dependencies":{"vite":"5.0.0"}}"#,
        )
        .expect("write package");

        let manifest = AnalyzeManifest::synthesize(
            &root,
            "vite",
            "node",
            Some("pnpm"),
            None,
            None,
            Some("pnpm run dev"),
            95,
        );

        assert_eq!(manifest.preferred_runtime, "pnpm");
        assert_eq!(
            manifest.recommended_command.as_deref(),
            Some("pnpm run dev --host 0.0.0.0 --port {PORT}")
        );
        assert!(manifest
            .auto_heals_applied
            .contains(&"hostInjection".to_string()));
        assert!(manifest
            .auto_heals_applied
            .contains(&"portInjection".to_string()));
    }

    #[test]
    fn infer_node_version_prefers_nvmrc() {
        let root = unique_temp_dir("rustgit-manifest-node-version");
        fs::create_dir_all(&root).expect("create root");
        fs::write(root.join(".nvmrc"), "v22\n").expect("write nvmrc");
        assert_eq!(infer_node_version(&root).as_deref(), Some("22"));
    }
}
