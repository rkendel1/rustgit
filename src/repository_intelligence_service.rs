use crate::repository_context_builder::{RepositoryContextBuilder, RepositoryQueryContext};
use crate::repository_embeddings::{RepositoryEmbedding, RepositoryEmbeddingPipeline};
use crate::repository_knowledge_graph::{RepositoryFailureRecord, RepositoryKnowledgeGraph};
use crate::{
    ExecutionGraph, ExecutionIntelligenceReadStore, FailureSignal, PersistenceResult,
    RepairStrategy, RepositoryFingerprint,
};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepositoryEvidence {
    pub evidence_type: String,
    pub reference_id: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RepositoryAnswer {
    pub answer: String,
    pub confidence: f32,
    pub evidence: Vec<RepositoryEvidence>,
    pub related_executions: Vec<String>,
    pub related_failures: Vec<String>,
    pub related_healings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepairPlan {
    pub reason: String,
    pub strategy: RepairStrategy,
}

#[allow(async_fn_in_trait)]
pub trait RepairKnowledgeProvider {
    async fn recommend_repair(&self, repository_id: Uuid, failure: FailureSignal) -> RepairPlan;
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryIntelligenceService {
    context_builder: RepositoryContextBuilder,
    embedding_pipeline: RepositoryEmbeddingPipeline,
}

impl RepositoryIntelligenceService {
    pub fn new(embedding_pipeline: RepositoryEmbeddingPipeline) -> Self {
        Self {
            context_builder: RepositoryContextBuilder,
            embedding_pipeline,
        }
    }

    pub async fn ask_repository(
        &self,
        repository_id: Uuid,
        question: String,
        repository_root: &Path,
        repository_fingerprint: RepositoryFingerprint,
        execution_graph: ExecutionGraph,
        store: &impl ExecutionIntelligenceReadStore,
    ) -> PersistenceResult<RepositoryAnswer> {
        let repository_id_text = repository_id.to_string();
        let graph = RepositoryKnowledgeGraph::from_store(
            repository_id_text.as_str(),
            repository_fingerprint,
            &execution_graph,
            store,
        )?;
        let context = self.context_builder.build(repository_root, &graph);
        let embeddings = self
            .embedding_pipeline
            .build_embeddings(repository_id_text.as_str(), &context)
            .await
            .map_err(|err| {
                crate::ExecutionIntelligencePersistenceError::Serialization(err.to_string())
            })?;
        Ok(answer_question(question, &graph, &context, &embeddings))
    }

    pub fn answer_repository_question(
        &self,
        question: &str,
        graph: &RepositoryKnowledgeGraph,
        repository_root: &Path,
    ) -> RepositoryAnswer {
        let context = self.context_builder.build(repository_root, graph);
        answer_question(question.to_string(), graph, &context, &[])
    }
}

fn answer_question(
    question: String,
    graph: &RepositoryKnowledgeGraph,
    context: &RepositoryQueryContext,
    embeddings: &[RepositoryEmbedding],
) -> RepositoryAnswer {
    let question_lower = question.to_ascii_lowercase();
    let successful_execution = graph
        .execution_history
        .iter()
        .rev()
        .find(|execution| execution.status.eq_ignore_ascii_case("success"));
    let latest_execution = graph.execution_history.last();
    let latest_failure = graph.failure_history.last();
    let latest_healing = graph.healing_history.last();
    let best_runtime = best_runtime(graph);
    let answer = if question_lower.contains("can this run")
        || question_lower.contains("can this repository run")
    {
        if successful_execution.is_some() {
            "Yes. This repository has prior successful executions and can run with the previously successful runtime.".to_string()
        } else {
            "No successful execution has been recorded yet. Start with the latest known runtime and healing history to establish a first green run.".to_string()
        }
    } else if question_lower.contains("why") && question_lower.contains("fail") {
        if let Some(failure) = latest_failure {
            format!(
                "The latest failure class was {} on execution {}. The most recent repair strategy was {}.",
                failure.failure_class,
                failure.execution_id,
                latest_healing
                    .map(|entry| entry.repair_strategy.clone())
                    .unwrap_or_else(|| "none".to_string())
            )
        } else {
            "No concrete failure record exists yet in execution-aware history.".to_string()
        }
    } else if question_lower.contains("runtime") && question_lower.contains("best") {
        match best_runtime {
            Some(runtime) => {
                format!("The best observed runtime is {runtime} based on success history.")
            }
            None => "No runtime performance history is available yet.".to_string(),
        }
    } else if question_lower.contains("heal") || question_lower.contains("repair") {
        if let Some(healing) = latest_healing {
            format!(
                "Recommended repair starts with '{}' for failure class '{}', based on prior healing history.",
                healing.repair_strategy, healing.failure_class
            )
        } else {
            "No prior repair history exists; start with failure classification and targeted dependency/runtime repair.".to_string()
        }
    } else {
        format!(
            "Execution-aware summary: {} executions, {} failures, {} healings, {} recovery snapshots.",
            graph.execution_history.len(),
            graph.failure_history.len(),
            graph.healing_history.len(),
            graph.temporal_recovery_history.len()
        )
    };
    let evidence = evidence(
        context,
        latest_execution.map(|entry| entry.execution_id.clone()),
        latest_failure.cloned(),
        latest_healing.map(|entry| entry.repair_strategy.clone()),
    );
    let retrieval_bonus = if !embeddings.is_empty() { 0.05 } else { 0.0 };
    let confidence = ((evidence.len() as f32 / 8.0) + retrieval_bonus).min(0.99);
    RepositoryAnswer {
        answer,
        confidence,
        related_executions: graph
            .execution_history
            .iter()
            .rev()
            .take(3)
            .map(|execution| execution.execution_id.clone())
            .collect(),
        related_failures: graph
            .failure_history
            .iter()
            .rev()
            .take(3)
            .map(|failure| failure.failure_class.clone())
            .collect(),
        related_healings: graph
            .healing_history
            .iter()
            .rev()
            .take(3)
            .map(|healing| healing.repair_strategy.clone())
            .collect(),
        evidence,
    }
}

fn evidence(
    context: &RepositoryQueryContext,
    execution_id: Option<String>,
    failure: Option<RepositoryFailureRecord>,
    repair_strategy: Option<String>,
) -> Vec<RepositoryEvidence> {
    let mut evidence = Vec::with_capacity(4);
    evidence.push(RepositoryEvidence {
        evidence_type: "file".to_string(),
        reference_id: context
            .code_context
            .first()
            .cloned()
            .unwrap_or_else(|| "repository-root".to_string()),
        detail: "Repository code/config context".to_string(),
    });
    evidence.push(RepositoryEvidence {
        evidence_type: "execution".to_string(),
        reference_id: execution_id.unwrap_or_else(|| "none".to_string()),
        detail: "Execution history signal".to_string(),
    });
    evidence.push(RepositoryEvidence {
        evidence_type: "failure".to_string(),
        reference_id: failure
            .as_ref()
            .map(|entry| entry.execution_id.clone())
            .unwrap_or_else(|| "none".to_string()),
        detail: failure
            .map(|entry| entry.failure_class)
            .unwrap_or_else(|| "No recorded failure".to_string()),
    });
    evidence.push(RepositoryEvidence {
        evidence_type: "repair".to_string(),
        reference_id: repair_strategy
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        detail: repair_strategy.unwrap_or_else(|| "No recorded repair".to_string()),
    });
    evidence
}

fn best_runtime(graph: &RepositoryKnowledgeGraph) -> Option<String> {
    let mut stats = HashMap::<String, (usize, usize)>::new();
    for execution in &graph.execution_history {
        let entry = stats
            .entry(execution.execution_tier.clone())
            .or_insert((0, 0));
        entry.0 += 1;
        if execution.status.eq_ignore_ascii_case("success") {
            entry.1 += 1;
        }
    }
    stats
        .into_iter()
        .max_by(|(_, left), (_, right)| {
            let left_ratio = left.1 as f32 / left.0 as f32;
            let right_ratio = right.1 as f32 / right.0 as f32;
            left_ratio
                .partial_cmp(&right_ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(runtime, _)| runtime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        repository_knowledge_graph::{
            ArchitectureGraph, RepositoryFailureRecord, RepositoryRuntimeRecord,
            TemporalRecoveryRecord,
        },
        DependencyGraph,
    };

    #[test]
    fn repository_answer_links_evidence_categories_and_confidence() {
        let graph = RepositoryKnowledgeGraph {
            repository_id: Uuid::new_v4(),
            repository_fingerprint: RepositoryFingerprint::default(),
            execution_history: vec![crate::EidbExecutionRecord {
                execution_id: "exec-1".to_string(),
                org_id: None,
                user_id: None,
                anon_user_id: Some("anon".to_string()),
                workspace_id: "ws-1".to_string(),
                repository_id: "repo".to_string(),
                commit_hash: "aaa".to_string(),
                started_at: 1,
                completed_at: Some(2),
                status: "success".to_string(),
                execution_tier: "WASM".to_string(),
            }],
            runtime_history: vec![RepositoryRuntimeRecord {
                execution_id: "exec-1".to_string(),
                runtime: "WASM".to_string(),
                status: "success".to_string(),
                duration_seconds: Some(1),
            }],
            failure_history: vec![RepositoryFailureRecord {
                execution_id: "exec-0".to_string(),
                failure_class: "WrongPackageManager".to_string(),
                observed_at: 0,
            }],
            healing_history: vec![crate::EidbHealingAttemptRecord {
                repository_id: "repo".to_string(),
                execution_id: "exec-0".to_string(),
                failure_class: "WrongPackageManager".to_string(),
                repair_strategy: "switch-pnpm".to_string(),
                success: true,
                created_at: 0,
            }],
            temporal_recovery_history: vec![TemporalRecoveryRecord {
                last_good_commit: Some("aaa".to_string()),
                recovery_path: vec!["bbb".to_string(), "aaa".to_string()],
                recovery_success: true,
            }],
            dependency_graph: DependencyGraph::default(),
            architecture_graph: ArchitectureGraph::default(),
        };
        let service = RepositoryIntelligenceService::default();
        let answer = service.answer_repository_question(
            "Why did the last build fail?",
            &graph,
            Path::new(env!("CARGO_MANIFEST_DIR")),
        );

        assert_eq!(answer.evidence.len(), 4);
        assert!(answer
            .evidence
            .iter()
            .any(|entry| entry.evidence_type == "file"));
        assert!(answer
            .evidence
            .iter()
            .any(|entry| entry.evidence_type == "execution"));
        assert!(answer
            .evidence
            .iter()
            .any(|entry| entry.evidence_type == "failure"));
        assert!(answer
            .evidence
            .iter()
            .any(|entry| entry.evidence_type == "repair"));
        assert!(answer.confidence > 0.0);
    }
}
