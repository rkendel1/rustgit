use std::fs;
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrameworkDetection {
    pub framework: String,
    pub language: String,
    pub evidence: Vec<String>,
}

pub fn detect_framework(root: &Path) -> FrameworkDetection {
    let mut evidence = Vec::new();

    if let Ok(package_json) = fs::read_to_string(root.join("package.json")) {
        let package_lc = package_json.to_ascii_lowercase();
        if package_lc.contains("\"next\"") {
            evidence.push("package.json:next".to_string());
            return FrameworkDetection {
                framework: "nextjs".to_string(),
                language: infer_node_language(root, &package_lc),
                evidence,
            };
        }
        if package_lc.contains("\"vite\"") {
            evidence.push("package.json:vite".to_string());
            return FrameworkDetection {
                framework: "vite".to_string(),
                language: infer_node_language(root, &package_lc),
                evidence,
            };
        }
        if package_lc.contains("\"svelte\"") {
            evidence.push("package.json:svelte".to_string());
            return FrameworkDetection {
                framework: "svelte".to_string(),
                language: infer_node_language(root, &package_lc),
                evidence,
            };
        }
        if package_lc.contains("\"react\"") {
            evidence.push("package.json:react".to_string());
            return FrameworkDetection {
                framework: "react".to_string(),
                language: infer_node_language(root, &package_lc),
                evidence,
            };
        }
        if package_lc.contains("\"express\"") {
            evidence.push("package.json:express".to_string());
            return FrameworkDetection {
                framework: "express".to_string(),
                language: infer_node_language(root, &package_lc),
                evidence,
            };
        }
    }

    if root.join("Cargo.toml").exists() {
        evidence.push("Cargo.toml".to_string());
        return FrameworkDetection {
            framework: "rust".to_string(),
            language: "rust".to_string(),
            evidence,
        };
    }
    if root.join("go.mod").exists() {
        evidence.push("go.mod".to_string());
        return FrameworkDetection {
            framework: "go".to_string(),
            language: "go".to_string(),
            evidence,
        };
    }
    if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        if root.join("manage.py").exists() || file_contains_token(root, "requirements.txt", "django")
        {
            evidence.push("python:django".to_string());
            return FrameworkDetection {
                framework: "django".to_string(),
                language: "python".to_string(),
                evidence,
            };
        }
        if file_contains_token(root, "requirements.txt", "fastapi")
            || file_contains_token(root, "pyproject.toml", "fastapi")
            || file_contains_token(root, "main.py", "fastapi")
        {
            evidence.push("python:fastapi".to_string());
            return FrameworkDetection {
                framework: "fastapi".to_string(),
                language: "python".to_string(),
                evidence,
            };
        }
        evidence.push("python-manifest".to_string());
        return FrameworkDetection {
            framework: "python".to_string(),
            language: "python".to_string(),
            evidence,
        };
    }
    if root.join("pom.xml").exists() {
        evidence.push("pom.xml".to_string());
        return FrameworkDetection {
            framework: "java".to_string(),
            language: "java".to_string(),
            evidence,
        };
    }
    if root.join("composer.json").exists() {
        evidence.push("composer.json".to_string());
        return FrameworkDetection {
            framework: "php".to_string(),
            language: "php".to_string(),
            evidence,
        };
    }
    if root.join("Gemfile").exists() {
        evidence.push("Gemfile".to_string());
        return FrameworkDetection {
            framework: "ruby".to_string(),
            language: "ruby".to_string(),
            evidence,
        };
    }
    if root.join("deno.json").exists() {
        evidence.push("deno.json".to_string());
        return FrameworkDetection {
            framework: "deno".to_string(),
            language: "typescript".to_string(),
            evidence,
        };
    }

    FrameworkDetection {
        framework: "unknown".to_string(),
        language: "unknown".to_string(),
        evidence,
    }
}

fn infer_node_language(root: &Path, package_json_lc: &str) -> String {
    if package_json_lc.contains("\"typescript\"") || root.join("tsconfig.json").exists() {
        "typescript".to_string()
    } else {
        "javascript".to_string()
    }
}

fn file_contains_token(root: &Path, relative_path: &str, token: &str) -> bool {
    fs::read_to_string(root.join(relative_path))
        .map(|content| content.to_ascii_lowercase().contains(token))
        .unwrap_or(false)
}
