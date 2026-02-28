use metaygn_shared::state::{BudgetState, MetacognitiveVector, RiskLevel};

#[test]
fn metacognitive_vector_quality() {
    let v = MetacognitiveVector {
        confidence: 0.8,
        coherence: 0.9,
        grounding: 0.7,
        complexity: 0.3,
        progress: 0.6,
    };
    // quality = (0.8 + 0.9 + 0.7 + (1 - 0.3) + 0.6) / 5
    //         = (0.8 + 0.9 + 0.7 + 0.7 + 0.6) / 5
    //         = 3.7 / 5
    //         = 0.74
    let q = v.overall_quality();
    assert!((q - 0.74).abs() < 1e-6, "expected ~0.74, got {q}");
}

#[test]
fn metacognitive_vector_compact_encode() {
    let v = MetacognitiveVector {
        confidence: 0.8,
        coherence: 0.9,
        grounding: 0.7,
        complexity: 0.3,
        progress: 0.6,
    };
    let encoded = v.compact_encode();
    // n = (value * 9) as u8
    // confidence: (0.8*9)=7.2 -> 7
    // coherence:  (0.9*9)=8.1 -> 8
    // grounding:  (0.7*9)=6.3 -> 6
    // complexity: (0.3*9)=2.7 -> 2
    // progress:   (0.6*9)=5.4 -> 5
    assert_eq!(encoded, "META:c7h8g6x2p5");
}

#[test]
fn budget_utilization() {
    let budget = BudgetState {
        max_tokens: 1000,
        consumed_tokens: 250,
        max_latency_ms: 5000,
        max_cost_usd: 1.0,
        risk_tolerance: RiskLevel::Medium,
    };
    assert_eq!(budget.tokens_remaining(), 750);
    assert!((budget.utilization() - 0.25).abs() < 1e-6);
}
