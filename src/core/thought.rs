use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MindVoice {
    Sensory, // [F₁] - Inertia/Body (Hardware Input) - Cyan
    Cortex,  // [F₂] - Drift/Semantic (LLM Thought) - Green
    Chem,    // [ΔE] - Energy Delta (Chemical State) - Magenta
    System,  // [ΔS] - State Delta (System Event) - DarkGray
    Vocal,   // [F₃] - Collapse/Observer (Vocalized) - White/Bold
}

#[derive(Debug, Clone)]
pub struct Thought {
    pub voice: MindVoice,
    pub text: String,
    #[allow(dead_code)]
    pub timestamp: Instant, // Reservado para timeline / Variable Metabolism
}

impl Thought {
    pub fn new(voice: MindVoice, text: String) -> Self {
        Self {
            voice,
            text,
            timestamp: Instant::now(),
        }
    }

    pub fn voice_label(&self) -> &str {
        match self.voice {
            MindVoice::Sensory => "F₁",   // Inertia/Body (Hardware Input)
            MindVoice::Cortex => "F₂",    // Drift/Semantic (LLM Output)
            MindVoice::Chem => "ΔE",      // Energy Delta (Chemical State Change)
            MindVoice::System => "ΔS",    // State Delta (System Event)
            MindVoice::Vocal => "F₃",     // Collapse/Observer (Vocalized Output)
        }
    }
}
