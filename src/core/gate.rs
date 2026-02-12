pub struct ExpressionGate {
    pub _metabolic_cost_per_word: f32,
    pub _meaningful_threshold: f32,
    pub last_vocalization_tick: u64,
    pub cooldown_ticks: u64,
}

impl ExpressionGate {
    pub fn new() -> Self {
        Self {
            _metabolic_cost_per_word: 0.01,
            _meaningful_threshold: 0.5,  // RAISED: Minimum entropy to even consider speaking
            last_vocalization_tick: 0,
            cooldown_ticks: 30,        // 0.5s at 60Hz - much more responsive
        }
    }

    pub fn attempt_vocalization(&mut self, adenosine: f32, entropy: f32, dopamine: f32, text: &str, current_tick: u64) -> bool {
        // 0. COOLDOWN CHECK (Prevent verbal diarrhea)
        if current_tick < self.last_vocalization_tick + self.cooldown_ticks {
            return false;
        }

        // 1. HALLUCINATION FILTER (The Anti-Marketing Firewall)
        // ALEPH is an organism, not a salesman.
        let blacklist = ["info product", "marketing", "subscribe", "chatbot", "language model", "http", "www", "AI"];
        let lowercase = text.to_lowercase();
        for &word in &blacklist {
            if lowercase.contains(word) {
                return false; // Silence hallucinations immediately
            }
        }

        // 2. PHYSICAL CHECK (The Body - Veto Power)
        // If adenosine is > 0.7, the system is too tired. Silence.
        if adenosine > 0.7 {
             return false;
        }

        // 3. LENGTH CHECK (Avoid garbage tokens)
        let word_count = text.split_whitespace().count();
        if word_count < 1 { return false; } // Allow single words (e.g. "Hola!")
        // if word_count > 40 { return false; } // Too long check kept 

        // 4. METABOLIC VALVE (Entropy vs Fatigue)
        // The "Density" of the thought must justify the cost.
        let speech_drive = entropy + (dopamine * 0.8); // High Dopamine = HIGH DRIVE
        let speech_resistance = adenosine + 0.2; // Lower resistance threshold
        
        if speech_drive <= speech_resistance {
            // EXCEPTION: ultra high dopamine overrides resistance
            if dopamine < 0.9 {
                return false;
            }
        }

        // 5. SENTENCE COMPLETION
        // We relax this. If it's a thought, just say it.
        // let is_complete = text.contains('.') || text.contains('!') || text.contains('?');
        // if !is_complete { return false; }

        // 6. VOCALIZATION APPROVED
        self.last_vocalization_tick = current_tick;
        true
    }
}

