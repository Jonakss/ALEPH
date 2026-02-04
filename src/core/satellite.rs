use std::time::Duration;

pub struct Satellite {
    pub _paranoia: f32, // 0.0 - 1.0 (Membrane Sensitivity)
    pub _refractive_index: f32, // 0.5 (Neutral)
    pub _lucidity: f32, // 0.0 - 1.0 (Distance from drama)
}

impl Satellite {
    pub fn new(paranoia: f32, refractive_index: f32) -> Self {
        Self {
            paranoia,
            refractive_index,
            lucidity: 1.0,
        }
    }

    /// INPUT FILTER (The Membrane)
    /// Decides if a stimulus penetrates the psyche or is rejected (Hardened).
    pub fn filter_input(&self, text: &str, entropy: f32, attention: f32) -> Option<String> {
        let _rng = rand::thread_rng();
        
        // 1. HARDENING (Rejection)
        // If Entropy > Attention, we naturally ignore. 
        // Paranoia INCREASES the threshold for acceptance (Hyper-vigilance filters noise).
        // Wait, Paranoia usually means seeing patterns in noise. 
        // "Inflammation" = Paranoia => We see THREATS. 
        // "Hardening" = Callousness => We ignore INPUT.
        
        // Let's use Paranoia as a "Threat Detector". High Paranoia = High Alert.
        // But here we want to model "Refusal to Engage" (Biological Strike).
        
        // Logic:
        // Input is rejected if Entropy (Internal Chaos) is too high relative to Attention.
        if entropy > (attention + 0.1) {
            return None; // Zoned out
        }

        Some(text.to_string())
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
