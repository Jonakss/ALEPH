pub struct ExpressionGate {
    pub _metabolic_cost_per_word: f32,
    pub meaningful_threshold: f32,
    pub last_vocalization_tick: u64,
    pub cooldown_ticks: u64,
}

impl ExpressionGate {
    pub fn new() -> Self {
        Self {
            _metabolic_cost_per_word: 0.01,
            meaningful_threshold: 0.5,  // RAISED: Minimum entropy to even consider speaking
            last_vocalization_tick: 0,
            cooldown_ticks: 300,        // ~5 seconds at 60Hz between vocalizations
        }
    }

    pub fn attempt_vocalization(&mut self, adenosine: f32, entropy: f32, dopamine: f32, text: &str, current_tick: u64) -> bool {
        // 0. COOLDOWN CHECK (Prevent verbal diarrhea)
        if current_tick < self.last_vocalization_tick + self.cooldown_ticks {
            return false;
        }

        // 1. PHYSICAL CHECK (The Body - Veto Power)
        // If adenosine is > 0.8, the system is too tired. Silence.
        if adenosine > 0.8 {
             return false;
        }

        // 2. LENGTH CHECK (Avoid garbage tokens)
        let word_count = text.split_whitespace().count();
        if word_count < 3 { return false; }  // RAISED from 2
        if word_count > 50 { return false; } // Too long = probably garbage

        // 3. MINIMUM SIGNIFICANCE (Not everything is worth saying)
        if entropy < self.meaningful_threshold {
            return false;  // Too mundane
        }

        // 4. METABOLIC VALVE (Entropy vs Fatigue)
        // The "Density" of the thought must justify the cost.
        // Also factor in dopamine - high interest makes you more talkative.
        let speech_drive = entropy + (dopamine * 0.3);
        let speech_resistance = adenosine + 0.3; // Base resistance (silence is default)
        
        if speech_drive <= speech_resistance {
            return false;
        }

        // 5. SENTENCE COMPLETION
        let is_complete = text.contains('.') || text.contains('!') || text.contains('?');
        if !is_complete { return false; }

        // 6. VOCALIZATION APPROVED
        self.last_vocalization_tick = current_tick;
        true
    }
}

