use crate::repository_knowledge_graph::{RepositoryFailureRecord, RepositoryKnowledgeGraph};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepositoryQueryContext {
    pub code_context: Vec<String>,
    pub execution_context: Vec<String>,
    pub failure_context: Vec<String>,
    pub recovery_context: Vec<String>,
}

impl RepositoryQueryContext {
    pub fn as_prompt_context(&self) -> String {
        format!(
            "Code Context:\n{}\n\nExecution Context:\n{}\n\nFailure Context:\n{}\n\nRecovery Context:\n{}",
            self.code_context.join("\n"),
            self.execution_context.join("\n"),
            self.failure_context.join("\n"),
            self.recovery_context.join("\n")
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryContextBuilder;

impl RepositoryContextBuilder {
    pub fn build(
        &self,
        repository_root: &Path,
        graph: &RepositoryKnowledgeGraph,
    ) -> RepositoryQueryContext {
        RepositoryQueryContext {
            code_context: code_context(repository_root),
            execution_context: execution_context(graph),
            failure_context: failure_context(&graph.failure_history, &graph.healing_history),
            recovery_context: recovery_context(graph),
        }
    }
}

fn code_context(repository_root: &Path) -> Vec<String> {
    [
        "package.json",
        "requirements.txt",
        "Cargo.toml",
        "Dockerfile",
        "pnpm-workspace.yaml",
        "turbo.json",
    ]
    .iter()
    .map(|name| repository_root.join(name))
    .filter(|path| path.is_file())
    .map(display_path)
    .collect()
}

fn execution_context(graph: &RepositoryKnowledgeGraph) -> Vec<String> {
    graph
        .runtime_history
        .iter()
        .map(|entry| {
            format!(
                "execution={} runtime={} status={} duration_seconds={}",
                entry.execution_id,
                entry.runtime,
                entry.status,
                entry
                    .duration_seconds
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            )
        })
        .collect()
}

fn failure_context(
    failures: &[RepositoryFailureRecord],
    healings: &[crate::EidbHealingAttemptRecord],
) -> Vec<String> {
    let mut lines = failures
        .iter()
        .map(|failure| {
            format!(
                "failure execution={} class={} observed_at={}",
                failure.execution_id, failure.failure_class, failure.observed_at
            )
        })
        .collect::<Vec<_>>();
    lines.extend(healings.iter().map(|healing| {
        format!(
            "repair execution={} strategy={} success={}",
            healing.execution_id, healing.repair_strategy, healing.success
        )
    }));
    lines
}

fn recovery_context(graph: &RepositoryKnowledgeGraph) -> Vec<String> {
    graph
        .temporal_recovery_history
        .iter()
        .map(|entry| {
            format!(
                "last_good_commit={} recovery_success={} path={}",
                entry
                    .last_good_commit
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                entry.recovery_success,
                entry.recovery_path.join(" -> ")
            )
        })
        .collect()
}

fn display_path(path: PathBuf) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        repository_knowledge_graph::{ArchitectureGraph, TemporalRecoveryRecord},
        DependencyGraph, RepositoryFingerprint,
    };

    #[test]
    fn context_builder_includes_code_execution_failure_and_recovery_sections() {
        let builder = RepositoryContextBuilder;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let graph = RepositoryKnowledgeGraph {
            repository_id: uuid::Uuid::new_v4(),
            repository_fingerprint: RepositoryFingerprint::default(),
            execution_history: vec![],
            runtime_history: vec![crate::repository_knowledge_graph::RepositoryRuntimeRecord {
                execution_id: "exec-1".to_string(),
                runtime: "WASM".to_string(),
                status: "success".to_string(),
                duration_seconds: Some(10),
            }],
            failure_history: vec![crate::repository_knowledge_graph::RepositoryFailureRecord {
                execution_id: "exec-2".to_string(),
                failure_class: "failed".to_string(),
                observed_at: 2,
            }],
            healing_history: vec![crate::EidbHealingAttemptRecord {
                repository_id: "repo".to_string(),
                execution_id: "exec-2".to_string(),
                failure_class: "failed".to_string(),
                repair_strategy: "retry".to_string(),
                success: true,
                created_at: 3,
            }],
            temporal_recovery_history: vec![TemporalRecoveryRecord {
                last_good_commit: Some("abc1234".to_string()),
                recovery_path: vec!["def5678".to_string(), "abc1234".to_string()],
                recovery_success: true,
            }],
            dependency_graph: DependencyGraph::default(),
            architecture_graph: ArchitectureGraph::default(),
        };

        let context = builder.build(root, &graph);
        assert!(context.code_context.contains(&"Cargo.toml".to_string()));
        assert!(context
            .execution_context
            .iter()
            .any(|line| line.contains("runtime=WASM")));
        assert!(context
            .failure_context
            .iter()
            .any(|line| line.contains("strategy=retry")));
        assert!(context
            .recovery_context
            .iter()
            .any(|line| line.contains("last_good_commit=abc1234")));
    }
}
