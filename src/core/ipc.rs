use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AlephPacket {
    /// Daemon -> Client: The heartbeat of the organism
    Telemetry {
        // Biological
        adenosine: f32,
        cortisol: f32,
        dopamine: f32,
        
        // Proprioception
        heart_rate: f32, // Derived from CPU
        lucidity: f32,   // Derived from RAM/Entropy
        
        // Narrative
        short_term_memory: Vec<String>, // Last few thoughts
        current_state: String, // "Dreaming", "Alert", "Panic"
    },
    
    /// Client -> Daemon: Perturbations
    Stimulus {
        text: String,
        force: f32, // Intensity of the input
    }
}
