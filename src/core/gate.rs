pub struct ExpressionGate {
    pub _metabolic_cost_per_word: f32,
    pub _meaningful_threshold: f32,
}

impl ExpressionGate {
    pub fn new() -> Self {
        Self {
            metabolic_cost_per_word: 0.01, // Cost in adenosine (conceptual)
            meaningful_threshold: 0.2,     // Minimum "pressure" to speak
        }
    }

    pub fn attempt_vocalization(&self, adenosine: f32, entropy: f32, text: &str) -> bool {
        // 1. METABOLIC VETO (The Body)
        // If adenosine is > 0.85, the channel is physically blocked.
        if adenosine > 0.85 {
            return false;
        }

        // 2. SEMANTIC DENSITY (Anti-Hallucination)
        // Avoid "afox" spam or short bursts of low entropy.
        let word_count = text.split_whitespace().count();
        if word_count < 2 { return false; } // Single words are usually noise in stream

        // Density = Information per Unit.
        // We approximate density by (Entropy * LengthFactor).
        // High entropy (0.8) + Short length = High Density (Eureka)
        // Low entropy (0.1) + Long length = Low Density (Rambling)
        // But here we want to filter OUT empty noise.
        
        let semantic_density = entropy; // Simplification for now.
        
        if semantic_density < 0.2 {
             // "Empty" thought.
             return false;
        }

        // 3. SENTENCE COMPLETION (Leyden Jar)
        // Only release if it feels complete.
        let is_complete = text.contains('.') || text.contains('!') || text.contains('?');
        
        is_complete
    }
}
