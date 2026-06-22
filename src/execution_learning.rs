use crate::execution_memory::ExecutionPattern;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionLearningEngine;

impl ExecutionLearningEngine {
    pub fn classify_failure(status: &str) -> String {
        let normalized = status.to_ascii_lowercase();
        if normalized.contains("pnpm")
            || normalized == "npm"
            || normalized.starts_with("npm ")
            || normalized.ends_with(" npm")
            || normalized.contains(" npm ")
        {
            "WrongPackageManager".to_string()
        } else if normalized.contains("timeout") {
            "Timeout".to_string()
        } else if normalized.contains("memory") {
            "ResourceExhaustion".to_string()
        } else if normalized.contains("success") {
            "None".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    pub fn learn_pattern(
        patterns: &mut Vec<ExecutionPattern>,
        fingerprint: &str,
        failure_type: &str,
        repair: &str,
        success: bool,
        duration_seconds: f64,
        cost_units: f64,
    ) {
        if let Some(existing) = patterns.iter_mut().find(|entry| {
            entry.fingerprint == fingerprint
                && entry.failure_type == failure_type
                && entry.repair == repair
        }) {
            let current = existing.execution_count as f64;
            let next = current + 1.0;
            existing.execution_count += 1;
            existing.success_rate =
                ((existing.success_rate * current) + if success { 1.0 } else { 0.0 }) / next;
            existing.average_duration =
                ((existing.average_duration * current) + duration_seconds) / next;
            existing.average_cost = ((existing.average_cost * current) + cost_units) / next;
            return;
        }
        patterns.push(ExecutionPattern {
            fingerprint: fingerprint.to_string(),
            failure_type: failure_type.to_string(),
            repair: repair.to_string(),
            success_rate: if success { 1.0 } else { 0.0 },
            execution_count: 1,
            average_duration: duration_seconds,
            average_cost: cost_units,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutionLearningEngine;
    use crate::execution_memory::ExecutionPattern;

    #[test]
    fn learning_engine_updates_existing_pattern_aggregate() {
        let mut patterns = vec![ExecutionPattern {
            fingerprint: "fp".to_string(),
            failure_type: "WrongPackageManager".to_string(),
            repair: "switch-pnpm".to_string(),
            success_rate: 1.0,
            execution_count: 1,
            average_duration: 10.0,
            average_cost: 1.0,
        }];
        ExecutionLearningEngine::learn_pattern(
            &mut patterns,
            "fp",
            "WrongPackageManager",
            "switch-pnpm",
            false,
            20.0,
            3.0,
        );
        let learned = patterns.first().expect("pattern should exist");
        assert_eq!(learned.execution_count, 2);
        assert_eq!(learned.success_rate, 0.5);
        assert_eq!(learned.average_duration, 15.0);
        assert_eq!(learned.average_cost, 2.0);
    }
}
