use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::{Result, RuntimeError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalyzeManifest {
    pub runtime: String,
    pub framework: String,
    #[serde(rename = "packageManager")]
    pub package_manager: Option<String>,
    pub build: Option<String>,
    pub start: Option<String>,
    pub dev: Option<String>,
    pub confidence: u8,
}

pub fn write_manifest(root: &Path, manifest: &AnalyzeManifest) -> Result<()> {
    let ddockit_dir = root.join(".ddockit");
    fs::create_dir_all(&ddockit_dir)?;
    let path = ddockit_dir.join("manifest.json");
    let payload = serde_json::to_string_pretty(manifest)
        .map_err(|e| RuntimeError::CommandFailed(format!("manifest_serialization_failed: {e}")))?;
    fs::write(path, payload)?;
    Ok(())
}
