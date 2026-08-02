//! Shared evaluation result type for Phase 10 evaluation scenarios.

use serde::{Serialize, Deserialize};

/// The result of a single evaluation scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub scenario: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub notes: String,
}

impl EvalResult {
    pub fn pass(scenario: &str, duration_ms: u64) -> Self {
        Self { scenario: scenario.to_string(), passed: true, duration_ms, notes: String::new() }
    }
    pub fn fail(scenario: &str, duration_ms: u64, reason: &str) -> Self {
        Self { scenario: scenario.to_string(), passed: false, duration_ms, notes: reason.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eval_result_pass_sets_passed_true() {
        let r = EvalResult::pass("test", 10);
        assert!(r.passed);
        assert_eq!(r.scenario, "test");
    }
    #[test]
    fn eval_result_fail_sets_passed_false() {
        let r = EvalResult::fail("test", 10, "error");
        assert!(!r.passed);
        assert_eq!(r.notes, "error");
    }
}
