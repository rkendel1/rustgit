use crate::execution_memory::{ExecutionMemory, ExecutionPattern};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OptimizedExecutionPlan {
    pub generated_plan: String,
    pub chosen_plan: String,
    pub reused_execution_ids: Vec<String>,
    pub applied_repairs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionOptimizer;

impl ExecutionOptimizer {
    pub fn optimize(
        &self,
        generated_plan: &str,
        similar_executions: &[ExecutionMemory],
        patterns: &[ExecutionPattern],
    ) -> OptimizedExecutionPlan {
        let chosen_plan = similar_executions
            .iter()
            .find(|entry| entry.success && !entry.chosen_plan.trim().is_empty())
            .map(|entry| entry.chosen_plan.clone())
            .unwrap_or_else(|| generated_plan.to_string());
        let repairs = patterns
            .iter()
            .map(|entry| entry.repair.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        OptimizedExecutionPlan {
            generated_plan: generated_plan.to_string(),
            chosen_plan,
            reused_execution_ids: similar_executions
                .iter()
                .map(|entry| entry.execution_id.clone())
                .collect(),
            applied_repairs: repairs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutionOptimizer;
    use crate::execution_memory::{ExecutionMemory, ExecutionPattern};

    #[test]
    fn optimizer_prefers_successful_similar_plan() {
        let optimizer = ExecutionOptimizer;
        let plan = optimizer.optimize(
            "npm install && npm run build",
            &[ExecutionMemory {
                execution_id: "exec-1".to_string(),
                repository_id: "repo-1".to_string(),
                commit_sha: "abc".to_string(),
                fingerprint_hash: "fp".to_string(),
                generated_plan: "npm install".to_string(),
                chosen_plan: "pnpm install && pnpm build".to_string(),
                success: true,
                failure_type: None,
                repair: None,
                duration_seconds: Some(12),
                cost_units: Some(1.2),
            }],
            &[ExecutionPattern {
                fingerprint: "fp".to_string(),
                failure_type: "WrongPackageManager".to_string(),
                repair: "switch-pnpm".to_string(),
                success_rate: 0.95,
                execution_count: 10,
                average_duration: 12.0,
                average_cost: 1.2,
            }],
        );
        assert_eq!(plan.chosen_plan, "pnpm install && pnpm build");
        assert_eq!(plan.reused_execution_ids, vec!["exec-1".to_string()]);
        assert_eq!(plan.applied_repairs, vec!["switch-pnpm".to_string()]);
    }
}
