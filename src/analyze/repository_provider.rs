use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Result, RuntimeError};

const RUNTIME_DISCOVERY_FILES: [&str; 19] = [
    "package.json",
    "pnpm-lock.yaml",
    "bun.lockb",
    "yarn.lock",
    "Cargo.toml",
    "Cargo.lock",
    "requirements.txt",
    "pyproject.toml",
    "poetry.lock",
    "uv.lock",
    "Dockerfile",
    "docker-compose.yml",
    "fly.toml",
    "next.config.js",
    "vite.config.ts",
    "turbo.json",
    "nx.json",
    "deno.json",
    "deno.jsonc",
];

const SKIPPED_DIRECTORIES: [&str; 8] = [
    ".git/",
    "node_modules/",
    "dist/",
    "build/",
    "coverage/",
    "vendor/",
    "target/",
    ".next/",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub provider: String,
    pub owner: String,
    pub repository: String,
    pub branch: String,
    pub commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryTreeNode {
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub sha: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryTree {
    pub root: String,
    pub children: Vec<RepositoryTreeNode>,
    pub directories: Vec<String>,
    pub files: Vec<String>,
    pub size: u64,
    pub sha: Option<String>,
    pub provider: String,
    pub branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryFile {
    pub path: String,
    pub sha: Option<String>,
    pub content: Vec<u8>,
}

#[async_trait]
pub trait RepositoryProvider: Send + Sync {
    async fn metadata(&self) -> Result<RepositoryMetadata>;
    async fn tree(&self) -> Result<RepositoryTree>;
    async fn exists(&self, path: &str) -> Result<bool>;
    async fn read(&self, path: &str) -> Result<RepositoryFile>;
    async fn download(&self, paths: &[String], destination: &Path) -> Result<()>;
}

pub fn runtime_discovery_paths(tree: &RepositoryTree) -> Vec<String> {
    let files = tree
        .files
        .iter()
        .filter(|path| !is_skipped_path(path))
        .filter(|path| {
            let name = Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            RUNTIME_DISCOVERY_FILES.contains(&name)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    files.into_iter().collect()
}

fn is_skipped_path(path: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|prefix| path.starts_with(prefix) || path.contains(&format!("/{prefix}")))
}

#[derive(Debug, Clone)]
pub struct LocalWorkspaceProvider {
    root: PathBuf,
}

impl LocalWorkspaceProvider {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn walk(root: &Path, path: &Path, files: &mut Vec<String>, directories: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let mut relative_text = relative.to_string_lossy().replace('\\', "/");
            if relative_text.is_empty() {
                continue;
            }
            if path.is_dir() {
                relative_text.push('/');
                if !is_skipped_path(&relative_text) {
                    directories.push(relative_text.clone());
                    Self::walk(root, &path, files, directories);
                }
            } else if !is_skipped_path(&relative_text) {
                files.push(relative_text);
            }
        }
    }
}

#[async_trait]
impl RepositoryProvider for LocalWorkspaceProvider {
    async fn metadata(&self) -> Result<RepositoryMetadata> {
        Ok(RepositoryMetadata {
            provider: "local".to_string(),
            owner: "local".to_string(),
            repository: self
                .root
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("workspace")
                .to_string(),
            branch: "local".to_string(),
            commit: None,
        })
    }

    async fn tree(&self) -> Result<RepositoryTree> {
        let mut files = Vec::new();
        let mut directories = Vec::new();
        Self::walk(&self.root, &self.root, &mut files, &mut directories);
        files.sort();
        directories.sort();
        let children = directories
            .iter()
            .map(|path| RepositoryTreeNode {
                path: path.clone(),
                is_directory: true,
                size: None,
                sha: None,
            })
            .chain(files.iter().map(|path| RepositoryTreeNode {
                path: path.clone(),
                is_directory: false,
                size: fs::metadata(self.root.join(path)).ok().map(|m| m.len()),
                sha: None,
            }))
            .collect::<Vec<_>>();
        let size = files
            .iter()
            .filter_map(|path| fs::metadata(self.root.join(path)).ok().map(|m| m.len()))
            .sum();
        Ok(RepositoryTree {
            root: "/".to_string(),
            children,
            directories,
            files,
            size,
            sha: None,
            provider: "local".to_string(),
            branch: "local".to_string(),
        })
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        Ok(self.root.join(path).exists())
    }

    async fn read(&self, path: &str) -> Result<RepositoryFile> {
        let content = fs::read(self.root.join(path))?;
        Ok(RepositoryFile {
            path: path.to_string(),
            sha: None,
            content,
        })
    }

    async fn download(&self, paths: &[String], destination: &Path) -> Result<()> {
        for path in paths {
            let source = self.root.join(path);
            let target = destination.join(path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source, target)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ForgeRepositoryProvider {
    provider: String,
    host: String,
    owner: String,
    repository: String,
    branch: String,
    commit: Option<String>,
    client: reqwest::Client,
}

impl ForgeRepositoryProvider {
    pub fn from_url(repo_url: &str, branch: &str, commit: &str) -> Option<Self> {
        let repo_url = repo_url.trim().trim_end_matches(".git");
        let parsed = Url::parse(repo_url).ok()?;
        let host = parsed.host_str()?.to_ascii_lowercase();
        let mut segments = parsed.path_segments()?;
        let owner = segments.next()?.trim_matches('/').to_string();
        let repository = segments.next()?.trim_matches('/').to_string();
        if owner.is_empty() || repository.is_empty() {
            return None;
        }
        let provider = if host == "github.com" {
            "github"
        } else if host == "gitlab.com" {
            "gitlab"
        } else if host == "codeberg.org" {
            "codeberg"
        } else if host.contains("forgejo") {
            "forgejo"
        } else if host.contains("gitea") {
            "gitea"
        } else {
            return None;
        };
        let branch = if branch.trim().is_empty() {
            "main".to_string()
        } else {
            branch.trim().to_string()
        };
        let commit = (!commit.trim().is_empty()).then(|| commit.trim().to_string());
        Some(Self {
            provider: provider.to_string(),
            host,
            owner,
            repository,
            branch,
            commit,
            client: reqwest::Client::new(),
        })
    }

    fn raw_url(&self, path: &str) -> String {
        match self.provider.as_str() {
            "github" => format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                self.owner, self.repository, self.branch, path
            ),
            "gitlab" => format!(
                "https://gitlab.com/{}/{}/-/raw/{}/{}",
                self.owner, self.repository, self.branch, path
            ),
            _ => format!(
                "https://{}/{}/{}/raw/branch/{}/{}",
                self.host, self.owner, self.repository, self.branch, path
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitHubTreeResponse {
    sha: String,
    tree: Vec<GitTreeItem>,
}

#[derive(Debug, Deserialize)]
struct GitTreeItem {
    path: String,
    #[serde(rename = "type")]
    kind: String,
    sha: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct GitLabTreeItem {
    path: String,
    #[serde(rename = "type")]
    kind: String,
}

#[async_trait]
impl RepositoryProvider for ForgeRepositoryProvider {
    async fn metadata(&self) -> Result<RepositoryMetadata> {
        Ok(RepositoryMetadata {
            provider: self.provider.clone(),
            owner: self.owner.clone(),
            repository: self.repository.clone(),
            branch: self.branch.clone(),
            commit: self.commit.clone(),
        })
    }

    async fn tree(&self) -> Result<RepositoryTree> {
        let (files, directories, children, sha) = if self.provider == "github" {
            let url = format!(
                "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
                self.owner, self.repository, self.branch
            );
            let response = self
                .client
                .get(url)
                .header("User-Agent", "rustgit")
                .send()
                .await
                .map_err(|err| RuntimeError::CommandFailed(format!("provider tree failed: {err}")))?;
            if !response.status().is_success() {
                return Err(RuntimeError::CommandFailed(format!(
                    "provider tree failed with status {}",
                    response.status()
                )));
            }
            let payload = response
                .json::<GitHubTreeResponse>()
                .await
                .map_err(|err| RuntimeError::CommandFailed(format!("provider tree decode failed: {err}")))?;
            let files = payload
                .tree
                .iter()
                .filter(|entry| entry.kind == "blob")
                .map(|entry| entry.path.clone())
                .collect::<Vec<_>>();
            let directories = payload
                .tree
                .iter()
                .filter(|entry| entry.kind == "tree")
                .map(|entry| format!("{}/", entry.path))
                .collect::<Vec<_>>();
            let children = payload
                .tree
                .into_iter()
                .map(|entry| RepositoryTreeNode {
                    path: entry.path,
                    is_directory: entry.kind == "tree",
                    size: entry.size,
                    sha: entry.sha,
                })
                .collect::<Vec<_>>();
            (files, directories, children, Some(payload.sha))
        } else if self.provider == "gitlab" {
            let project = format!("{}/{}", self.owner, self.repository).replace('/', "%2F");
            let url = format!(
                "https://gitlab.com/api/v4/projects/{project}/repository/tree?recursive=true&per_page=100&ref={}",
                self.branch
            );
            let response = self
                .client
                .get(url)
                .header("User-Agent", "rustgit")
                .send()
                .await
                .map_err(|err| RuntimeError::CommandFailed(format!("provider tree failed: {err}")))?;
            if !response.status().is_success() {
                return Err(RuntimeError::CommandFailed(format!(
                    "provider tree failed with status {}",
                    response.status()
                )));
            }
            let payload = response
                .json::<Vec<GitLabTreeItem>>()
                .await
                .map_err(|err| RuntimeError::CommandFailed(format!("provider tree decode failed: {err}")))?;
            let files = payload
                .iter()
                .filter(|entry| entry.kind == "blob")
                .map(|entry| entry.path.clone())
                .collect::<Vec<_>>();
            let directories = payload
                .iter()
                .filter(|entry| entry.kind == "tree")
                .map(|entry| format!("{}/", entry.path))
                .collect::<Vec<_>>();
            let children = payload
                .into_iter()
                .map(|entry| RepositoryTreeNode {
                    path: entry.path,
                    is_directory: entry.kind == "tree",
                    size: None,
                    sha: None,
                })
                .collect::<Vec<_>>();
            (files, directories, children, None)
        } else {
            let url = format!(
                "https://{}/api/v1/repos/{}/{}/git/trees/{}?recursive=true",
                self.host, self.owner, self.repository, self.branch
            );
            let response = self
                .client
                .get(url)
                .header("User-Agent", "rustgit")
                .send()
                .await
                .map_err(|err| RuntimeError::CommandFailed(format!("provider tree failed: {err}")))?;
            if !response.status().is_success() {
                return Err(RuntimeError::CommandFailed(format!(
                    "provider tree failed with status {}",
                    response.status()
                )));
            }
            let payload = response
                .json::<GitHubTreeResponse>()
                .await
                .map_err(|err| RuntimeError::CommandFailed(format!("provider tree decode failed: {err}")))?;
            let files = payload
                .tree
                .iter()
                .filter(|entry| entry.kind == "blob")
                .map(|entry| entry.path.clone())
                .collect::<Vec<_>>();
            let directories = payload
                .tree
                .iter()
                .filter(|entry| entry.kind == "tree")
                .map(|entry| format!("{}/", entry.path))
                .collect::<Vec<_>>();
            let children = payload
                .tree
                .into_iter()
                .map(|entry| RepositoryTreeNode {
                    path: entry.path,
                    is_directory: entry.kind == "tree",
                    size: entry.size,
                    sha: entry.sha,
                })
                .collect::<Vec<_>>();
            (files, directories, children, Some(payload.sha))
        };

        let size = children.iter().filter_map(|entry| entry.size).sum();
        Ok(RepositoryTree {
            root: "/".to_string(),
            children,
            directories,
            files,
            size,
            sha,
            provider: self.provider.clone(),
            branch: self.branch.clone(),
        })
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let tree = self.tree().await?;
        Ok(tree.files.iter().any(|candidate| candidate == path))
    }

    async fn read(&self, path: &str) -> Result<RepositoryFile> {
        let response = self
            .client
            .get(self.raw_url(path))
            .header("User-Agent", "rustgit")
            .send()
            .await
            .map_err(|err| RuntimeError::CommandFailed(format!("provider read failed: {err}")))?;
        if !response.status().is_success() {
            return Err(RuntimeError::CommandFailed(format!(
                "provider read failed with status {} for {path}",
                response.status()
            )));
        }
        let content = response
            .bytes()
            .await
            .map_err(|err| RuntimeError::CommandFailed(format!("provider read body failed: {err}")))?
            .to_vec();
        Ok(RepositoryFile {
            path: path.to_string(),
            sha: None,
            content,
        })
    }

    async fn download(&self, paths: &[String], destination: &Path) -> Result<()> {
        for path in paths {
            let file = self.read(path).await?;
            let target = destination.join(path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target, file.content)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{runtime_discovery_paths, ForgeRepositoryProvider, RepositoryTree};

    #[test]
    fn runtime_discovery_paths_filters_to_runtime_files_and_skips_heavy_dirs() {
        let tree = RepositoryTree {
            root: "/".to_string(),
            children: vec![],
            directories: vec![],
            files: vec![
                "package.json".to_string(),
                "apps/web/package.json".to_string(),
                "apps/web/next.config.js".to_string(),
                "node_modules/pkg/package.json".to_string(),
                "dist/package.json".to_string(),
                "README.md".to_string(),
                "services/api/Cargo.toml".to_string(),
            ],
            size: 0,
            sha: None,
            provider: "github".to_string(),
            branch: "main".to_string(),
        };
        let paths = runtime_discovery_paths(&tree);
        assert_eq!(
            paths,
            vec![
                "apps/web/next.config.js".to_string(),
                "apps/web/package.json".to_string(),
                "package.json".to_string(),
                "services/api/Cargo.toml".to_string()
            ]
        );
    }

    #[test]
    fn provider_factory_parses_supported_forges() {
        let github =
            ForgeRepositoryProvider::from_url("https://github.com/org/repo.git", "main", "");
        assert!(github.is_some());
        let gitlab = ForgeRepositoryProvider::from_url("https://gitlab.com/org/repo", "main", "");
        assert!(gitlab.is_some());
        let codeberg =
            ForgeRepositoryProvider::from_url("https://codeberg.org/org/repo", "main", "");
        assert!(codeberg.is_some());
    }
}
