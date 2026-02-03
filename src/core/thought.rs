use std::time::Instant;

#[derive(Debug, Clone)]
pub enum MindVoice {
    Sensory, // [EAR], [BODY] - Cyan
    Cortex,  // [BROCA], [WERNICKE], [THINK] - Green
    Chem,    // [DOPA], [CORT] - Magenta
    System,  // [SYS], [SAVE] - DarkGray/Yellow
    Vocal,   // [SPEAK] - White/Bold
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
}
