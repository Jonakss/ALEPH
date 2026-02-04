// Unused imports from ratatui removed as this file is now just a data definition wrapper.
// The actual TUI logic resides in tui/client.rs.

use crate::core::thought::Thought;

// Estructura de Telemetría que viene del Backend
#[allow(dead_code)]
pub struct Telemetry {
    pub audio_spectrum: crate::senses::ears::AudioSpectrum, // Full Spectrum
    pub entropy: f32,         // 0.0 - 1.0
    pub neuron_active_count: usize, // Memories (Vectors)
    pub reservoir_size: usize,      // Structural Neurons (Processing Power)
    pub system_status: String,// "FLOW", "PANIC", etc.
    #[allow(dead_code)]
    pub last_entropy_delta: f32, // Cambio de entropía (reservado para Variable Metabolism)
    pub fps: f64,             // Backend ticks per second
    pub cpu_load: f32,        // Proprioception
    pub ram_load: f32,        // Proprioception
    pub adenosine: f32, // Sleep Pressure
    pub dopamine: f32,  // Reward
    pub cortisol: f32,  // Stress
    pub insight_intensity: f32, // 0.0 - 1.0 (Flash trigger)
    pub timeline: Vec<Thought>, // Unified Stream of Consciousness
    pub activity_map: Vec<f32>, // Neuronal activity (100 neurons, 0.0-1.0)
    pub novelty_score: f32, // Last novelty check result
    pub reservoir_state: String, // Description of reservoir mood
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            audio_spectrum: crate::senses::ears::AudioSpectrum::default(),
            entropy: 0.0,
            neuron_active_count: 0,
            reservoir_size: 100, // Default start
            system_status: "INIT".to_string(),
            last_entropy_delta: 0.0,
            fps: 0.0,
            cpu_load: 0.0,
            ram_load: 0.0,
            adenosine: 0.0,
            dopamine: 0.5,
            cortisol: 0.0,
            insight_intensity: 0.0,
            timeline: Vec::new(),
            activity_map: vec![0.0; 100],
            novelty_score: 0.0,
            reservoir_state: "Estable".to_string(),
        }
    }
}

pub mod client;
mod avatar;
mod monologue;

// ui function removed (logic is in client.rs)
