use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    Retry,
    Decompose,
    Prune,
    Escalate,
}

#[derive(Debug, Clone)]
pub struct RecoveryDecisionMatrix {
    pub max_retries: u8,
    pub max_decompose: u8,
}

impl Default for RecoveryDecisionMatrix {
    fn default() -> Self {
        Self {
            max_retries: 2,
            max_decompose: 2,
        }
    }
}

impl RecoveryDecisionMatrix {
    pub fn decide(
        &self,
        retries_attempted: u8,
        decompose_attempted: u8,
        is_optional: bool,
    ) -> RecoveryAction {
        if retries_attempted < self.max_retries {
            RecoveryAction::Retry
        } else if decompose_attempted < self.max_decompose {
            RecoveryAction::Decompose
        } else if is_optional {
            RecoveryAction::Prune
        } else {
            RecoveryAction::Escalate
        }
    }
}

pub struct LoopDetector {
    signatures: HashMap<u64, u8>,
    max_loops_per_signature: u8,
}

impl LoopDetector {
    pub fn new(max_loops_per_signature: u8) -> Self {
        Self {
            signatures: HashMap::new(),
            max_loops_per_signature,
        }
    }

    pub fn calculate_signature(tool: &str, args_hash: u64, outcome_snippet: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        tool.hash(&mut hasher);
        args_hash.hash(&mut hasher);
        outcome_snippet.hash(&mut hasher);
        hasher.finish()
    }

    pub fn register_failure(&mut self, signature: u64) -> RecoveryAction {
        let count = self.signatures.entry(signature).or_insert(0);
        *count += 1;

        if *count > self.max_loops_per_signature {
            RecoveryAction::Escalate
        } else {
            RecoveryAction::Retry // Just an indicator, the real decision happens in RecoveryDecisionMatrix
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_decision_matrix() {
        let matrix = RecoveryDecisionMatrix::default(); // max_retries: 2, max_decompose: 2

        // Retry limit enforcement
        assert_eq!(matrix.decide(0, 0, false), RecoveryAction::Retry);
        assert_eq!(matrix.decide(1, 0, false), RecoveryAction::Retry);
        
        // Decompose triggering after RETRY exhaustion
        assert_eq!(matrix.decide(2, 0, false), RecoveryAction::Decompose);
        assert_eq!(matrix.decide(2, 1, false), RecoveryAction::Decompose);

        // Prune for optional steps
        assert_eq!(matrix.decide(2, 2, true), RecoveryAction::Prune);

        // Escalate for required steps after retries and decomposes
        assert_eq!(matrix.decide(2, 2, false), RecoveryAction::Escalate);
        assert_eq!(matrix.decide(3, 3, false), RecoveryAction::Escalate);
    }

    #[test]
    fn test_loop_detector() {
        let mut detector = LoopDetector::new(2);
        let sig = LoopDetector::calculate_signature("sysinfo", 12345, "permission denied");

        assert_eq!(detector.register_failure(sig), RecoveryAction::Retry); // count 1
        assert_eq!(detector.register_failure(sig), RecoveryAction::Retry); // count 2
        assert_eq!(detector.register_failure(sig), RecoveryAction::Escalate); // count 3 -> Escalate
    }
}
