use std::sync::mpsc::Sender;
use std::thread;

use crate::core::thought::{Thought, MindVoice};
use crate::core::llm::CortexInput;
// use rand::Rng; 

/// Inner Voice - Silent rumination thread
/// TRIGGERED BY BODY PULSE.
/// Creates internal dialogue that is NOT vocalized, only logged to Stream of Consciousness.
pub fn spawn_inner_voice(
    tx_cortex: Sender<CortexInput>,
    tx_thoughts: Sender<Thought>,
) -> Sender<()> {
    let (tx_pulse, rx_pulse) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let _ = tx_thoughts.send(Thought::new(MindVoice::System, "🧠 Bio-Clock: Inner Voice Synced.".to_string()));
        
        // Wait for body pulse
        while let Ok(_) = rx_pulse.recv() {
            // MECHANICAL HONESTY: No scripts.
            // We provide the context (bio_state, adenosine, etc) and letting the LLM decide WHAT to think.
            // The "prompt" is just a trigger.
            
            let prompt = "Genera un pensamiento interno espontáneo basado en mi estado actual (Sin guión).";
            
            // Send to Cortex for internal processing
            // Inner voice: no cognitive impairment (low-stress rumination)
            let input = CortexInput {
                text: format!("[SELF REFLECTION] {}", prompt),
                bio_state: "Reflexión activa.".to_string(),
                _somatic_state: "Sistema estable.".to_string(),
                _long_term_memory: None,
                _cpu_load: 10.0,
                _ram_pressure: 0.3,
                _cognitive_impairment: 0.0,
                // Inner rumination assumes baseline stats, actual modification happens in Cortex
                entropy: 0.5, 
                adenosine: 0.3,
            };
            
            let _ = tx_cortex.send(input);
            let _ = tx_thoughts.send(Thought::new(MindVoice::System, "💭 Rumiación: Generando pensamiento espontáneo...".to_string()));
        }
    });

    tx_pulse
}
