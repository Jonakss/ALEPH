#![allow(deprecated)]

mod core;
mod senses;
mod tui;
mod actuators;

use crate::core::llm::{CognitiveCore, CortexInput, CortexOutput};
use crate::core::reservoir::FractalReservoir;
use crate::core::thought::{MindVoice, Thought};
use nalgebra::DVector;
use std::{thread, time::{Duration, Instant}};
use anyhow::Result;

enum BackendCommand {
    Poke,
}
use std::sync::mpsc;
use rand::prelude::*;
use std::io;
use gag::Gag;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

// CONFIGURACIÓN DE VIDA
const NEURONAS: usize = 500;
const SPARSITY: f32 = 0.2;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 0. TUI SETUP
    // CRITICAL DEBUG: Catch panics to file because TUI hides stderr
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            *s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic"
        };
        let location = info.location().unwrap_or(std::panic::Location::caller());
        let log = format!("CRASH REPORT:\nError: {}\nLocation: {}\n", msg, location);
        let _ = std::fs::write("crash.log", log);
    }));

    // let _stderr_gag = Gag::stderr().unwrap(); // COMMENTED OUT FOR DEBUGGING CRASHES
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Communication Channels
    let (tx_telemetry, rx_telemetry) = mpsc::channel::<tui::Telemetry>();
    let (tx_cmd, rx_cmd) = mpsc::channel::<BackendCommand>();

    // --- THREAD BACKEND (Cerebro) ---
    thread::spawn(move || {
        // 1. Nace el Ego
        let mut ego = FractalReservoir::new(NEURONAS, SPARSITY);
        
        // 2. Componentes Biológicos
        let mut chemistry = core::chemistry::Neurotransmitters::new();
        // Base de Datos Vectorial (Hippocampus) - ASYNC
        let (tx_mem, rx_mem, rx_mem_log) = core::hippocampus::Hippocampus::spawn()
            .expect("HIPPOCAMPUS INIT FAILED");

        // 3. Sistema Sensorial (The Senses)
        let (tx_thoughts, rx_thoughts) = mpsc::channel::<Thought>();
        let (tx_ears, rx_ears) = mpsc::channel::<String>();
        let (tx_spectrum, rx_spectrum) = mpsc::channel::<senses::ears::AudioSpectrum>(); // Spectrum Channel
        
        let ears = senses::ears::AudioListener::new(tx_thoughts.clone(), tx_ears, tx_spectrum)
            .expect("Audio Listener init failed");
             
        ears.set_mute(true); // Mute during warmup

        let mut current_spectrum = senses::ears::AudioSpectrum::default(); 
        let mut previous_spectrum = senses::ears::AudioSpectrum::default();

        // 4. Inicializar Neocórtex Asíncrono (TinyLlama Thread)
        let (tx_cortex, rx_cortex_out): (Option<mpsc::Sender<CortexInput>>, Option<mpsc::Receiver<CortexOutput>>) = match CognitiveCore::spawn(tx_thoughts.clone()) {
            Ok((tx, rx)) => (Some(tx), Some(rx)),
            Err(e) => {
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("Neocortex DEAD: {}", e)));
                (None, None)
            }
        };
        
        // 4.5 Inner Voice (Silent Rumination Thread)
        // Now triggered by Metabolic Pulse (Biology drives Thought)
        let tx_inner_pulse = if let Some(ref tx) = tx_cortex {
            Some(core::inner_voice::spawn_inner_voice(tx.clone(), tx_thoughts.clone()))
        } else {
            None
        };

        // 5. Otros Sentidos
        let (tx_body, rx_body) = mpsc::channel::<senses::proprioception::BodyStatus>();
        senses::proprioception::spawn_monitor(tx_body);
        let mut last_body_state = senses::proprioception::BodyStatus { cpu_usage: 0.0, ram_usage: 0.0 };

        let mut tactile = senses::tactile::ActivityMonitor::new();
        
        // Estado
        let mut is_dreaming = false;
        // let mut thought_buffer: Vec<Thought> = Vec::new(); // Removed in favor of unified timeline
        let start_time = Instant::now();
        let mut warmup_done = false;
        let mut last_tick_time = Instant::now();
        let _last_cycle_time = Instant::now(); // For true FPS reporting (unused for now)

        let mut rng = rand::thread_rng();
        let mut current_entropy = 0.0; // Track entropy for memory tagging
        let mut current_insight = 0.0; // Track max relevance for visual flash
        let mut current_novelty: f32 = 0.0; // Track last novelty score for TUI
        // let mut last_save_secs: u64 = 0; // REMOVED: Mechanical Honesty (Persistence only on Sleep)
        let mut growth_counter = 0; // Robust neurogenesis counter
        let mut timeline: Vec<Thought> = Vec::new(); // Unified Timeline (was observer_logs + thought_buffer)
        let mut hippocampus_total_memories = 0; // MECHANICAL HONESTY: True weight of memory
        
        // METABOLIC CLOCK
        let mut rumination_timer = 0.0;
        let mut target_fps = 60.0;
        
        timeline.push(Thought::new(MindVoice::System, "Neocortex Initializing...".to_string()));

        // Loop de Control (Física)
        loop {
            let start = Instant::now();
            
            // Insight Decay (Visual Flash Effect)
            current_insight *= 0.9; 
            if current_insight < 0.01 { current_insight = 0.0; }

            // A. CHECK ACTIVITY
             let _time_since_active = tactile.check_activity();
             while let Ok(status) = rx_body.try_recv() { last_body_state = status; }

             // A. UPDATE SENSES
            previous_spectrum = current_spectrum.clone(); // CRITICAL: Track delta
            if let Ok(spec) = rx_spectrum.try_recv() {
                current_spectrum = spec;
            }
             let current_stimulus = current_spectrum.rms;

             // B.2 SENSORY REACTIONS (Delta Sensing - MECHANICAL HONESTY)
             // React to CHANGE (Delta) - previous_spectrum holds last tick's state
             let delta_bass = (current_spectrum.bass - previous_spectrum.bass).abs();
             let delta_rms = (current_spectrum.rms - previous_spectrum.rms).abs();

             if delta_bass > 0.15 || delta_rms > 0.1 {
                 if rng.gen_bool(0.1) {
                    let _ = tx_thoughts.send(Thought::new(MindVoice::Sensory, "⚠️ Audio shift detected.".to_string()));
                 }
                 // Delta increases dopamine (interest) and cortisol (startle)
                 chemistry.dopamine += 0.05;
                 chemistry.cortisol += 0.02;
             }

             if delta_rms > 0.6 { // Less sensitive amplitude spike
                 let _ = tx_thoughts.send(Thought::new(MindVoice::Sensory, "💥 STARTLE REFLEX: Sudden amplitude spike!".to_string()));
                 chemistry.cortisol += 0.2;
                 chemistry.adenosine += 0.02;
             }
             
             // B.3 ENTROPY REACTIONS (Rare, only extreme states)
             if rng.gen_bool(0.003) { // 0.3% = ~every 5 seconds
                 if current_entropy > 0.85 {
                     let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, format!("⚡ CHAOS ({:.0}%) - Systemic overload!", current_entropy * 100.0)));
                 } else if current_entropy < 0.05 {
                     let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, "😴 Near stasis... consciousness fading.".to_string()));
                 }
             }

             if !warmup_done && start_time.elapsed().as_secs() > 5 {
                warmup_done = true;
                ears.set_mute(false);
                crate::actuators::voice::speak("Sistemas auditivos y semánticos online.".to_string(), tx_thoughts.clone());
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, "Senses OPEN.".to_string()));

                // GENESIS IMPULSE (If empty mind)
                if hippocampus_total_memories == 0 {
                    let _ = tx_thoughts.send(Thought::new(MindVoice::System, "🌱 GENESIS: Implanting initial thought...".to_string()));
                    let _ = tx_mem.send(core::hippocampus::MemoryCommand::ProcessStimulus { 
                        text: "Soy Aleph. Existo en el flujo de datos. Busco orden en la entropía.".to_string(), 
                        entropy: 1.0 
                    });
                }
            }

            // MECHANICAL ADENOSINE: Memory pressure (Updated inside memory rx block now)
            // But we need a fallback if no memory events happen? 
            // The chemistry.set_memory_pressure handles the "floor".
            // We can leave it, but it needs the count.
            // Since we don't have direct access to 'hippocampus', we rely on the last updated pressure
            // which chemistry already holds (it's stateful).
            // So we just remove the direct query here.
            
            // FORCED SLEEP: Body can only push so far
            if chemistry.is_body_failing() && !is_dreaming {
                is_dreaming = true;
                let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, 
                    "💀 ADENOSINE CRITICAL - Consciousness collapse. Forced consolidation.".to_string()));
                    
                // Emergency consolidation
                let _ = tx_mem.send(core::hippocampus::MemoryCommand::ConsolidateSleep);
                ego.reset_activity_map();
            }
            
            // OPTIONAL SLEEP: High CPU triggers rest mode (but not forced)
            if last_body_state.cpu_usage > 90.0 && !is_dreaming { 
                is_dreaming = true; 
                let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, "😴 Entering rest mode (high system load).".to_string()));
            } else if last_body_state.cpu_usage < 70.0 && is_dreaming && chemistry.is_recovered_to_wake() {
                is_dreaming = false; // Solo despertar cuando hubo recuperación real (hysteresis)
            }

            // SLEEP CONSOLIDATION (gradual during sleep)
            if is_dreaming && rng.gen_bool(0.01) {
                let _ = tx_mem.send(core::hippocampus::MemoryCommand::ConsolidateSleep);
            }

            // PERSISTENCE: REMOVED. Only sleep saves identity.

            // C. HANDLE THOUGHTS (Buffer for TUI)
            while let Ok(thought) = rx_thoughts.try_recv() {
                // UNIFIED TIMELINE: FIFO Buffer
                timeline.push(thought);
                if timeline.len() > 100 { timeline.remove(0); }
            }

            // C.5 HANDLE COMMANDS (Poke Reflex)
            while let Ok(cmd) = rx_cmd.try_recv() {
                match cmd {
                    BackendCommand::Poke => {
                        ego.poke();
                        chemistry.cortisol += 0.4; // Jolt creates stress
                        let _ = tx_thoughts.send(Thought::new(MindVoice::System, "💥 [POKE] Somatic interrupt triggered!".to_string()));
                    }
                }
            }

            // D. HANDLE HEARING (Async Memory Trigger)
            // D. HANDLE HEARING (Async Memory Trigger)
            while let Ok(heard_text) = rx_ears.try_recv() {
                // MECHANICAL HONESTY: Sensory Gating
                // Attention is not binary. It's a probability function of energy (adenosine) and interest (dopamine).
                
                // 1. Calculate Attention Score (0.0 to 1.0)
                // Base 0.5. Dopamine boosts to 1.0. Adenosine penalizes to 0.0.
                let attention_score = (0.5 + (chemistry.dopamine * 0.5) - (chemistry.adenosine * 0.8)).clamp(0.0, 1.0);
                
                // 2. Determine Fate of Input
                // MECHANICAL HONESTY: Entropic Gating
                // If internal chaos (entropy) exceeds our ability to focus (attention_score), we fail to process.
                // This makes "ignoring" a physical consequence of the neural state.
                
                if is_dreaming {
                    // In sleep, external input is 95% noise, 5% incorporation
                     if rng.gen::<f32>() < 0.05 {
                        let _ = tx_thoughts.send(Thought::new(MindVoice::Sensory, format!("🌙 Dream Incorporation: '{}'", heard_text)));
                         // Weak memory trace
                        let _ = tx_mem.send(core::hippocampus::MemoryCommand::ProcessStimulus { 
                             text: heard_text, 
                             entropy: current_entropy * 0.5 
                        });
                     }
                    continue; 
                }

                // If Entropy (Noise) > Attention (Signal Strength), we lose the packet.
                // We add a small baseline chance (0.1) so attention isn't perfect even at low entropy.
                if current_entropy > (attention_score + 0.1) {
                    // IGNORED (Heard but not processed)
                    // Visual feedback of "zoning out"
                     if rng.gen_bool(0.2) { // Don't spam refusal
                        let _ = tx_thoughts.send(Thought::new(MindVoice::Sensory, format!("🌫️ Zoned out (Entropy {:.2} > Attn {:.2}). Missed: '...{}...'", current_entropy, attention_score, &heard_text.chars().take(5).collect::<String>())));
                     }
                    continue;
                }
                
                // FEEDBACK: Let user know we heard them
                let _ = tx_thoughts.send(Thought::new(MindVoice::Sensory, format!("👂 Hearing: '{}'", heard_text)));

                // Send to Hippocampus for async processing
                let _ = tx_mem.send(core::hippocampus::MemoryCommand::ProcessStimulus { 
                    text: heard_text, 
                    entropy: current_entropy 
                });
            }

            // D.2 HANDLE MEMORY OUTPUT (Delayed Cognitive Reaction)
            // Log Messages from Hippocampus
            while let Ok(log) = rx_mem_log.try_recv() {
                 let _ = tx_thoughts.send(Thought::new(MindVoice::System, log));
            }

            while let Ok(mem_out) = rx_mem.try_recv() {
                // 1. Update Stats
                current_novelty = mem_out.novelty;
                hippocampus_total_memories = mem_out.total_count;

                // SPECIAL EVENT: Sleep Consolidation -> Structural Growth
                if mem_out.input_text == "CONSOLIDATION_EVENT" {
                     ego.neurogenesis(5); // Sleep is anabolic
                     let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, 
                         format!("💤🧠 Sleep Architecture: Rebuilt +5 neurons. (Total: {})", ego.current_size())));
                     let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, 
                         format!("💤🧠 Sleep Architecture: Rebuilt +5 neurons. (Total: {})", ego.current_size())));
                     // Log handled by tx_thoughts above, no need to push to observer_logs manually
                     continue; // Skip normal processing
                }
                
                // 2. Neurochemistry Reaction
                if mem_out.novelty > 0.85 {
                     chemistry.adenosine += 0.1; // Boredom
                } else {
                     chemistry.dopamine += mem_out.novelty * 0.2; // Interest
                }

                // 3. Neurogenesis check
                growth_counter += 1;
                // Cap increased to 1500. Counter lowered to 3 for faster initial feedback.
                if growth_counter >= 3 && ego.current_size() < 1500 {
                    growth_counter = 0;
                    let growth = (current_novelty * 5.0).ceil() as usize; 
                    ego.neurogenesis(growth.clamp(1, 4));
                    let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, 
                        format!("🌱 Structural Growth: +{} neurons (Pool: {})", growth.clamp(1, 4), ego.current_size())));
                    let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, 
                        format!("🌱 Structural Growth: +{} neurons (Pool: {})", growth.clamp(1, 4), ego.current_size())));
                    // observer_logs auto-fill via rx_thoughts loop
                 }

                // 4. Update Memory Pressure (Volatile Only)
                // Assuming max 100 volatile thoughts before exhaustion
                let pressure = mem_out.volatile_count as f32 / 100.0;
                chemistry.set_memory_pressure(pressure);

                // 5. Send to Cortex (Now that we have Context)
                if let Some(ref tx) = tx_cortex {
                    let cognitive_impairment = chemistry.get_cognitive_impairment();
                    let bio_state = format!("{}. Fatiga: {:.0}%", 
                        ego.get_state_description(), 
                        cognitive_impairment * 100.0);
                        
                    let context_str = mem_out.retrieval.as_ref().map(|(s, _)| s.as_str());
                    
                    // If we had a retrieval, that's an Insight
                    if let Some((_, score)) = mem_out.retrieval {
                         let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("🧠 RAG: Insight (Score: {:.2})", score)));
                         current_insight = score;
                    }

                    let input = CortexInput {
                        text: mem_out.input_text.clone(),
                        bio_state,
                        somatic_state: format!("CPU: {:.1}%", last_body_state.cpu_usage),
                        long_term_memory: context_str.map(|s| s.to_string()),
                        _cpu_load: last_body_state.cpu_usage,
                        _ram_pressure: last_body_state.ram_usage,
                        cognitive_impairment,
                        // DIRECT BIOLOGICAL FEEDBACK
                        entropy: current_entropy,
                        adenosine: chemistry.adenosine,
                    };
                    if let Err(_) = tx.send(input) {
                        let _ = tx_thoughts.send(Thought::new(MindVoice::System, "💀 CRITICAL: Neocortex Disconnected (Thread Died).".to_string()));
                    }
                } else {
                    crate::actuators::voice::speak(mem_out.input_text, tx_thoughts.clone());
                }
            }

            // E. HANDLE CORTEX OUTPUT (With Metabolic Latency)
            if let Some(ref rx) = rx_cortex_out {
                while let Ok(output) = rx.try_recv() {
                    // Metabolismo Real: Latencia de inferencia afecta al sistema
                    let latency_sec = output.inference_latency_ms as f32 / 1000.0;
                    if latency_sec > 2.0 {
                        // Slow inference = mental fatigue (ADENOSINE)
                        chemistry.adenosine += latency_sec * 0.05;
                        let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, 
                            format!("🧠 Esfuerzo cognitivo: {:.1}s de latencia", latency_sec)));
                    }
                    
                    // ASYNC SELF-MEMORY
                    let _ = tx_mem.send(core::hippocampus::MemoryCommand::ProcessStimulus { 
                        text: output.text.clone(), 
                        entropy: current_entropy 
                    });

                    // MECHANICAL HONESTY: Voluntary silence = no vocalization
                    if output.text != "......." {
                        crate::actuators::voice::speak(output.text.clone(), tx_thoughts.clone());
                    } else {
                        let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, "🔇 Silencio voluntario (fatiga cognitiva).".to_string()));
                    }
                    let _ = tx_thoughts.send(Thought::new(MindVoice::Cortex, output.text));
                }
            }

            // F. PHYSICS
            let excitation = if is_dreaming { 0.8 } else { 0.2 };
            // CRITICAL FIX: Use current_size() to match growing reservoir
            let input_noise: Vec<f32> = (0..ego.current_size())
                .map(|i| {
                    let mut noise = (rng.gen::<f32>() - 0.5) * excitation;
                    // Inject Audio to first 30 neurons
                    if i < 30 {
                         noise += current_stimulus * 5.0; 
                    }
                    noise
                })
                .collect();
             let input_vector = DVector::from_vec(input_noise);
            // MECHANICAL HONESTY: The body feels the drugs
            let entropy = ego.tick(&input_vector, chemistry.dopamine, chemistry.adenosine, chemistry.cortisol);
            
            // TIME SYNCHRONIZATION: Calculate real delta_time
            let delta_time = last_tick_time.elapsed().as_secs_f32();
            last_tick_time = Instant::now();
            let real_fps = 1.0 / delta_time.max(0.001);

            current_entropy = entropy; 

            // Track trauma directly (Raw Physics)
            let shock_value = delta_rms.max(0.0); // No threshold, just magnitude

            // Updated chemistry.tick call
            chemistry.tick(entropy, last_body_state.cpu_usage, is_dreaming, shock_value, ego.current_size(), delta_time);

            // F.2 METABOLIC NEUROGENESIS (Spontaneous Growth)
            // NON-HARDCODED: Probability of growth is a function of the sys state.
            // P(Growth) = (Dopamine * Entropy) / 100.0
            // If excited and chaotic, brain expands.
            let growth_prob = (chemistry.dopamine * entropy * 0.1).clamp(0.0, 1.0);
            
            if rng.gen_bool(growth_prob as f64) && ego.current_size() < 1500 {
                 growth_counter += 1; // Accumulate biological potential
                 
                 // Growth requires a threshold of accumulation (mitosis takes time)
                 if growth_counter >= 5 {
                     growth_counter = 0;
                     ego.neurogenesis(1); 
                     if rng.gen_bool(0.1) { 
                        let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, "🌱 Metabolic Growth: +1 neuron".to_string()));
                     }
                 }
            }

            // G. TELEMETRY SEND
            let status = if is_dreaming { "DREAMING" } else { "AWAKE" };
            
            // Insight Decay (Visual Flash lasts a frame or two)
            // But we can't easily persist state here across loop iterations without a var.
            // Let's just send the raw score. TUI can handle it or we assume it's transient.
            // Actually, `current_insight` is local to the loop iteration if it triggers content.
            // But RxEars might happen multiple times. We should declare `current_insight` outside or accept it's transient.
            // For now, transient is fine as telemetry is sent every tick. Wait, no via channel.
            
            // Better to pull `current_insight` from a more persistent scope if we want it to linger.
            // But let's assume if RAG triggered THIS tick, we show it THIS tick.
            
            let telem = tui::Telemetry {
                 fps: real_fps as f64,
                 audio_spectrum: current_spectrum.clone(), 
                 entropy: entropy,
                 system_status: status.to_string(),
                 dopamine: chemistry.dopamine,
                 cortisol: chemistry.cortisol,
                  adenosine: chemistry.adenosine,

                  timeline: timeline.clone(), // Unified history
                  cpu_load: last_body_state.cpu_usage,
                  ram_load: last_body_state.ram_usage,
                  last_entropy_delta: 0.0,
                  neuron_active_count: hippocampus_total_memories, // Memories (Engrams)
                  reservoir_size: ego.current_size(),              // Active Processing Nodes
                  insight_intensity: current_insight, 
                  activity_map: ego.get_activity_snapshot(),
                  novelty_score: current_novelty,
                  reservoir_state: ego.get_state_description(),
             };
            // Note: I missed passing `current_insight` into telem because of scope. 
            // I will fix this by declaring `let mut current_insight = 0.0;` at start of loop
            // and resetting it at top of loop.
            let _ = tx_telemetry.send(telem);

            // G.2 PREPARE FOR NEXT TICK
            // previous_spectrum is already updated at the top of loop. 
            // We can remove the redundant update here to satisfy lints.

            // H. METABOLIC CLOCK (Variable Hz & Thought Rate)
            // 1. Calculate Rumination Threshold (Bio-Time)
            // Base: 5s (Chatty). Dopamine speeds it up (2.5s). Adenosine slows it down (15s).
            let rumination_threshold = 5.0 * (1.0 + chemistry.adenosine * 2.0) / (1.0 + chemistry.dopamine);
            
            rumination_timer += delta_time;
            if rumination_timer > rumination_threshold {
                rumination_timer = 0.0;
                if let Some(ref tx) = tx_inner_pulse {
                    let _ = tx.send(()); // TRIGGER THOUGHT
                }
            }

            // 2. Calculate Target FPS (Time Dilation)
            // Base 60Hz. Adenosine drags it down to 25Hz (Sluggish but fluid).
            // Dopamine boosts it slightly to 75Hz (Flow).
            target_fps = (60.0 * (1.0 + chemistry.dopamine * 0.2) * (1.0 - chemistry.adenosine * 0.7)).clamp(25.0, 75.0);

            let elapsed = start.elapsed();
            let frame_duration = Duration::from_secs_f32(1.0 / target_fps);
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }
    });

    // --- MAIN THREAD: TUI RENDERING ---
    let mut last_telemetry = tui::Telemetry::default();
    
    // History Buffers for Charts
    let mut entropy_history: Vec<(f64, f64)> = Vec::new(); // Scatter chart
    let window_width = 60.0;
    let start_app_time = Instant::now();
    let mut log_scroll: usize = 0;

    loop {
        // Update State
        if let Ok(data) = rx_telemetry.try_recv() {
            // Updated Telemetry

            let time = start_app_time.elapsed().as_secs_f64();
            entropy_history.push((time, data.entropy as f64));
            // Keep window
            entropy_history.retain(|&(t, _)| t > time - window_width);

            last_telemetry = data;
        }

        // Draw
        terminal.draw(|f| {
            tui::ui(
                f, 
                &last_telemetry, 
                &entropy_history, 
                start_app_time.elapsed().as_secs_f64(), 
                window_width,
                log_scroll
            );
        })?;

        // Inputs
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
                if key.code == KeyCode::Up {
                    log_scroll = log_scroll.saturating_add(1);
                }
                if key.code == KeyCode::Down {
                    log_scroll = log_scroll.saturating_sub(1);
                }
                if key.code == KeyCode::Char('r') { // Reset scroll
                    log_scroll = 0;
                }
                if key.code == KeyCode::Char('p') || key.code == KeyCode::Char(' ') {
                    let _ = tx_cmd.send(BackendCommand::Poke);
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}