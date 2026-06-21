use crate::execution_memory::{ExecutionContextSnapshot, ExecutionMemory, ExecutionPattern};
use crate::execution_optimizer::{ExecutionOptimizer, OptimizedExecutionPlan};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionContextBuilder {
    optimizer: ExecutionOptimizer,
}

impl ExecutionContextBuilder {
    pub fn build(
        &self,
        execution_id: &str,
        generated_plan: &str,
        similar_executions: &[ExecutionMemory],
        patterns: &[ExecutionPattern],
    ) -> (ExecutionContextSnapshot, OptimizedExecutionPlan) {
        let optimized = self
            .optimizer
            .optimize(generated_plan, similar_executions, patterns);
        (
            ExecutionContextSnapshot {
                execution_id: execution_id.to_string(),
                similar_execution_ids: optimized.reused_execution_ids.clone(),
                retrieved_patterns: patterns.iter().map(|entry| entry.repair.clone()).collect(),
                generated_plan: generated_plan.to_string(),
                chosen_plan: optimized.chosen_plan.clone(),
            },
            optimized,
        )
    }
}
