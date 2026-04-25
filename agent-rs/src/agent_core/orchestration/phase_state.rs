use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Discover,
    Discuss,
    Plan,
    Execute,
    Verify,
    Close,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Phase::Discover => "Discover",
            Phase::Discuss => "Discuss",
            Phase::Plan => "Plan",
            Phase::Execute => "Execute",
            Phase::Verify => "Verify",
            Phase::Close => "Close",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseStateMachine {
    current_phase: Phase,
}

impl Default for PhaseStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl PhaseStateMachine {
    pub fn new() -> Self {
        Self {
            current_phase: Phase::Discover,
        }
    }

    pub fn current_phase(&self) -> Phase {
        self.current_phase
    }

    pub fn set_phase(&mut self, phase: Phase) {
        self.current_phase = phase;
    }

    pub fn can_transition_to(&self, next_phase: Phase) -> bool {
        match (self.current_phase, next_phase) {
            (Phase::Discover, Phase::Discuss) => true,
            (Phase::Discuss, Phase::Plan) => true,
            (Phase::Plan, Phase::Execute) => true,
            (Phase::Execute, Phase::Verify) => true,
            (Phase::Verify, Phase::Execute) => true, // Retry
            (Phase::Verify, Phase::Plan) => true,    // Re-plan on failure
            (Phase::Verify, Phase::Close) => true,
            (Phase::Close, _) => false,              // Terminal state
            _ => false,
        }
    }

    pub fn transition_to(&mut self, next_phase: Phase) -> Result<(), String> {
        if self.can_transition_to(next_phase) {
            self.current_phase = next_phase;
            Ok(())
        } else {
            Err(format!(
                "Invalid transition from {} to {}",
                self.current_phase, next_phase
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        let mut sm = PhaseStateMachine::new();
        assert_eq!(sm.current_phase(), Phase::Discover);

        assert!(sm.transition_to(Phase::Discuss).is_ok());
        assert_eq!(sm.current_phase(), Phase::Discuss);

        assert!(sm.transition_to(Phase::Plan).is_ok());
        assert_eq!(sm.current_phase(), Phase::Plan);

        assert!(sm.transition_to(Phase::Execute).is_ok());
        assert_eq!(sm.current_phase(), Phase::Execute);

        assert!(sm.transition_to(Phase::Verify).is_ok());
        assert_eq!(sm.current_phase(), Phase::Verify);

        assert!(sm.transition_to(Phase::Close).is_ok());
        assert_eq!(sm.current_phase(), Phase::Close);
    }

    #[test]
    fn test_invalid_transitions() {
        let mut sm = PhaseStateMachine::new();
        assert_eq!(sm.current_phase(), Phase::Discover);

        assert!(sm.transition_to(Phase::Execute).is_err());
        assert_eq!(sm.current_phase(), Phase::Discover);

        assert!(sm.transition_to(Phase::Plan).is_err());
        assert!(sm.transition_to(Phase::Verify).is_err());
        assert!(sm.transition_to(Phase::Close).is_err());
    }

    #[test]
    fn test_verify_retry_and_replan() {
        let mut sm = PhaseStateMachine::new();
        sm.set_phase(Phase::Verify);

        // Verify can transition back to Execute (Retry)
        assert!(sm.transition_to(Phase::Execute).is_ok());
        assert_eq!(sm.current_phase(), Phase::Execute);

        sm.set_phase(Phase::Verify);
        // Verify can transition back to Plan (Replan)
        assert!(sm.transition_to(Phase::Plan).is_ok());
        assert_eq!(sm.current_phase(), Phase::Plan);
    }

    #[test]
    fn test_terminal_state() {
        let mut sm = PhaseStateMachine::new();
        sm.set_phase(Phase::Close);

        assert!(sm.transition_to(Phase::Discover).is_err());
        assert!(sm.transition_to(Phase::Discuss).is_err());
        assert!(sm.transition_to(Phase::Plan).is_err());
        assert!(sm.transition_to(Phase::Execute).is_err());
        assert!(sm.transition_to(Phase::Verify).is_err());
    }
}
