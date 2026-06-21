use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecutionBlueprint {
    #[serde(rename = "preferredProvider")]
    pub preferred_provider: String,
    pub fallback: Vec<String>,
}

pub fn build_blueprint(runtime: &str) -> ExecutionBlueprint {
    let preferred_provider = match runtime {
        "node" | "bun" | "deno" => "browser-wasm",
        "rust" | "go" => "docker",
        "python" | "java" | "php" | "ruby" => "fly",
        _ => "docker",
    }
    .to_string();

    ExecutionBlueprint {
        preferred_provider,
        fallback: vec![
            "fly".to_string(),
            "docker".to_string(),
            "codespaces".to_string(),
        ],
    }
}
