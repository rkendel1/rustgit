use super::*;

pub struct VirtualFileSystem {
    root: PathBuf,
}

impl VirtualFileSystem {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn read(&self, relative_path: &str) -> Result<Vec<u8>> {
        let path = self.resolve(relative_path)?;
        Ok(fs::read(path)?)
    }

    pub fn write(&self, relative_path: &str, bytes: &[u8]) -> Result<()> {
        let path = self.resolve(relative_path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, bytes)?;
        Ok(())
    }

    /// List relative paths of all files under the root, up to `limit` entries.
    /// Does not read file contents.
    pub fn list(&self, limit: usize) -> Result<Vec<String>> {
        let mut paths = Vec::new();
        if self.root.exists() {
            list_paths(&self.root, &self.root, &mut paths, limit);
        }
        Ok(paths)
    }

    pub fn snapshot(&self) -> Result<WorkspaceSnapshot> {
        let mut entries = HashMap::new();
        collect_files(&self.root, &self.root, &mut entries)?;
        Ok(WorkspaceSnapshot { entries })
    }

    pub fn restore(&self, snapshot: &WorkspaceSnapshot) -> Result<()> {
        for (relative, bytes) in &snapshot.entries {
            let path = self.resolve(relative)?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, bytes)?;
        }
        Ok(())
    }

    fn resolve(&self, relative_path: &str) -> Result<PathBuf> {
        let path = self.root.join(relative_path);
        if !path.starts_with(&self.root) {
            return Err(RuntimeError::InvalidPath(relative_path.to_string()));
        }
        Ok(path)
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceSnapshot {
    pub entries: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct RestApiSpec {
    pub routes: Vec<&'static str>,
}

fn list_paths(root: &Path, current: &Path, paths: &mut Vec<String>, limit: usize) {
    if paths.len() >= limit {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        if paths.len() >= limit {
            return;
        }
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let is_dir = if file_type.is_symlink() {
            path.metadata().map(|m| m.is_dir()).unwrap_or(false)
        } else {
            file_type.is_dir()
        };
        if is_dir {
            list_paths(root, &path, paths, limit);
        } else {
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            paths.push(relative.to_string_lossy().to_string());
        }
    }
}

fn collect_files(
    root: &Path,
    current: &Path,
    entries: &mut HashMap<String, Vec<u8>>,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let is_dir = if file_type.is_symlink() {
            path.metadata().map(|m| m.is_dir()).unwrap_or(false)
        } else {
            file_type.is_dir()
        };
        if is_dir {
            collect_files(root, &path, entries)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| RuntimeError::InvalidPath(path.display().to_string()))?
                .to_string_lossy()
                .to_string();
            entries.insert(relative, fs::read(&path)?);
        }
    }
    Ok(())
}

pub(crate) fn current_unix_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn parse_workspace_id(request_target: &str) -> Option<String> {
    let normalized = request_target
        .strip_prefix("https://")
        .or_else(|| request_target.strip_prefix("http://"))
        .unwrap_or(request_target);

    if let Some(id) = normalized
        .strip_prefix("workspace-")
        .and_then(|value| value.strip_suffix(".trythissoftware.com"))
    {
        return Some(id.to_string());
    }

    normalized
        .strip_prefix("trythissoftware.com/w/")
        .or_else(|| normalized.strip_prefix("/w/"))
        .map(|id| id.to_string())
}

pub(crate) fn parse_execution_id(request_target: &str) -> Option<String> {
    let raw = request_target
        .strip_prefix("https://trythissoftware.com/e/")
        .or_else(|| request_target.strip_prefix("http://trythissoftware.com/e/"))
        .or_else(|| request_target.strip_prefix("trythissoftware.com/e/"))
        .or_else(|| request_target.strip_prefix("/e/"))
        .or_else(|| request_target.strip_prefix("e/"))?;
    let normalized = raw.split(['?', '#']).next()?.trim_matches('/').to_string();
    (!normalized.is_empty()).then_some(normalized)
}
