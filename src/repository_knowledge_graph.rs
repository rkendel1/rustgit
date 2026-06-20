use crate::{
    eidb_execution_status_is_success, DependencyGraph, EidbExecutionRecord, EidbHealingAttemptRecord,
    ExecutionGraph, ExecutionIntelligenceReadStore, PersistenceResult, RepositoryFingerprint,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepositoryRuntimeRecord {
    pub execution_id: String,
    pub runtime: String,
    pub status: String,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepositoryFailureRecord {
    pub execution_id: String,
    pub failure_class: String,
    pub observed_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemporalRecoveryRecord {
    pub last_good_commit: Option<String>,
    pub recovery_path: Vec<String>,
    pub recovery_success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchitectureNode {
    pub id: String,
    pub node_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchitectureEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct ArchitectureGraph {
    pub nodes: Vec<ArchitectureNode>,
    pub edges: Vec<ArchitectureEdge>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryKnowledgeGraph {
    pub repository_id: Uuid,
    pub repository_fingerprint: RepositoryFingerprint,
    pub execution_history: Vec<EidbExecutionRecord>,
    pub runtime_history: Vec<RepositoryRuntimeRecord>,
    pub failure_history: Vec<RepositoryFailureRecord>,
    pub healing_history: Vec<EidbHealingAttemptRecord>,
    pub temporal_recovery_history: Vec<TemporalRecoveryRecord>,
    pub dependency_graph: DependencyGraph,
    pub architecture_graph: ArchitectureGraph,
}

impl RepositoryKnowledgeGraph {
    pub fn from_store(
        repository_id: &str,
        repository_fingerprint: RepositoryFingerprint,
        execution_graph: &ExecutionGraph,
        store: &impl ExecutionIntelligenceReadStore,
    ) -> PersistenceResult<Self> {
        let execution_history = store.executions_for_repository(repository_id)?;
        let healing_history = store.healing_attempts_for_repository(repository_id)?;
        let last_good_commit = store.last_good_commit_for_repository(repository_id)?;
        let runtime_history = runtime_history(&execution_history);
        let failure_history = failure_history(&execution_history, &healing_history);
        let temporal_recovery_history = vec![TemporalRecoveryRecord {
            last_good_commit,
            recovery_path: execution_history
                .iter()
                .map(|execution| execution.commit_hash.clone())
                .collect(),
            recovery_success: execution_history
                .iter()
                .any(|execution| eidb_execution_status_is_success(&execution.status)),
        }];
        let architecture_graph = ArchitectureGraph {
            nodes: execution_graph
                .nodes
                .iter()
                .map(|node| ArchitectureNode {
                    id: node.id.clone(),
                    node_type: format!("{:?}", node.node_type),
                })
                .collect(),
            edges: execution_graph
                .edges
                .iter()
                .map(|edge| ArchitectureEdge {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                })
                .collect(),
        };
        Ok(Self {
            repository_id: Uuid::parse_str(repository_id).unwrap_or_else(|_| Uuid::new_v4()),
            dependency_graph: repository_fingerprint.dependency_graph.clone(),
            repository_fingerprint,
            execution_history,
            runtime_history,
            failure_history,
            healing_history,
            temporal_recovery_history,
            architecture_graph,
        })
    }
}

fn runtime_history(executions: &[EidbExecutionRecord]) -> Vec<RepositoryRuntimeRecord> {
    executions
        .iter()
        .map(|execution| RepositoryRuntimeRecord {
            execution_id: execution.execution_id.clone(),
            runtime: execution.execution_tier.clone(),
            status: execution.status.clone(),
            duration_seconds: execution
                .completed_at
                .map(|completed_at| completed_at.saturating_sub(execution.started_at)),
        })
        .collect()
}

fn failure_history(
    executions: &[EidbExecutionRecord],
    healings: &[EidbHealingAttemptRecord],
) -> Vec<RepositoryFailureRecord> {
    let mut failures = executions
        .iter()
        .filter(|execution| !eidb_execution_status_is_success(&execution.status))
        .map(|execution| RepositoryFailureRecord {
            execution_id: execution.execution_id.clone(),
            failure_class: execution.status.clone(),
            observed_at: execution.completed_at.unwrap_or(execution.started_at),
        })
        .collect::<Vec<_>>();
    failures.extend(
        healings
            .iter()
            .filter(|attempt| !attempt.success)
            .map(|attempt| RepositoryFailureRecord {
                execution_id: attempt.execution_id.clone(),
                failure_class: attempt.failure_class.clone(),
                observed_at: attempt.created_at,
            }),
    );
    failures
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EidbExecutionRecord, EidbHealingAttemptRecord, EidbRepositoryRecord, ExecutionIntelligenceDatabase,
    };

    #[test]
    fn repository_graph_aggregates_execution_and_recovery_history() {
        let mut database = ExecutionIntelligenceDatabase::default();
        database.repositories.insert(
            "repo-knowledge".to_string(),
            EidbRepositoryRecord {
                repo_id: "repo-knowledge".to_string(),
                repo_url: "https://github.com/octocat/hello-world".to_string(),
                default_branch: "main".to_string(),
                first_seen: 1,
                last_seen: 2,
            },
        );
        database.record_execution(EidbExecutionRecord {
            execution_id: "exec-success".to_string(),
            org_id: None,
            user_id: None,
            anon_user_id: Some("anon".to_string()),
            workspace_id: "ws-1".to_string(),
            repository_id: "repo-knowledge".to_string(),
            commit_hash: "aaa".to_string(),
            started_at: 10,
            completed_at: Some(20),
            status: "success".to_string(),
            execution_tier: "WASM".to_string(),
        });
        database.record_execution(EidbExecutionRecord {
            execution_id: "exec-failure".to_string(),
            org_id: None,
            user_id: None,
            anon_user_id: Some("anon".to_string()),
            workspace_id: "ws-2".to_string(),
            repository_id: "repo-knowledge".to_string(),
            commit_hash: "bbb".to_string(),
            started_at: 21,
            completed_at: Some(25),
            status: "failed".to_string(),
            execution_tier: "CLOUD".to_string(),
        });
        database.record_healing_attempt(EidbHealingAttemptRecord {
            repository_id: "repo-knowledge".to_string(),
            execution_id: "exec-failure".to_string(),
            failure_class: "WrongPackageManager".to_string(),
            repair_strategy: "switch-pnpm".to_string(),
            success: true,
            created_at: 26,
        });

        let graph = RepositoryKnowledgeGraph::from_store(
            "repo-knowledge",
            RepositoryFingerprint {
                repo_id: "repo-knowledge".to_string(),
                ..RepositoryFingerprint::default()
            },
            &ExecutionGraph::default(),
            &database,
        )
        .expect("knowledge graph should build");

        assert_eq!(graph.execution_history.len(), 2);
        assert_eq!(graph.runtime_history.len(), 2);
        assert_eq!(graph.failure_history.len(), 1);
        assert_eq!(graph.healing_history.len(), 1);
        assert_eq!(graph.temporal_recovery_history.len(), 1);
    }
}
