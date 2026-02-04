use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn voice_label(&self) -> &str {
        match self.voice {
            MindVoice::Sensory => "EAR",
            MindVoice::Cortex => "PLANET",
            MindVoice::Chem => "STAR",
            MindVoice::System => "SYS",
            MindVoice::Vocal => "VOCAL",
        }
    }
}
