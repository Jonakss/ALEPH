use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum CognitiveEvent {
    StimulusStart(f32), // Sudden rise in energy/entropy
    Trauma(f32),        // Entropy > Threshold (Panic)
    Stagnation,         // Entropy near 0
    Flow,               // Optimal state
    Boredom,            // Low variance for too long
    Neurogenesis,       // Growth Trigger
}

impl std::fmt::Display for CognitiveEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CognitiveEvent::StimulusStart(val) => write!(f, "ATTENTION: Stimulus Detected (Δ: {:.2})", val),
            CognitiveEvent::Trauma(val) => write!(f, "TRAUMA: Overwhelmed (H: {:.2})", val),
            CognitiveEvent::Stagnation => write!(f, "STAGNATION: System Idle"),
            CognitiveEvent::Flow => write!(f, "FLOW: Optimal State"),
            CognitiveEvent::Boredom => write!(f, "BOREDOM: Seeking Stimulus"),
            CognitiveEvent::Neurogenesis => write!(f, "🧬 NEUROGENESIS: Structural Growth Initiated"),
        }
    }
}

pub struct Neocortex {
    entropy_history: VecDeque<f32>,
    last_derivative: f32,
    trauma_counter: usize,   // Ticks in high entropy
    growth_cooldown: usize,  // Ticks until next growth allowed
}

impl Neocortex {
    pub fn new() -> Self {
        let mut history = VecDeque::new();
        // Pre-fill with 0 to allow derivative calc immediately
        history.push_back(0.0); 
        Self {
            entropy_history: history,
            last_derivative: 0.0,
            trauma_counter: 0,
            growth_cooldown: 0,
        }
    }

    pub fn observe(&mut self, current_entropy: f32) -> Option<CognitiveEvent> {
        // Cooldown tick
        if self.growth_cooldown > 0 {
            self.growth_cooldown -= 1;
        }

        // 1. Memory Management (Keep last 2 ticks for simple derivative)
        let last_entropy = *self.entropy_history.back().unwrap_or(&0.0);
        self.entropy_history.push_back(current_entropy);
        if self.entropy_history.len() > 10 {
            self.entropy_history.pop_front();
        }

        // 2. Math: Calculate Derivative
        let derivative = current_entropy - last_entropy;
        self.last_derivative = derivative;

        // 3. Logic: Event Detection (Structual Consciousness)
        
        // A. Neurogenesis Check (Chronic Stress)
        if current_entropy > 0.7 {
            self.trauma_counter += 1;
        } else {
            if self.trauma_counter > 0 {
                self.trauma_counter -= 1; // Heal slowly
            }
        }

        // Si trauma dura >= 300 ticks (5s a 60Hz) y no hay cooldown
        if self.trauma_counter > 300 && self.growth_cooldown == 0 {
            self.trauma_counter = 0; // Reset trauma (catharsis)
            self.growth_cooldown = 300; // 5s cooldown
            return Some(CognitiveEvent::Neurogenesis);
        }
        
        // B. Sudden Spike (Attention)
        if derivative > 0.15 {
             return Some(CognitiveEvent::StimulusStart(derivative));
        }

        // C. Trauma (Panic Threshold) - Solo avisar si es muy alto
        if current_entropy > 0.85 {
            return Some(CognitiveEvent::Trauma(current_entropy));
        }

        // D. Stagnation / Boredom (Machine Zone)
        if current_entropy < 0.05 {
             return Some(CognitiveEvent::Stagnation);
        }

        None
    }
}
