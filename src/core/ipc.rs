use serde::{Deserialize, Serialize};
use crate::senses::ears::AudioSpectrum;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AlephPacket {
    /// Daemon -> Client: The heartbeat of the organism
    Telemetry {
        // Biological
        adenosine: f32,
        cortisol: f32,
        dopamine: f32,
        oxytocin: f32, // Added
        
        // Sensory
        audio_spectrum: AudioSpectrum,
        
        // Proprioception
        heart_rate: f32, // Derived from CPU
        lucidity: f32,   // Derived from RAM/Entropy
        reservoir_activity: Vec<f32>, // Neural Grid State
        
        // Narrative
        short_term_memory: Vec<String>, // Last few thoughts
        current_state: String, // "Dreaming", "Alert", "Panic"
        entropy: f32, // Added for TUI Chart
        
        // System
        loop_frequency: f32, // Actual Backend Hz
        cpu_usage: f32,      // Hardware Load
    },
    
    /// Client -> Daemon: Perturbations
    Stimulus {
        text: String,
        force: f32, // Intensity of the input
    }
}
