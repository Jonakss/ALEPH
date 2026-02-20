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
        oxytocin: f32,
        
        // Sensory
        audio_spectrum: AudioSpectrum,
        
        // Proprioception
        heart_rate: f32,
        lucidity: f32,
        reservoir_activity: Vec<f32>,
        
        // Narrative
        short_term_memory: Vec<String>,
        current_state: String,
        entropy: f32,
        
        // System
        loop_frequency: f32,
        cpu_usage: f32,
        activations: Vec<f32>,
        region_map: Vec<u8>,
        reservoir_size: usize,
        visual_cortex: Vec<f32>, // 64x64 Grid from Eyes
        
        // Spatial Topology (Real backend positions)
        neuron_positions: Vec<[f32; 3]>,
    },
    
    /// Client -> Daemon: Perturbations
    Stimulus {
        text: String,
        force: f32, // Intensity of the input
    }
}
