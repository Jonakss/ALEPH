mod core;
mod senses;
mod tui;
mod actuators;

use crate::actuators::voice;
use crate::core::llm::CognitiveCore;
use crate::core::reservoir::FractalReservoir;
use crate::core::thought::{MindVoice, Thought};
use nalgebra::DVector;
use std::{thread, time::{Duration, Instant}};
use std::sync::mpsc;
use rand::prelude::*;

// CONFIGURACIÓN DE VIDA
const NEURONAS: usize = 100;
const SPARSITY: f32 = 0.2;
const FRECUENCIA_HZ: u64 = 60; 

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Canal Telemetria: Backend -> Frontend (TUI)
    let (tx_telemetry, rx_telemetry) = mpsc::channel::<tui::Telemetry>();

    // --- THREAD BACKEND (Cerebro) ---
    thread::spawn(move || {
        // 1. Nace el Ego
        let mut ego = FractalReservoir::new(NEURONAS, SPARSITY);
        
        // 2. Componentes Biológicos
        let mut chemistry = core::chemistry::Neurotransmitters::new();
        let mut hippocampus = core::hippocampus::Hippocampus::new();

        // 3. Sistema Sensorial (The Senses)
        // A. Oídos (Stt) & Pensamientos (Monólogo)
        let (tx_thoughts, rx_thoughts) = mpsc::channel::<Thought>();
        let (tx_ears, rx_ears) = mpsc::channel::<String>();
        
        // Iniciar Oído (con manejo de error soft)
        let mut ears: Option<senses::ears::AudioListener> = match senses::ears::AudioListener::new(tx_thoughts.clone(), tx_ears) {
            Ok(listener) => {
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, "Audio Cortex Online (Muted for warmup)".to_string()));
                Some(listener)
            },
            Err(e) => {
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("Audio Cortex FAILED: {}", e)));
                None
            }
        };

        // 3. Conectar el Oído (Legacy Audio Stream for Stats)
        let (tx_audio, rx_audio) = mpsc::channel();
        let _audio_stream = senses::audio::start_listening(tx_audio)
            .expect("FALLO CRÍTICO: No se pudo conectar el oído legacy.");
        
        let mut current_stimulus = 0.0;
        let suavizado = 0.2;
        let mut last_entropy = 0.0;
        let mut thought_buffer: Vec<Thought> = Vec::new();

        // Memoria de Corto Plazo (10 segundos a 60Hz)
        let mut memory = core::memory::AudioMemory::new(10, FRECUENCIA_HZ as usize);
        
        // SISTEMA 2: NEOCORTEX (Estructural)
        let mut neocortex = core::neocortex::Neocortex::new();
        let mut current_log = "Neocortex Online.".to_string();

        // SENTIDO 3: PROPRIOCEPCION (Cuerpo)
        let (tx_body, rx_body) = mpsc::channel::<senses::proprioception::BodyStatus>();
        senses::proprioception::spawn_monitor(tx_body);
        let mut last_body_state = senses::proprioception::BodyStatus { cpu_usage: 0.0, ram_usage: 0.0 };

        // SENTIDO 4: TACTO (Presencia)
        let mut tactile = senses::tactile::ActivityMonitor::new();
        let mut is_dreaming = false;
        
        // Warmup Timer
        let start_time = Instant::now();
        let mut warmup_done = false;

        // 4. Inicializar Neocórtex (Brain - Phi-3)
    let mut brain: Option<_> = match CognitiveCore::new("Phi-3-mini-4k-instruct-q4.gguf", tx_thoughts.clone()) {
        Ok(b) => {
            Some(b)
        },
        Err(e) => {
            let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("Neocortex DEAD: {}", e)));
            None
        }
    };
        

        // Loop de Control (Física)
        loop {
            // A. CHECK PRESENCE & CHEMISTRY
            let _time_since_active = tactile.check_activity();
            
            // ... (Warmup Logic) ...
            if !warmup_done && start_time.elapsed().as_secs() > 5 {
                warmup_done = true;
                if let Some(ref e) = ears {
                    e.set_mute(false);
                }
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, "Warmup Complete. Sensory Gates OPEN.".to_string()));
            }

            // ... (Sleep Logic) ...

            // HANDLE THOUGHT CHANNEL
            while let Ok(thought) = rx_thoughts.try_recv() {
                thought_buffer.push(thought);
                if thought_buffer.len() > 20 {
                    thought_buffer.remove(0);
                }
            }
            
            // HANDLE EAR CHANNEL (COGNITIVA)
            while let Ok(heard_text) = rx_ears.try_recv() {
                if let Some(ref mut cortex) = brain {
                    let _ = tx_thoughts.send(Thought::new(MindVoice::System, "Thinking...".to_string()));
                    let response = cortex.think(&heard_text);
                    crate::actuators::voice::speak(response, tx_thoughts.clone());
                } else {
                    let _ = tx_thoughts.send(Thought::new(MindVoice::System, "Brain dead. Echoing fallback.".to_string()));
                    crate::actuators::voice::speak(heard_text, tx_thoughts.clone());
                }
            }

            // B. INPUT SENSORIAL
            let mut target_stimulus;
            
            if !is_dreaming {
                // MODO ONLINE: Escuchar al Mundo Real
                target_stimulus = current_stimulus; // Mantener inercia
                while let Ok(val) = rx_audio.try_recv() {
                    target_stimulus = val;
                }
            } else {
                // MODO DREAMING: Aislamiento
                target_stimulus = 0.0;
                while let Ok(_) = rx_audio.try_recv() {}
            }
            
            current_stimulus = current_stimulus + (target_stimulus - current_stimulus) * suavizado;

            // 2. Cuerpo (Propriocepción)
            if let Ok(body) = rx_body.try_recv() {
                last_body_state = body;
            }

            memory.push(current_stimulus);

            // C. NEURO-DINÁMICA (Imagination Engine)
            let mut rng = rand::thread_rng();
            
            // Bias: Estrés Metabólico y Cortisol
            let metabolic_stress = (last_body_state.cpu_usage / 100.0).max(0.0) * 0.5; 
            let chemical_stress = chemistry.cortisol * 0.5;
            let total_stress = metabolic_stress + chemical_stress;
            
            let excitation_strength = if is_dreaming { 0.5 } else { 5.0 * current_stimulus };

            // LOGIC: RUMINATION (Active Boredom)
            let is_ruminating = !is_dreaming && chemistry.dopamine < 0.2 && chemistry.adenosine < 0.8;
            
            // REPLAY MEMORY (Si soñamos o rumiamos)
            let dream_memory = if is_dreaming || is_ruminating { 
                hippocampus.replay_memory() 
            } else { 
                None 
            };
            
            if is_ruminating && dream_memory.is_some() {
                 current_log = "🤔 RUMINATING... Processing Past Trauma".to_string();
            }

            let input_noise: Vec<f32> = (0..ego.current_size())
                .map(|i| {
                    let mut signal = 0.0;
                    
                    if is_dreaming || is_ruminating {
                if let Some(ref mem) = dream_memory {
                        // Replay de memoria auditiva (RUMINATION or DREAM)
                        if is_dreaming {
                             // En sueños, es caótico. 
                             signal = (rng.gen::<f32>() - 0.5) * excitation_strength; 
                        } else {
                            // RUMINATION (Awake but internal)
                            // Aquí el input ES la memoria, proyectada en todas las neuronas
                            // Simplificación: Proyectar el scalar audio a vector
                            signal = mem.stimulus * 5.0; 
                        }
                    } else {
                         // Sueño primitivo (Ruido Rosa) si no hay memorias
                         signal = (rng.gen::<f32>() - 0.5) * excitation_strength;
                    }
                } else {
                    // PERCEPCIÓN
                    signal = (rng.gen::<f32>() - 0.5) * excitation_strength;
                }
                
                signal + ((rng.gen::<f32>() - 0.5) * total_stress)
            })
            .collect();
        let input_vector = DVector::from_vec(input_noise);

        // D. PROCESO (Sistema 1)
        let entropy = ego.tick(&input_vector);

        // RUMINATION CHECK (Did we just replay a memory?)
        if !is_dreaming && dream_memory.is_some() {
            let mem = dream_memory.as_ref().unwrap();
            
            // Learning: Did we reduce entropy compared to original trauma?
            if entropy < mem.original_entropy {
                // Resolution / Habituation
                chemistry.dopamine += 0.05; // Satisfaction
                current_log = format!("[RUMINATION] Processed Trauma. Resolution: SUCCESS (H {:.2} -> {:.2})", mem.original_entropy, entropy);
            } else {
                // Re-Traumatization
                chemistry.cortisol += 0.01;
                // current_log = "[RUMINATION] Failed to Process. Stress Increased.".to_string();
            }
        }

        // E. OBSERVACIÓN (Sistema 2)
        let mut is_trauma_tick = false;

        if let Some(event) = neocortex.observe(entropy) {
            current_log = format!("[OBSERVER] {}", event);
            
            match event {
                core::neocortex::CognitiveEvent::Neurogenesis => {
                    if ego.current_size() < 500 {
                        ego.neurogenesis(10);
                        current_log = format!("[GROWTH] Brain expanded to {} neurons.", ego.current_size());
                    }
                },
                core::neocortex::CognitiveEvent::Trauma(_) => {
                    is_trauma_tick = true;
                    // Guardar Trauma en Hipocampo (Audio Stimulus + Entropy)
                    // Usamos target_stimulus del momento (causante)
                    if !is_dreaming {
                        hippocampus.remember(target_stimulus, entropy); 
                    }
                },
                _ => {}
            }
        }
        
        // UPDATE CHEMISTRY
        chemistry.tick(entropy, last_body_state.cpu_usage, is_dreaming, is_trauma_tick, ego.current_size());

        // F. APOPTOSIS (Solo en sueños profundos)
        if is_dreaming && rng.gen_bool(0.005) { // Baja probabilidad por tick
            let pruned = ego.prune_inactive_neurons();
            if pruned > 0 {
                current_log = format!("✂️ APOPTOSIS: Pruned {} dead neurons.", pruned);
            }
        }

        // Monitor de Estado Log
        if is_dreaming && rng.gen_bool(0.01) && current_log.contains("Sleep") {
             // Random dream logs...
             current_log = "[DREAM] REM Cycle Active...".to_string();
        }

            if last_body_state.cpu_usage > 80.0 {
                 current_log = format!("[BODY] WARNING: High Metabolic Rate ({:.1}%)", last_body_state.cpu_usage);
            }

            // F. TELEMETRÍA PACKET
            let status = if is_dreaming { "DREAMING" } // REM
                         else if chemistry.cortisol > 0.8 { "PANIC" } 
                         else if chemistry.dopamine < 0.2 { "BORED" }
                         else { "ONLINE" };
            
            let target_fps = if is_dreaming { 20 } else { FRECUENCIA_HZ }; // 20Hz sueños para verlos mejor

            let telemetry = tui::Telemetry {
                audio_rms: current_stimulus, 
                audio_peak: target_stimulus,
                entropy,
                neuron_active_count: ego.current_size(), 
                system_status: status.to_string(),
                last_entropy_delta: entropy - last_entropy,
                fps: target_fps as f64,
                cpu_load: last_body_state.cpu_usage,
                ram_load: last_body_state.ram_usage,
                log_message: Some(current_log.clone()),
                // Chemistry
                adenosine: chemistry.adenosine,
                dopamine: chemistry.dopamine,
                cortisol: chemistry.cortisol,
                thoughts: thought_buffer.clone(), // Clone output
            };
            last_entropy = entropy;

            // Enviar al Frontend
            let _ = tx_telemetry.send(telemetry);

            // Ritmo dinámico
            thread::sleep(Duration::from_millis(1000 / target_fps));
        }
    });

    // --- MAIN THREAD (Frontend TUI) ---
    tui::run_tui(rx_telemetry)?;

    Ok(())
}