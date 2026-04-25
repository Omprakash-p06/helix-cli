use serde::{Deserialize, Serialize};
use crate::audit::{AuditStore, hash_payload};
use crate::agent_core::diagnostics::logs::{get_system_logs};
use crate::agent_core::diagnostics::system::{SystemProvider};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagnosticState {
    Observe,
    Hypothesize,
    Test,
    Synthesize,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub description: String,
    pub status: HypothesisStatus,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HypothesisStatus {
    Pending,
    Proven,
    Disproven,
}

pub struct DiagnosticEngine {
    state: DiagnosticState,
    hypotheses: Vec<Hypothesis>,
    evidence_collected: Vec<String>,
    audit_store: Option<std::sync::Arc<AuditStore>>,
}

impl DiagnosticEngine {
    pub fn new(audit_store: Option<std::sync::Arc<AuditStore>>) -> Self {
        Self {
            state: DiagnosticState::Observe,
            hypotheses: Vec::new(),
            evidence_collected: Vec::new(),
            audit_store,
        }
    }

    pub fn transition(&mut self, next_state: DiagnosticState) {
        let old_state = format!("{:?}", self.state);
        let new_state = format!("{:?}", next_state);
        
        self.state = next_state;

        if let Some(ref audit) = self.audit_store {
            let _ = audit.append_event(
                "agent",
                "diagnostics",
                "state_transition",
                "DiagnosticEngine",
                Some("allow"),
                Some("success"),
                Some(&format!("Transition from {} to {}", old_state, new_state)),
                None,
                &hash_payload(&format!("{}->{}", old_state, new_state)),
                None,
                None,
            );
        }
    }

    pub fn get_state(&self) -> DiagnosticState {
        self.state.clone()
    }

    pub fn add_evidence(&mut self, evidence: String) {
        if let Some(ref audit) = self.audit_store {
            let _ = audit.append_event(
                "agent",
                "diagnostics",
                "evidence_collection",
                "DiagnosticEngine",
                Some("allow"),
                Some("success"),
                Some(&format!("Collected evidence: {}", evidence)),
                None,
                &hash_payload(&evidence),
                None,
                None,
            );
        }

        self.evidence_collected.push(evidence);
    }

    pub fn collect_log_evidence(&mut self, limit: usize) -> Result<(), String> {
        let logs = get_system_logs(limit)?;
        for entry in logs {
            self.add_evidence(format!("[{}] {} {}: {}", entry.timestamp, entry.level, entry.source, entry.message));
        }
        Ok(())
    }

    pub fn collect_system_evidence(&mut self) {
        let mut provider = SystemProvider::new();
        let stats = provider.get_system_stats();
        self.add_evidence(format!("System Stats: CPU Usage: {:.2}%, Mem: {}/{} bytes", 
            stats.global_cpu_usage, stats.used_memory, stats.total_memory));
        
        let procs = provider.list_processes();
        // Just add top 5 processes by CPU as evidence
        let mut procs = procs;
        procs.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        for proc in procs.iter().take(5) {
            self.add_evidence(format!("Process: {} (PID: {}), CPU: {:.2}%, Status: {}", 
                proc.name, proc.pid, proc.cpu_usage, proc.status));
        }
    }

    pub fn add_hypothesis(&mut self, description: String) {
        self.hypotheses.push(Hypothesis {
            description: description.clone(),
            status: HypothesisStatus::Pending,
            evidence: Vec::new(),
        });

        if let Some(ref audit) = self.audit_store {
            let _ = audit.append_event(
                "agent",
                "diagnostics",
                "hypothesis_generation",
                "DiagnosticEngine",
                Some("allow"),
                Some("success"),
                Some(&format!("New hypothesis: {}", description)),
                None,
                &hash_payload(&description),
                None,
                None,
            );
        }
    }

    pub fn update_hypothesis(&mut self, index: usize, status: HypothesisStatus, evidence: String) {
        if let Some(h) = self.hypotheses.get_mut(index) {
            h.status = status.clone();
            h.evidence.push(evidence.clone());

            if let Some(ref audit) = self.audit_store {
                let _ = audit.append_event(
                    "agent",
                    "diagnostics",
                    "hypothesis_test",
                    "DiagnosticEngine",
                    Some("allow"),
                    Some("success"),
                    Some(&format!("Hypothesis {} updated to {:?}: {}", h.description, status, evidence)),
                    None,
                    &hash_payload(&format!("{:?}:{}", status, evidence)),
                    None,
                    None,
                );
            }
        }
    }

    pub fn format_evidence(&self) -> String {
        self.evidence_collected.join("\n")
    }

    pub fn synthesize_diagnosis(&self) -> String {
        let proven: Vec<_> = self.hypotheses.iter()
            .filter(|h| h.status == HypothesisStatus::Proven)
            .collect();
        
        let final_diagnosis = if proven.is_empty() {
            "No conclusive diagnosis reached.".to_string()
        } else {
            let mut summary = String::from("Final Diagnosis:\n");
            for h in proven {
                summary.push_str(&format!("- {}\n", h.description));
                for e in &h.evidence {
                    summary.push_str(&format!("  Evidence: {}\n", e));
                }
            }
            summary
        };

        if let Some(ref audit) = self.audit_store {
            let _ = audit.append_event(
                "agent",
                "diagnostics",
                "synthesis",
                "DiagnosticEngine",
                Some("allow"),
                Some("success"),
                Some("Synthesized final diagnosis"),
                None,
                &hash_payload(&final_diagnosis),
                None,
                None,
            );
        }

        final_diagnosis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_lifecycle() {
        let mut engine = DiagnosticEngine::new(None);
        assert_eq!(engine.get_state(), DiagnosticState::Observe);

        engine.add_evidence("Found 404 errors in logs".to_string());
        engine.transition(DiagnosticState::Hypothesize);
        assert_eq!(engine.get_state(), DiagnosticState::Hypothesize);

        engine.add_hypothesis("Missing static files".to_string());
        engine.transition(DiagnosticState::Test);
        assert_eq!(engine.get_state(), DiagnosticState::Test);

        engine.update_hypothesis(0, HypothesisStatus::Proven, "Checked /var/www/static and it is empty".to_string());
        engine.transition(DiagnosticState::Synthesize);
        
        let diagnosis = engine.synthesize_diagnosis();
        assert!(diagnosis.contains("Missing static files"));
        assert!(diagnosis.contains("Checked /var/www/static and it is empty"));
    }
}
