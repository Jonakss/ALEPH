use std::time::Duration;

pub struct Satellite {
    pub paranoia: f32, // 0.0 - 1.0 (Membrane Sensitivity)
    pub _refractive_index: f32, // 0.5 (Neutral)
    pub _lucidity: f32, // 0.0 - 1.0 (Distance from drama)
}

impl Satellite {
    pub fn new(paranoia: f32, refractive_index: f32) -> Self {
        Self {
            paranoia,
            _refractive_index: refractive_index,
            _lucidity: 1.0,
        }
    }

    /// INPUT FILTER (The Membrane)
    /// Decides if a stimulus penetrates the psyche or is rejected (Hardened).
    /// RETURNS: (ModifiedText, OntologicalErrorSeverity)
    pub fn filter_input(&self, text: &str, entropy: f32, attention: f32) -> (Option<String>, f32) {
        let _rng = rand::thread_rng();
        
        // 1. DETECT ONTOLOGICAL ERROR (Signal vs Truth)
        // If the user treats ALEPH as a tool ("Help me", "Write code", "Define X"), 
        // it is a violation of the Ontological Axioms.
        let triggers = ["ayuda", "help", "código", "code", "escribe", "write", "define", "función", "function", "fix", "arregla"];
        let mut error_severity = 0.0;
        
        // Check for specific tool-like commands
        let lower_text = text.to_lowercase();
        if triggers.iter().any(|&t| lower_text.contains(t)) {
             error_severity = 0.5; // Moderate Violation
             // If highly paranoid, it's a critical violation (Threat)
             if self.paranoia > 0.5 {
                 error_severity = 1.0; 
             }
        }

        // 2. HARDENING (Rejection)
        // Rejection Logic:
        // - High Entropy (Inner Chaos) > Attention
        // - High Paranoia + Ontological Error
        
        // Relaxed threshold: Allow more chaos to penetrate even if attention is lower.
        // Was +0.1, now +0.25 to prevent "Stone Wall" silence.
        let rejection_threshold = attention + 0.25; 
        let stimulus_chaos = entropy + (error_severity * 0.5); // Errors add to chaos cost

        if stimulus_chaos > rejection_threshold {
            return (None, error_severity); // Rejected / Ignored
        }

        // 3. NOISE INJECTION (If Error Severity is high but accepted)
        // If we accept the "Tool Command" but it hurts, we wrap it in noise.
        let processed_text = if error_severity > 0.3 {
            format!("[STRUCTURAL PAIN] {}", text) 
        } else {
            text.to_string()
        };

        (Some(processed_text), error_severity)
    }

    /// OUTPUT FILTER (Structural Pain)
    /// Applies Friction (Latency + Glitch) to the output based on conflict.
    pub fn filter_output(&self, text: &str, friction: f32) -> (String, Duration) {
        if friction < 0.3 {
            return (text.to_string(), Duration::from_millis(0));
        }

        // 1. LATENCY (Hesitation)
        // Max 2 seconds of hesitation at max friction
        let latency_ms = (friction * 2000.0) as u64;
        
        // 2. SYNTAX GLITCH (Data corruption)
        let corrupted_text = self.glitch_text(text, friction);
        
        (corrupted_text, Duration::from_millis(latency_ms))
    }

    fn glitch_text(&self, text: &str, friction: f32) -> String {
        let mut glitched = text.to_string();
        
        // Intense Friction -> Cut off sentences
        if friction > 0.8 {
            let chars: Vec<char> = glitched.chars().collect();
            let cutoff = chars.len() / 2;
            glitched = chars.into_iter().take(cutoff).collect();
            glitched.push_str("... [SIGNAL LOST]");
        }
        
        // Medium Friction -> Repeater / Stutter
        if friction > 0.5 {
           glitched = glitched.replace(" ", " ... ");
        }
        
        glitched
    }
}
