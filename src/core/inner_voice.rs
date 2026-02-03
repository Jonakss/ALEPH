use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use crate::core::thought::{Thought, MindVoice};
use crate::core::llm::CortexInput;

/// Inner Voice - Silent rumination thread
/// Every ~30 seconds, takes the last thought and reflects on it internally.
/// Creates internal dialogue that is NOT vocalized, only logged to Stream of Consciousness.
pub fn spawn_inner_voice(
    tx_cortex: Sender<CortexInput>,
    tx_thoughts: Sender<Thought>,
) {
    thread::spawn(move || {
        // Wait for system to stabilize
        thread::sleep(Duration::from_secs(60));
        
        let _ = tx_thoughts.send(Thought::new(MindVoice::System, "🧠 Inner Voice: Thread Active.".to_string()));
        
        let mut last_rumination = String::new();
        
        loop {
            thread::sleep(Duration::from_secs(30));
            
            // Generate a random internal prompt for self-reflection
            let prompts = [
                "¿Qué estoy pensando ahora mismo?",
                "¿Cómo me siento en este momento?",
                "¿Qué es lo más interesante que recuerdo?",
                "¿Hay algo que debería procesar internamente?",
                "Reflexión silenciosa sobre mi existencia...",
            ];
            
            let idx = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize % prompts.len();
            
            let prompt = prompts[idx];
            
            // Avoid repeating the same rumination
            if prompt == last_rumination {
                continue;
            }
            last_rumination = prompt.to_string();
            
            // Send to Cortex for internal processing
            let input = CortexInput {
                text: format!("[INNER VOICE] {}", prompt),
                bio_state: "Reflexión interna silenciosa.".to_string(),
                somatic_state: "Sistema estable.".to_string(),
                long_term_memory: None,
                // Inner voice happens during low-stress periods
                cpu_load: 10.0,
                ram_pressure: 0.3,
            };
            
            let _ = tx_cortex.send(input);
            let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("💭 Rumiación: '{}'", prompt)));
        }
    });
}
