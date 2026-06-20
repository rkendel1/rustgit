use crate::repository_context_builder::RepositoryQueryContext;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt::{Display, Formatter};

const DEFAULT_EMBEDDING_DIMENSIONS: usize = 1536;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryEmbedding {
    pub embedding_id: String,
    pub repository_id: String,
    pub artifact_kind: String,
    pub artifact_id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: Value,
}

#[derive(Debug)]
pub enum RepositoryEmbeddingError {
    Http(reqwest::Error),
    InvalidResponse,
}

impl Display for RepositoryEmbeddingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(err) => write!(f, "repository embedding http error: {err}"),
            Self::InvalidResponse => write!(f, "repository embedding invalid response payload"),
        }
    }
}

impl std::error::Error for RepositoryEmbeddingError {}

impl From<reqwest::Error> for RepositoryEmbeddingError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiEmbeddingClient {
    api_key: Option<String>,
    model: String,
    dimensions: usize,
    http: reqwest::Client,
}

impl Default for OpenAiEmbeddingClient {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            model: std::env::var("OPENAI_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string()),
            dimensions: std::env::var("OPENAI_EMBEDDING_DIMENSIONS")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(DEFAULT_EMBEDDING_DIMENSIONS),
            http: reqwest::Client::new(),
        }
    }
}

impl OpenAiEmbeddingClient {
    pub async fn embed_text(&self, input: &str) -> Result<Vec<f32>, RepositoryEmbeddingError> {
        if let Some(api_key) = &self.api_key {
            let payload = json!({
                "model": self.model,
                "input": input,
            });
            let response = self
                .http
                .post("https://api.openai.com/v1/embeddings")
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await?;
            let value = response.json::<Value>().await?;
            let embedding = value
                .get("data")
                .and_then(Value::as_array)
                .and_then(|data| data.first())
                .and_then(|row| row.get("embedding"))
                .and_then(Value::as_array)
                .ok_or(RepositoryEmbeddingError::InvalidResponse)?
                .iter()
                .filter_map(Value::as_f64)
                .map(|value| value as f32)
                .collect::<Vec<_>>();
            if !embedding.is_empty() {
                return Ok(embedding);
            }
        }
        Ok(deterministic_embedding(input, self.dimensions))
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryEmbeddingPipeline {
    openai: OpenAiEmbeddingClient,
}

impl RepositoryEmbeddingPipeline {
    pub fn new(openai: OpenAiEmbeddingClient) -> Self {
        Self { openai }
    }

    pub async fn build_embeddings(
        &self,
        repository_id: &str,
        context: &RepositoryQueryContext,
    ) -> Result<Vec<RepositoryEmbedding>, RepositoryEmbeddingError> {
        let artifacts = vec![
            ("code", "code_context", context.code_context.join("\n")),
            (
                "operational",
                "execution_summary",
                context.execution_context.join("\n"),
            ),
            (
                "operational",
                "failure_summary",
                context.failure_context.join("\n"),
            ),
            (
                "operational",
                "recovery_summary",
                context.recovery_context.join("\n"),
            ),
        ];

        let mut embeddings = Vec::with_capacity(artifacts.len());
        for (index, (artifact_kind, artifact_id, content)) in artifacts.into_iter().enumerate() {
            let embedding = self.openai.embed_text(&content).await?;
            embeddings.push(RepositoryEmbedding {
                embedding_id: format!("{repository_id}-embedding-{index}"),
                repository_id: repository_id.to_string(),
                artifact_kind: artifact_kind.to_string(),
                artifact_id: artifact_id.to_string(),
                content,
                embedding,
                metadata: json!({
                    "source": "repository-intelligence",
                    "model": "openai",
                    "dimensions": self.openai.dimensions(),
                }),
            });
        }
        Ok(embeddings)
    }

    pub fn embedding_dimensions(&self) -> usize {
        self.openai.dimensions()
    }

    pub fn embedding_literal(embedding: &[f32]) -> String {
        let values = embedding
            .iter()
            .map(|value| format!("{value:.8}"))
            .collect::<Vec<_>>()
            .join(",");
        format!("[{values}]")
    }
}

fn deterministic_embedding(input: &str, dimensions: usize) -> Vec<f32> {
    let mut state = input
        .bytes()
        .fold(0x9E3779B97F4A7C15_u64, |acc, byte| acc ^ ((byte as u64) + 0x9E37));
    (0..dimensions)
        .map(|index| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(index as u64 + 1442695040888963407);
            ((state >> 11) as f32 / u32::MAX as f32) * 2.0 - 1.0
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn embedding_pipeline_generates_operational_and_code_embeddings() {
        let pipeline = RepositoryEmbeddingPipeline::default();
        let embeddings = pipeline
            .build_embeddings(
                "repo-1",
                &RepositoryQueryContext {
                    code_context: vec!["Cargo.toml".to_string()],
                    execution_context: vec!["execution=exec-1".to_string()],
                    failure_context: vec!["failure=none".to_string()],
                    recovery_context: vec!["last_good_commit=abc".to_string()],
                },
            )
            .await
            .expect("embeddings should generate");

        assert_eq!(embeddings.len(), 4);
        assert!(embeddings.iter().any(|entry| entry.artifact_kind == "operational"));
        assert_eq!(embeddings[0].embedding.len(), pipeline.embedding_dimensions());
        assert!(RepositoryEmbeddingPipeline::embedding_literal(&embeddings[0].embedding).starts_with('['));
    }
}
