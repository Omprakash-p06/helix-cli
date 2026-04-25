use agent_rs::agent_core::diagnostics::reasoning::{DiagnosticEngine, DiagnosticState, HypothesisStatus};
use agent_rs::audit::AuditStore;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_service_crash_diagnostic_loop() {
    let tmp = tempdir().unwrap();
    let audit_db = tmp.path().join("audit.db");
    let audit_store = Arc::new(AuditStore::new(&audit_db).unwrap());
    
    let mut engine = DiagnosticEngine::new(Some(audit_store.clone()));
    
    // 1. Observe
    assert_eq!(engine.get_state(), DiagnosticState::Observe);
    engine.add_evidence("User reports web server is down".to_string());
    engine.add_evidence("Logs show 'Connection refused' on port 80".to_string());
    
    // 2. Hypothesize
    engine.transition(DiagnosticState::Hypothesize);
    assert_eq!(engine.get_state(), DiagnosticState::Hypothesize);
    engine.add_hypothesis("Nginx service is stopped".to_string());
    engine.add_hypothesis("Port 80 is blocked by firewall".to_string());
    
    // 3. Test
    engine.transition(DiagnosticState::Test);
    assert_eq!(engine.get_state(), DiagnosticState::Test);
    
    // Test hypothesis 0
    engine.update_hypothesis(0, HypothesisStatus::Proven, "systemctl status nginx shows 'inactive (dead)'".to_string());
    
    // 4. Synthesize
    engine.transition(DiagnosticState::Synthesize);
    assert_eq!(engine.get_state(), DiagnosticState::Synthesize);
    let diagnosis = engine.synthesize_diagnosis();
    
    assert!(diagnosis.contains("Nginx service is stopped"));
    assert!(diagnosis.contains("Final Diagnosis:"));
    
    // Verify audit log
    let events = audit_store.query_events(None, None, None, None, None, None).unwrap();
    assert!(events.iter().any(|e| e.event_type == "state_transition"), "Should have state_transition events");
    assert!(events.iter().any(|e| e.event_type == "hypothesis_generation"), "Should have hypothesis_generation events");
    assert!(events.iter().any(|e| e.event_type == "evidence_collection"), "Should have evidence_collection events");
}

#[tokio::test]
async fn test_resource_exhaustion_diagnostic() {
    let tmp = tempdir().unwrap();
    let audit_db = tmp.path().join("audit.db");
    let audit_store = Arc::new(AuditStore::new(&audit_db).unwrap());
    
    let mut engine = DiagnosticEngine::new(Some(audit_store.clone()));
    
    // Observe
    engine.add_evidence("System is slow".to_string());
    engine.collect_system_evidence();
    
    // Hypothesize
    engine.transition(DiagnosticState::Hypothesize);
    engine.add_hypothesis("CPU spike due to runaway process".to_string());
    
    // Test
    engine.transition(DiagnosticState::Test);
    engine.update_hypothesis(0, HypothesisStatus::Proven, "Found 'stress-ng' using 99% CPU".to_string());
    
    // Synthesize
    engine.transition(DiagnosticState::Synthesize);
    let diagnosis = engine.synthesize_diagnosis();
    
    assert!(diagnosis.contains("CPU spike due to runaway process"));
    assert!(diagnosis.contains("stress-ng"));
}

#[tokio::test]
async fn test_path_permission_check_diagnostic() {
    let tmp = tempdir().unwrap();
    let audit_db = tmp.path().join("audit.db");
    let audit_store = Arc::new(AuditStore::new(&audit_db).unwrap());
    
    let mut engine = DiagnosticEngine::new(Some(audit_store.clone()));
    
    // Observe
    engine.add_evidence("Cannot read /root/.ssh/id_rsa".to_string());
    
    // Hypothesize
    engine.transition(DiagnosticState::Hypothesize);
    engine.add_hypothesis("Insufficient permissions".to_string());
    
    // Test
    engine.transition(DiagnosticState::Test);
    // In real scenario, the agent would try to read it and fail
    engine.update_hypothesis(0, HypothesisStatus::Proven, "Confirmed permission denied for non-root user".to_string());
    
    // Synthesize
    engine.transition(DiagnosticState::Synthesize);
    let diagnosis = engine.synthesize_diagnosis();
    
    assert!(diagnosis.contains("Insufficient permissions"));
}
