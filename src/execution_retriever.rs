use crate::execution_memory::{ExecutionMemory, ExecutionPattern};

#[derive(Debug, Clone, Default)]
pub struct ExecutionRetriever {
    pub memories: Vec<ExecutionMemory>,
    pub patterns: Vec<ExecutionPattern>,
}

impl ExecutionRetriever {
    pub fn similar_executions(&self, fingerprint_hash: &str, limit: usize) -> Vec<ExecutionMemory> {
        let mut candidates = self
            .memories
            .iter()
            .filter(|entry| entry.fingerprint_hash == fingerprint_hash)
            .cloned()
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right.success.cmp(&left.success).then_with(|| {
                match (left.duration_seconds, right.duration_seconds) {
                    (Some(left_duration), Some(right_duration)) => {
                        left_duration.cmp(&right_duration)
                    }
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            })
        });
        candidates.into_iter().take(limit).collect()
    }

    pub fn patterns_for_failure(
        &self,
        fingerprint_hash: &str,
        failure_type: &str,
        limit: usize,
    ) -> Vec<ExecutionPattern> {
        let mut candidates = self
            .patterns
            .iter()
            .filter(|entry| {
                entry.fingerprint == fingerprint_hash && entry.failure_type == failure_type
            })
            .cloned()
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .success_rate
                .partial_cmp(&left.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.into_iter().take(limit).collect()
    }
}
