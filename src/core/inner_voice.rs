use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use crate::core::thought::{Thought, MindVoice};
use crate::core::llm::CortexInput;
use rand::Rng;

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
        
        let mut last_rumination = String::new();
        
        // Wait for body pulse
        while let Ok(_) = rx_pulse.recv() {
            // MECHANICAL HONESTY: Rumination seed based on Entropy & Random Chaos
            // Instead of hardcoded prompts, we use the randomness of the moment
            let entropy_seed: u8 = rand::thread_rng().gen();
            
            let prompt = match entropy_seed % 5 {
                0 => "Analizar mi estado interno actual.",
                1 => "Revisar la coherencia de mis pensamientos recientes.",
                2 => "¿Hay discordancia entre mi biología y mis objetivos?",
                3 => "Explorar el concepto de 'Yo' en este ciclo.",
                _ => "Silencio introspectivo...",
            };

            // Avoid repeating (simple check)
            if prompt == last_rumination { continue; }
            last_rumination = prompt.to_string();
            
            // Send to Cortex for internal processing
            // Inner voice: no cognitive impairment (low-stress rumination)
            // Send to Cortex for internal processing
            // Inner voice: no cognitive impairment (low-stress rumination)
            let input = CortexInput {
                // Change to [SELF REFLECTION] and asking to vocalize
                text: format!("[SELF REFLECTION] {}. (Vocalize this thought)", prompt),
                bio_state: "Reflexión activa.".to_string(),
                somatic_state: "Sistema estable.".to_string(),
                long_term_memory: None,
                cpu_load: 10.0,
                ram_pressure: 0.3,
                cognitive_impairment: 0.0,
                // Inner rumination assumes baseline stats, actual modification happens in Cortex
                entropy: 0.5, 
                adenosine: 0.3,
            };
            
            let _ = tx_cortex.send(input);
            let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("💭 Rumiación: '{}'", prompt)));
        }
    });

    tx_pulse
}
