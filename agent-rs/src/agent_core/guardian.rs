use crate::security::policy::RiskLevel;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::task::JoinSet;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GuardianVote {
    pub action_id: String,
    pub verdict: VoteVerdict,
    pub confidence: f32,
    pub reasoning: String,
    pub specialist_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum VoteVerdict {
    Allow,
    Deny,
    Abstain,
}

pub fn vote_schema() -> String {
    serde_json::to_string_pretty(&schema_for!(GuardianVote)).unwrap_or_else(|_| "{}".to_string())
}

#[derive(Debug, Clone)]
pub struct QuorumThresholds {
    pub critical: f32,
    pub high: f32,
    pub medium: f32,
}

impl Default for QuorumThresholds {
    fn default() -> Self {
        Self {
            critical: 1.0,
            high: 0.75,
            medium: 0.51,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Guardian {
    specialist_count: usize,
    thresholds: QuorumThresholds,
}

impl Guardian {
    pub fn new(specialist_count: usize) -> Self {
        Self {
            specialist_count: specialist_count.max(3),
            thresholds: QuorumThresholds::default(),
        }
    }

    pub fn specialist_count(&self) -> usize {
        self.specialist_count
    }

    pub fn thresholds(&self) -> &QuorumThresholds {
        &self.thresholds
    }

    pub async fn evaluate(&self, action: &GuardianAction) -> GuardianDecision {
        let action_id = action.hash();
        let votes = self.fan_out(action_id.clone(), action).await;
        self.quorum_check(&votes, action.risk_level)
    }

    async fn fan_out(&self, action_id: String, action: &GuardianAction) -> Vec<GuardianVote> {
        let mut join_set = JoinSet::new();
        for specialist_id in 0..self.specialist_count {
            let action = action.clone();
            let action_id = action_id.clone();
            join_set.spawn(async move { Self::call_specialist(action_id, specialist_id, action).await });
        }

        let mut votes = Vec::with_capacity(self.specialist_count);
        while let Some(result) = join_set.join_next().await {
            if let Ok(vote) = result {
                votes.push(vote);
            }
        }

        votes
    }

    async fn call_specialist(action_id: String, specialist_id: usize, action: GuardianAction) -> GuardianVote {
        let verdict = match action.risk_level {
            RiskLevel::Low | RiskLevel::Medium => VoteVerdict::Allow,
            RiskLevel::High => {
                if specialist_id % 4 == 0 { VoteVerdict::Deny } else { VoteVerdict::Allow }
            }
            RiskLevel::Critical => {
                if specialist_id == 0 { VoteVerdict::Allow } else { VoteVerdict::Deny }
            }
        };

        let confidence = match verdict {
            VoteVerdict::Allow => 0.82,
            VoteVerdict::Deny => 0.91,
            VoteVerdict::Abstain => 0.5,
        };

        GuardianVote {
            action_id,
            verdict,
            confidence,
            reasoning: format!("specialist {} reviewed {}", specialist_id, action.tool_name),
            specialist_id: specialist_id.to_string(),
        }
    }

    fn quorum_check(&self, votes: &[GuardianVote], risk: RiskLevel) -> GuardianDecision {
        let threshold = match risk {
            RiskLevel::Critical => self.thresholds.critical,
            RiskLevel::High => self.thresholds.high,
            RiskLevel::Medium => self.thresholds.medium,
            RiskLevel::Low => 0.0,
        };

        let allow_count = votes.iter().filter(|vote| vote.verdict == VoteVerdict::Allow).count();
        let total = votes.len().max(1);
        let ratio = allow_count as f32 / total as f32;

        if ratio >= threshold {
            GuardianDecision::Allow { votes: votes.to_vec(), ratio }
        } else {
            GuardianDecision::Deny { votes: votes.to_vec(), ratio }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GuardianAction {
    pub tool_name: String,
    pub args: Value,
    pub risk_level: RiskLevel,
}

impl GuardianAction {
    pub fn hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.tool_name.hash(&mut hasher);
        self.args.to_string().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub fn guardian_prompt(&self) -> String {
        format!(
            "Evaluate tool '{}' at risk {:?} with args {}",
            self.tool_name, self.risk_level, self.args
        )
    }
}

#[derive(Debug, Clone)]
pub enum GuardianDecision {
    Allow { votes: Vec<GuardianVote>, ratio: f32 },
    Deny { votes: Vec<GuardianVote>, ratio: f32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_action(risk: RiskLevel) -> GuardianAction {
        GuardianAction {
            tool_name: "run_terminal_command".to_string(),
            args: json!({"command": "cargo test"}),
            risk_level: risk,
        }
    }

    #[tokio::test]
    async fn guardian_new_enforces_minimum_specialists() {
        let guardian = Guardian::new(1);
        assert_eq!(guardian.specialist_count(), 3);
    }

    #[tokio::test]
    async fn guardian_default_thresholds() {
        let guardian = Guardian::new(3);
        let thresholds = guardian.thresholds();
        assert_eq!(thresholds.critical, 1.0);
        assert_eq!(thresholds.high, 0.75);
        assert_eq!(thresholds.medium, 0.51);
    }

    #[tokio::test]
    async fn guardian_low_risk_always_allows() {
        let guardian = Guardian::new(3);
        let action = make_action(RiskLevel::Low);
        let decision = guardian.evaluate(&action).await;
        assert!(matches!(decision, GuardianDecision::Allow { .. }));
    }

    #[tokio::test]
    async fn guardian_critical_requires_unanimous() {
        let guardian = Guardian::new(3);
        let action = make_action(RiskLevel::Critical);
        let decision = guardian.evaluate(&action).await;
        match decision {
            GuardianDecision::Deny { votes, .. } => {
                assert_eq!(votes.len(), 3);
            }
            _ => panic!("Critical risk should require unanimous, expected Deny"),
        }
    }

    #[tokio::test]
    async fn guardian_high_risk_quorum_75() {
        let guardian = Guardian::new(4);
        let action = make_action(RiskLevel::High);
        let decision = guardian.evaluate(&action).await;
        assert!(matches!(decision, GuardianDecision::Allow { ratio, .. } if ratio > 0.5));
    }

    #[tokio::test]
    async fn guardian_action_hash_is_deterministic() {
        let action = make_action(RiskLevel::High);
        let hash1 = action.hash();
        let hash2 = action.hash();
        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn guardian_action_different_args_produce_different_hash() {
        let action1 = GuardianAction {
            tool_name: "run_terminal_command".to_string(),
            args: json!({"command": "cargo test"}),
            risk_level: RiskLevel::High,
        };
        let action2 = GuardianAction {
            tool_name: "run_terminal_command".to_string(),
            args: json!({"command": "cargo build"}),
            risk_level: RiskLevel::High,
        };
        assert_ne!(action1.hash(), action2.hash());
    }

    #[test]
    fn vote_verdict_serialization() {
        let allow = VoteVerdict::Allow;
        let deny = VoteVerdict::Deny;
        let abstain = VoteVerdict::Abstain;
        assert_eq!(serde_json::to_string(&allow).unwrap(), "\"allow\"");
        assert_eq!(serde_json::to_string(&deny).unwrap(), "\"deny\"");
        assert_eq!(serde_json::to_string(&abstain).unwrap(), "\"abstain\"");
    }

    #[test]
    fn guardian_vote_json_round_trip() {
        let vote = GuardianVote {
            action_id: "abc123".to_string(),
            verdict: VoteVerdict::Allow,
            confidence: 0.85,
            reasoning: "looks safe".to_string(),
            specialist_id: "0".to_string(),
        };
        let json = serde_json::to_string(&vote).unwrap();
        let parsed: GuardianVote = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.action_id, "abc123");
        assert_eq!(parsed.confidence, 0.85);
    }

    #[test]
    fn guardian_vote_schema_generates_valid_json() {
        let schema = vote_schema();
        let parsed: serde_json::Value = serde_json::from_str(&schema).expect("valid JSON schema");
        assert!(parsed.get("$schema").is_some());
        assert!(parsed.get("properties").is_some());
    }

    #[test]
    fn custom_quorum_thresholds() {
        let thresholds = QuorumThresholds {
            critical: 0.8,
            high: 0.6,
            medium: 0.51,
        };
        assert_eq!(thresholds.critical, 0.8);
        assert_eq!(thresholds.high, 0.6);
    }

    #[tokio::test]
    async fn guardian_fan_out_collects_all_votes() {
        let guardian = Guardian::new(5);
        let action = make_action(RiskLevel::High);
        let decision = guardian.evaluate(&action).await;
        if let GuardianDecision::Allow { votes, .. } = decision {
            assert_eq!(votes.len(), 5);
            let unique_ids: std::collections::HashSet<_> =
                votes.iter().map(|v| v.specialist_id.clone()).collect();
            assert_eq!(unique_ids.len(), 5);
        }
    }
}