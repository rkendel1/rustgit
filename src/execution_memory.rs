use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExecutionMemory {
    pub execution_id: String,
    pub repository_id: String,
    pub commit_sha: String,
    pub fingerprint_hash: String,
    pub generated_plan: String,
    pub chosen_plan: String,
    pub success: bool,
    pub failure_type: Option<String>,
    pub repair: Option<String>,
    pub duration_seconds: Option<u64>,
    pub cost_units: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExecutionPattern {
    pub fingerprint: String,
    pub failure_type: String,
    pub repair: String,
    pub success_rate: f64,
    pub execution_count: u64,
    pub average_duration: f64,
    pub average_cost: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExecutionContextSnapshot {
    pub execution_id: String,
    pub similar_execution_ids: Vec<String>,
    pub retrieved_patterns: Vec<String>,
    pub generated_plan: String,
    pub chosen_plan: String,
}
