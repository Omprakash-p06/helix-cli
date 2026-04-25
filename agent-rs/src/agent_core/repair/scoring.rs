/// Bayesian Confidence Scoring for Diagnostic Hypotheses
/// 
/// Logic: Confidence = (Token Probs * Tool Reliability) - Information Gap

pub fn calculate_confidence(
    token_probs: f64, 
    historical_reliability: f64, 
    evidence_coverage: f64 
) -> f64 {
    let raw_score = token_probs * historical_reliability;
    let gap_penalty = (1.0 - evidence_coverage) * 0.5;
    (raw_score - gap_penalty).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_calibration() {
        // High everything
        assert!(calculate_confidence(0.9, 0.9, 1.0) > 0.8);
        
        // Low evidence coverage
        assert!(calculate_confidence(0.9, 0.9, 0.2) < 0.6);
        
        // Low token probability
        assert!(calculate_confidence(0.3, 0.9, 1.0) < 0.4);
        
        // Clamp to 0.0
        assert_eq!(calculate_confidence(0.1, 0.1, 0.0), 0.0);
    }
}
