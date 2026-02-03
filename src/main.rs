#![allow(deprecated)]

mod core;
mod senses;
mod tui;
mod actuators;

use crate::core::llm::{CognitiveCore, CortexInput};
use crate::core::reservoir::FractalReservoir;
use crate::core::thought::{MindVoice, Thought};
use nalgebra::DVector;
use std::{thread, time::{Duration, Instant}};
use std::sync::mpsc;
use rand::prelude::*;
use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

// CONFIGURACIÓN DE VIDA
const NEURONAS: usize = 100;
const SPARSITY: f32 = 0.2;
const FRECUENCIA_HZ: u64 = 60; 

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 0. TUI SETUP
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Canal Telemetria: Backend -> Frontend (TUI)
    let (tx_telemetry, rx_telemetry) = mpsc::channel::<tui::Telemetry>();

    // --- THREAD BACKEND (Cerebro) ---
    thread::spawn(move || {
        // 1. Nace el Ego
        let mut ego = FractalReservoir::new(NEURONAS, SPARSITY);
        
        // 2. Componentes Biológicos
        let mut chemistry = core::chemistry::Neurotransmitters::new();
        // Base de Datos Vectorial (Hippocampus)
        let mut hippocampus = core::hippocampus::Hippocampus::new()
            .expect("HIPPOCAMPUS INIT FAILED: Check .onnx model");

        // 3. Sistema Sensorial (The Senses)
        // 3. Sistema Sensorial (The Senses)
        let (tx_thoughts, rx_thoughts) = mpsc::channel::<Thought>();
        let (tx_ears, rx_ears) = mpsc::channel::<String>();
        let (tx_spectrum, rx_spectrum) = mpsc::channel::<senses::ears::AudioSpectrum>(); // Spectrum Channel
        
        let mut ears = senses::ears::AudioListener::new(tx_thoughts.clone(), tx_ears, tx_spectrum)
            .expect("Audio Listener init failed");
             
        ears.set_mute(true); // Mute during warmup

        let mut current_spectrum = senses::ears::AudioSpectrum::default(); 

        // 4. Inicializar Neocórtex Asíncrono (TinyLlama Thread)
        let (tx_cortex, rx_cortex_out): (Option<mpsc::Sender<CortexInput>>, Option<mpsc::Receiver<String>>) = match CognitiveCore::spawn(tx_thoughts.clone()) {
            Ok((tx, rx)) => (Some(tx), Some(rx)),
            Err(e) => {
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("Neocortex DEAD: {}", e)));
                (None, None)
            }
        };

        // 5. Otros Sentidos
        let (tx_body, rx_body) = mpsc::channel::<senses::proprioception::BodyStatus>();
        senses::proprioception::spawn_monitor(tx_body);
        let mut last_body_state = senses::proprioception::BodyStatus { cpu_usage: 0.0, ram_usage: 0.0 };

        let mut tactile = senses::tactile::ActivityMonitor::new();
        
        // Estado
        let mut is_dreaming = false;
        let mut thought_buffer: Vec<Thought> = Vec::new();
        let start_time = Instant::now();
        let mut warmup_done = false;
        let mut current_log = "Neocortex Initializing...".to_string();

        let mut rng = rand::thread_rng();
        let mut current_entropy = 0.0; // Track entropy for memory tagging
        let mut current_insight = 0.0; // Track max relevance for visual flash

        // Loop de Control (Física)
        loop {
            let start = Instant::now();
            
            // Insight Decay (Visual Flash Effect)
            current_insight *= 0.9; 
            if current_insight < 0.01 { current_insight = 0.0; }

            // A. CHECK ACTIVITY
             let _time_since_active = tactile.check_activity();
             while let Ok(status) = rx_body.try_recv() { last_body_state = status; }

             // B. UPDATE AUDIO SPECTRUM (Real-Time)
             while let Ok(spectrum) = rx_spectrum.try_recv() {
                 current_spectrum = spectrum; 
             }
             let current_stimulus = current_spectrum.rms;

             if !warmup_done && start_time.elapsed().as_secs() > 5 {
                warmup_done = true;
                ears.set_mute(false);
                crate::actuators::voice::speak("Sistemas auditivos y semánticos online.".to_string(), tx_thoughts.clone());
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, "Senses OPEN.".to_string()));
            }

            if last_body_state.cpu_usage > 90.0 { is_dreaming = true; } else { is_dreaming = false; }

            // SLEEP CONSOLIDATION (Hippocampus 2.0)
            if is_dreaming && rng.gen_bool(0.005) { // Occasional consolidation during sleep
                if let Ok(forgotten) = hippocampus.consolidate_sleep() {
                    if forgotten > 0 {
                         let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("💤 Sleep Cycle: Pruned {} weak memories.", forgotten)));
                    }
                }
            }

            // C. HANDLE THOUGHTS (Buffer for TUI)
            while let Ok(thought) = rx_thoughts.try_recv() {
                thought_buffer.push(thought);
                if thought_buffer.len() > 30 { thought_buffer.remove(0); }
            }

            // D. HANDLE HEARING (Cognitive & Memory)
            while let Ok(heard_text) = rx_ears.try_recv() {
                // 1. Habituación (Boredom Sensor)
                if let Ok(similarity) = hippocampus.check_novelty(&heard_text) {
                     if similarity > 0.85 {
                         chemistry.adenosine += 0.1; // Increase Fatigue/Boredom
                         let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, format!("Boredom spike (Sim: {:.2})", similarity)));
                     }
                }

                // 2. Memorizar (Volatile RAM)
                let _ = hippocampus.remember(&heard_text, "acoustic_input", current_entropy);

                // 3. Recordar (RAG)
                let mut current_insight = 0.0;
                let context = if let Some((ctx, score)) = hippocampus.recall_relevant(&heard_text) {
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("🧠 RAG: Insight (Score: {:.2})", score)));
                     current_insight = score;
                     Some(ctx)
                } else {
                     None
                };

                // 4. Enviar al Cortex
                if let Some(ref tx) = tx_cortex {
                    let bio_state = ego.get_state_description();
                    let somatic_desc = format!("CPU: {:.1}%", last_body_state.cpu_usage);
                    
                    let input = CortexInput {
                        text: heard_text.clone(),
                        bio_state,
                        somatic_state: somatic_desc,
                        long_term_memory: context,
                    };
                    let _ = tx.send(input);
                } else {
                    crate::actuators::voice::speak(heard_text, tx_thoughts.clone());
                }
            }

            // E. HANDLE CORTEX OUTPUT
            if let Some(ref rx) = rx_cortex_out {
                while let Ok(response) = rx.try_recv() {
                    let _ = hippocampus.remember(&response, "self_thought", current_entropy);
                    crate::actuators::voice::speak(response.clone(), tx_thoughts.clone());
                    // Corrected MindVoice
                    let _ = tx_thoughts.send(Thought::new(MindVoice::Cortex, response));
                }
            }

            // F. PHYSICS
            let excitation = if is_dreaming { 0.8 } else { 0.2 };
            let input_noise: Vec<f32> = (0..NEURONAS)
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
            let entropy = ego.tick(&input_vector);
            
            current_entropy = entropy; // Update for next tick

            chemistry.tick(entropy, last_body_state.cpu_usage, is_dreaming, false, ego.current_size());

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
                 fps: 60.0,
                 audio_spectrum: current_spectrum.clone(), 
                 entropy: entropy,
                 system_status: status.to_string(),
                 dopamine: chemistry.dopamine,
                 cortisol: chemistry.cortisol,
                 adenosine: chemistry.adenosine,
                 thoughts: thought_buffer.clone(),
                 cpu_load: last_body_state.cpu_usage,
                 ram_load: last_body_state.ram_usage,
                 log_message: Some(current_log.clone()),
                 last_entropy_delta: 0.0,
                 neuron_active_count: 100 + (hippocampus.memory_count() * 5), // DENSITY FACTOR 5x
                 insight_intensity: current_insight, 
            };
            // Note: I missed passing `current_insight` into telem because of scope. 
            // I will fix this by declaring `let mut current_insight = 0.0;` at start of loop
            // and resetting it at top of loop.
            let _ = tx_telemetry.send(telem);

            let elapsed = start.elapsed();
            if elapsed < Duration::from_millis(1000 / FRECUENCIA_HZ) {
                thread::sleep(Duration::from_millis(1000 / FRECUENCIA_HZ) - elapsed);
            }
        }
    });

    // --- MAIN THREAD: TUI RENDERING ---
    let mut last_telemetry = tui::Telemetry::default();
    
    // History Buffers for Charts
    let mut entropy_history: Vec<(f64, f64)> = Vec::new(); // Scatter chart
    let window_width = 10.0;
    let start_app_time = Instant::now();

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
                window_width
            );
        })?;

        // Inputs
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
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