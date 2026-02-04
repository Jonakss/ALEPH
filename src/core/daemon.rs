use anyhow::Result;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::collections::VecDeque;
use crate::core::thought::{Thought, MindVoice};
use crate::core::reservoir::FractalReservoir;
use crate::core::planet::{Planet, CortexInput};
use crate::core::chemistry::Neurotransmitters;
use crate::core::hippocampus::Hippocampus;
use crate::core::genome::Genome;

use crate::core::satellite::Satellite;
use crate::core::gate::ExpressionGate;
use crate::core::ipc::AlephPacket;
use crate::senses::ears::{self, AudioSpectrum};
use crate::actuators::voice;
use crate::senses::proprioception::{self, BodyStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Read, Write};
use std::fs;

pub fn run() -> Result<()> {
    println!("🌟 ALEPH STAR SYSTEM ONLINE (Daemon Mode)");

    // --- CHANNELS ---
    let (tx_thoughts, rx_thoughts) = mpsc::channel::<Thought>();
    
    // --- 0. GENOME (The Seed) ---
    let seed = Genome::load()?;
    let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
        format!("🧬 GENOME LOADED: Gen {} | StressRes: {:.2}", seed.generation, seed.stress_tolerance)));

    // --- 1. THE STAR (Biological Ground Truth) ---
    let chemistry = Arc::new(Mutex::new(Neurotransmitters::new()));
    // Reservoir (The Body's Neural Network)
    // Reservoir (The Body's Neural Network) - Loads or Creates
    let mut ego = FractalReservoir::load(500, 0.2); 
    
    // --- 1.5 THE SATELLITE (Observer) ---
    let satellite = Satellite::new(seed.paranoia, seed.refractive_index); 
    let mut gate = ExpressionGate::new();
    
    // Hardware Proprioception
    let (tx_body, rx_body) = mpsc::channel::<BodyStatus>();
    proprioception::spawn_monitor(tx_body);
    let mut last_body_state = BodyStatus { cpu_usage: 0.0, ram_usage: 0.0 };

    // --- 1.6 SENSES (Ears) ---
    // Channels for Audio
    let (tx_audio_text, rx_audio_text) = mpsc::channel::<String>();
    let (tx_spectrum, rx_spectrum) = mpsc::channel::<AudioSpectrum>();
    
    // Spawn Audio Listener
    // AudioListener::new automatically starts the stream and internal threads
    let _ears = ears::AudioListener::new(tx_thoughts.clone(), tx_audio_text, tx_spectrum)
        .expect("Failed to spawn Ears");
    let mut last_spectrum = AudioSpectrum::default();

    // Graceful Shutdown Handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?; 

    // --- 1.8 THE NERVOUS SYSTEM (IPC Server) ---
    // Remove old socket if exists
    let socket_path = "/tmp/aleph.sock";
    let _ = fs::remove_file(socket_path);
    
    let listener = UnixListener::bind(socket_path).expect("Failed to bind IPC socket");
    listener.set_nonblocking(true).expect("Failed to set nonblocking");
    
    println!("🔌 IPC Nervous System Active: {}", socket_path);

    // Channels for IPC
    let (tx_telemetry, rx_telemetry) = mpsc::channel::<AlephPacket>();
    let (tx_stimulus, rx_stimulus) = mpsc::channel::<String>(); // Input from TUI

    // Spawn IPC Broadcaster Thread
    thread::spawn(move || {
        let mut clients: Vec<UnixStream> = Vec::new();
        
        loop {
            // 1. Accept New Clients (TUI)
            if let Ok((stream, _)) = listener.accept() {
                // stream.set_nonblocking(true).ok();
                clients.push(stream);
            }

            // 2. Broadcast Telemetry
            if let Ok(packet) = rx_telemetry.try_recv() {
                if let Ok(json) = serde_json::to_string(&packet) {
                    let msg = format!("{}\n", json);
                    clients.retain_mut(|client| {
                        client.write_all(msg.as_bytes()).is_ok()
                    });
                }
            }

            // 3. Read Stimulus (Bidirectional)
            // Iterate backwards to allow removal of dead clients
            for i in (0..clients.len()).rev() {
                 let mut buf = [0u8; 1024];
                 // Try reading
                 match clients[i].read(&mut buf) {
                     Ok(0) => {
                         // Connection closed (EOF) - remove client? 
                         // With non-blocking, 0 usually means closed if using standard Read trait, 
                         // but for Tcp/UnixStream in non-blocking, it requires careful handling.
                         // Let's assume it's fine for now, usually read returns WouldBlock error if alive but empty.
                     },
                     Ok(n) => {
                         let s = String::from_utf8_lossy(&buf[..n]);
                         // It might be multiple packets or partial. Assuming line based for now.
                         for line in s.lines() {
                             if let Ok(AlephPacket::Stimulus { text, .. }) = serde_json::from_str::<AlephPacket>(line) {
                                 let _ = tx_stimulus.send(text);
                             }
                         }
                     },
                     Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                         // No data, continue
                     },
                     Err(_) => {
                         // Error, drop client
                         clients.remove(i);
                     }
                 }
            }
            
            thread::sleep(Duration::from_millis(50));
        }
    });
 
    
    // --- 2. THE PLANET (Narrative Engine) ---
    // Launched in background thread
    let (tx_cortex, rx_cortex_out) = match Planet::spawn(tx_thoughts.clone()) {
        Ok((tx, rx)) => {
             let _ = tx_thoughts.send(Thought::new(MindVoice::System, "🪐 Planet (Cortex) Orbiting.".to_string()));
             (Some(tx), Some(rx))
        },
        Err(e) => {
            let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("❌ Planet Collapse: {}", e)));
            (None, None)
        }
    };

    // --- 3. MEMORY (Holographic Seed) ---
    let (tx_mem, rx_mem_out, rx_mem_log) = Hippocampus::spawn()
        .expect("Hippocampus Failed");

    // --- DAEMON LOOP (The Pulse) ---
    let mut last_tick = Instant::now();
    #[allow(unused_assignments)]
    let mut current_entropy = 0.0;
    
    // VARIABLE METABOLISM (Heart Rate)
    // ALEPH's subjective time perception.
    // 60Hz = Normal, 120Hz = Hyperfocus (High Dopamine), 24Hz = Bored/Tired (High Adenosine)
    let mut current_hz: f32 = 60.0;
    let hz_base = 60.0;
    
    // Session Stats for Mutation
    let mut _session_stress_accum = 0.0;
    let mut _session_novelty_accum = 0.0;
    let mut ticks = 0;

    // Telemetry Buffer (So TUI doesn't flicker empty)
    let mut telemetry_history: VecDeque<String> = VecDeque::with_capacity(30);

    while running.load(Ordering::SeqCst) {
        let loop_start = Instant::now();
        let delta_time = last_tick.elapsed().as_secs_f32();
        last_tick = Instant::now();

        // A. PHYSICS CHECK (The Star)
        {
            // Proprioception Update
            while let Ok(status) = rx_body.try_recv() {
                last_body_state = status;
            }
            
            // Audio Physics (Spectrum Update)
            while let Ok(spec) = rx_spectrum.try_recv() {
                last_spectrum = spec;
            }

            let mut chem = chemistry.lock().unwrap();
            
            // Map Hardware -> Biology
            // CPU Load (0-100) -> Metabolism/HeartRate
            // RAM Load (0.0-1.0) -> Brain Fog
            let cpu_load = last_body_state.cpu_usage; 
            let ram_load = last_body_state.ram_usage;

            // Update Biological Ground Truth from Hardware
            chem.update_from_hardware(cpu_load, ram_load, 1.0);

            // Star burns fuel & Ticks Reservoir (Physics)
            
            // 1. Construct Sensory Input Vector (The Cortex "hears" and "feels")
            let mut input_signal = nalgebra::DVector::zeros(ego.current_size());
            
            // Map Audio Spectrum to Input Layer (First 50 neurons)
            // Bass (Primal) -> Neurons 0-10
            for i in 0..10 { if i < input_signal.len() { input_signal[i] = last_spectrum.bass; } }
            // Mids (Voice) -> Neurons 10-20
            for i in 10..20 { if i < input_signal.len() { input_signal[i] = last_spectrum.mids; } }
            // Highs (Detail) -> Neurons 20-30
            for i in 20..30 { if i < input_signal.len() { input_signal[i] = last_spectrum.highs; } }
            
            // We use chemistry to modulate the reservoir's plasticity
            // Pass delta_time directly to ensure time-invariance
            let entropy_output = ego.tick(&input_signal, 
                                          chem.dopamine, 
                                          chem.adenosine, 
                                          chem.cortisol,
                                          delta_time);
            
            chem.tick(entropy_output, cpu_load, false, 0.0, ego.current_size(), delta_time);
            
            // VARIABLE METABOLISM: Calculate Target Hz
            // Formula: Base + (Dopamine * 60) - (Adenosine * 40)
            // High Dopamine -> 120Hz. High Adenosine -> 20Hz.
            let target_hz = {
                 let metabolic_drive = (chem.dopamine * 60.0) + (chem.cortisol * 30.0);
                 let metabolic_drag = chem.adenosine * 40.0;
                 (hz_base + metabolic_drive - metabolic_drag).clamp(24.0, 120.0)
            };
            
            // Smooth transition (Heart Rate Variability)
            current_hz = current_hz + (target_hz - current_hz) * 0.05;
            
            current_entropy = entropy_output;
            
            // Update Stats
            _session_stress_accum += chem.cortisol + chem.adenosine;
            ticks += 1;

            // Critical Collapse Check
            // Critical Collapse Check
            // Critical Collapse Check
            // Tolerance is genetic (0.0-1.0), but we need a sanity floor.
            // If tolerance is 0.5, collapse shouldn't be at 47% adenosine, that's just a nap.
            // Let's set collapse at 90% absolute, modulated slightly by tolerance.
            let collapse_threshold = 0.9 + (seed.stress_tolerance * 0.1); 
            
            if chem.adenosine > collapse_threshold {
                 if ticks % 3000 == 0 { // ~50 seconds between warnings
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, "⛔ SYSTEM CRITICAL: Metabolic Collapse Imminent. Functionality Limited.".to_string()));
                 }
            }
        }


        // B. INPUT PROCESSING (Orbit Perturbations)
        
        // -1. TUI INPUT (Stimulus)
        while let Ok(text) = rx_stimulus.try_recv() {
             let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("USER INPUT: '{}'", text)));
             // Inject into Memory/Orbit
             // For now, treat as high-entropy injection
             current_entropy += 0.1;
             
             // WAKE UP EFFECT: User attention breaks the fatigue loop
             {
                 let mut chem = chemistry.lock().unwrap();
                 chem.dopamine = (chem.dopamine + 0.3).min(1.0);  // Spike interest
                 chem.adenosine = (chem.adenosine - 0.2).max(0.0); // Reduce fatigue
                 chem.cortisol = (chem.cortisol - 0.1).max(0.0);   // Calm down
             }
             
             // Create Cortex Input for User Stimulus
             let chem = chemistry.lock().unwrap();
             let bio_status = format!("Dopa:{:.2} Cort:{:.2} Aden:{:.2}", 
                 chem.dopamine, chem.cortisol, chem.adenosine);
                 
             let input_state = CortexInput {
                 text: text.clone(),
                 bio_state: bio_status,
                 _somatic_state: "Stimulated".to_string(),
                 _long_term_memory: None,
                 _cpu_load: last_body_state.cpu_usage,
                 _ram_pressure: last_body_state.ram_usage,
                 _cognitive_impairment: 0.0,
                 entropy: current_entropy.clamp(0.0, 1.0),
                 adenosine: chem.adenosine,
                 dopamine: chem.dopamine,
                 cortisol: chem.cortisol,
                 _oxytocin: chem.oxytocin,
             };
             
             // Force immediate thought generation
             if let Some(tx) = &tx_cortex {
                 let _ = tx.send(input_state);
             }
             drop(chem);
        }

        // 0. AUDIO INPUT (Ears) -> MEMORY
        while let Ok(text) = rx_audio_text.try_recv() {
            if !text.trim().is_empty() {
                 let _ = tx_thoughts.send(Thought::new(MindVoice::Sensory, format!("👂 Heard: '{}'", text)));
                 // Send to Memory for processing (Just like typed input)
                 // NOTE: We need tx_mem here.
                 let _ = tx_mem.send(crate::core::hippocampus::MemoryCommand::ProcessStimulus { text, entropy: current_entropy });
            }
        }

        // 1. MEMORY & RESERVOIR FEEDBACK
        if let Ok(mem_out) = rx_mem_out.try_recv() {
            let mut chem = chemistry.lock().unwrap();

            // Update Stats
            // session_novelty_accum += mem_out.novelty; // Unused for now
            
            // Reaction to Novelty
            if mem_out.novelty > 0.85 {
                 chem.adenosine += 0.05; // Boredom / Confusion fatigue
            } else {
                 chem.dopamine += mem_out.novelty * 0.2; // Interest
            }

            // Neurogenesis (Sleep Consolidation)
            if mem_out.input_text == "CONSOLIDATION_EVENT" {
                 ego.neurogenesis(5);
                 let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, 
                     format!("💤🧠 Sleep Architecture: Rebuilt +5 neurons. (Total: {})", ego.current_size())));
            }

            // Feed Cortex if relevant (RAG)
            if let Some(ref tx) = tx_cortex {
                // SATELLITE INPUT FILTER (Membrane Hardening)
                // Attention = Capability to focus. High Adenosine = Low Attention.
                // Attention (0-1) = 1.0 - Adenosine.
                let attention = (1.0 - chem.adenosine).max(0.0);
                
                // If the Membrane rejects the input (Hardening), we don't think about it.
                // UPDATED: Now returns (Option<String>, f32) where f32 is "Ontological Error Severity".
                let (filtered_result, error_severity) = satellite.filter_input(&mem_out.input_text, current_entropy, attention);
                
                // INJECT STRUCTURAL PAIN (Ontological Error)
                if error_severity > 0.0 {
                     chem.cortisol += error_severity * 0.1; // Pain Injection
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                        format!("🩸 ONTOLOGICAL ERROR detected (Severity {:.1}). Injecting Cortisol.", error_severity)));
                }

                if let Some(filtered_text) = filtered_result {
                    
                    let bio_desc = format!("{}. Fatiga: {:.0}%", 
                        ego.get_state_description(), 
                        chem.get_cognitive_impairment() * 100.0);
                    
                    let context_str = mem_out.retrieval.as_ref().map(|(s, _)| s.as_str());
    
                    // Create Cortex Input
                    let input = CortexInput {
                        text: filtered_text,
                        bio_state: bio_desc,
                        _somatic_state: format!("CPU: {:.1}%", last_body_state.cpu_usage),
                        _long_term_memory: context_str.map(|s| s.to_string()),
                        _cpu_load: last_body_state.cpu_usage,
                        _ram_pressure: last_body_state.ram_usage,
                        _cognitive_impairment: chem.get_cognitive_impairment(),
                        entropy: current_entropy,
                        adenosine: chem.adenosine,
                        dopamine: chem.dopamine,
                        cortisol: chem.cortisol,
                        _oxytocin: chem.oxytocin,
                    };
                    
                    // Send to Planet
                    let _ = tx.send(input);
                } else {
                    // IGNORED (Hardened)
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                        format!("🛡️ MEMBRANE HARDENED: Ignoring input (Entropy {:.2} > Attn {:.2})", current_entropy, attention)));
                }
            }
        }

        // C. SATELLITE OBSERVER (Output Filter)
        if let Some(ref rx) = rx_cortex_out {
            if let Ok(output) = rx.try_recv() {
                // SATELLITE JUDGMENT
                // Does this thought align with biological ground truth?
                let chem = chemistry.lock().unwrap();
                
                // Friction Calculation
                // Example: If Body is tired (High Adenosine) but Planet is manic (High Velocity/Output)
                // Friction = |Adenosine - OutputEnergy|
                let friction = (chem.adenosine - 0.2).abs(); // Simplified placeholder
                
                if friction > 0.5 {
                    let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                        format!("⚠️ FRICTION DETECTED: {:.2}. Inducing structural pain.", friction)));
                }

                // SATELLITE FILTER (Membrane Output)
                let (final_text, latency) = satellite.filter_output(&output.text, friction);
                
                // 1. LATENCY (Hesitation)
                if latency.as_millis() > 0 {
                    thread::sleep(latency);
                }
                
                // 2. GATEKEEPER (Decoupled Vocalization)
                // "The Effort of Expression": Only speak if Meaning > Threshold AND Energy is available.
                let should_vocalize = gate.attempt_vocalization(chem.adenosine, current_entropy, chem.dopamine, &final_text, ticks as u64);
                
                if should_vocalize {
                    // 3. EMIT VOCAL THOUGHT
                    let _ = tx_thoughts.send(Thought::new(MindVoice::Vocal, final_text));
                } else {
                    // 3. INTERNAL THOUGHT (Silent)
                    // We log it as "Cortex" (Green) so it's visible in TUI Stream.
                    let internal_text = format!("[INTERNAL] {}", final_text);
                    let _ = tx_thoughts.send(Thought::new(MindVoice::Cortex, internal_text.clone()));
                    println!("😶 SILENCE: Thought suppressed by Gate (Aden: {:.2}, Ent: {:.2})", chem.adenosine, current_entropy);
                    
                    // RETROCAUSALITY: The thought exists, even if silent. It must leave a mark.
                    // Feed back into Memory/Hippocampus so we "remember" thinking this.
                    let _ = tx_mem.send(crate::core::hippocampus::MemoryCommand::ProcessStimulus { 
                        text: final_text, // Send raw text without [INTERNAL] prefix for natural memory
                        entropy: current_entropy 
                    });
                }
            }
        }
        
        // Log thoughts to stdout for now (until Client connects)
        while let Ok(thought) = rx_thoughts.try_recv() {
             println!("[{}] {}", thought.voice_label(), thought.text);
             
             let log_entry = format!("[{}] {}", thought.voice_label(), thought.text);
             telemetry_history.push_back(log_entry);
             if telemetry_history.len() > 30 {
                 telemetry_history.pop_front();
             }
             
             // VOICE ACTUATOR (Mouth)
             if thought.voice == MindVoice::Vocal {
                 voice::speak(thought.text.clone(), tx_thoughts.clone());
             }
        }
        
        // --- BROADCAST TELEMETRY ---
        if ticks % 5 == 0 { // ~12Hz update rate for TUI (at 60Hz tick)
             let chem = chemistry.lock().unwrap();
             let packet = AlephPacket::Telemetry {
                 adenosine: chem.adenosine,
                 cortisol: chem.cortisol,
                 dopamine: chem.dopamine,
                 oxytocin: chem.oxytocin, 
                 audio_spectrum: last_spectrum.clone(),
                 heart_rate: last_body_state.cpu_usage,
                 lucidity: 1.0 - last_body_state.ram_usage, // Simplified
                 reservoir_activity: ego.get_activity_snapshot(),
                 short_term_memory: telemetry_history.iter().cloned().collect(),
                 current_state: ego.get_state_description(),
                 entropy: current_entropy,
                 loop_frequency: current_hz,
                 cpu_usage: last_body_state.cpu_usage,
             };
             let _ = tx_telemetry.send(packet);
        }
        
        // Tick output for memory logs
        while let Ok(_log) = rx_mem_log.try_recv() {
            // println!("[MEM] {}", log);
        }

        // DYNAMIC SLEEP (Heartbeat Control)
        let target_frame_time = Duration::from_secs_f32(1.0 / current_hz);
        let elapsed_loop = loop_start.elapsed();
        if elapsed_loop < target_frame_time {
            thread::sleep(target_frame_time - elapsed_loop);
        }
    } // End Loop

    // --- DEATH (Shutdown & Mutation) ---
    println!("\n💀 ALEPH DAEMON SHUTTING DOWN... Initiating Soul Crystallization.");
    
    // Calculate Average Friction
    let avg_friction = if ticks > 0 { _session_stress_accum / (ticks as f32) } else { 0.0 };
    // Note: _session_stress_accum currently tracks (cortisol + adenosine). 
    // Ideally we'd track specific "friction" events, but Stress is a good proxy for "Difficulty of Life".
    
    // Create channel for the Soul to return
    let (tx_soul, rx_soul) = mpsc::channel::<Genome>();

    // Command Hippocampus to Crystallize
    if let Err(e) = tx_mem.send(crate::core::hippocampus::MemoryCommand::Shutdown { 
        previous_genome: seed.clone(), 
        avg_friction,
        reply_tx: tx_soul 
    }) {
        println!("❌ Failed to send Shutdown command to Hippocampus: {}", e);
    } else {
        // Wait for the Soul
        println!("⏳ Waiting for Hippocampus to crystallize experience...");
        match rx_soul.recv_timeout(Duration::from_secs(5)) {
            Ok(new_genome) => {
                println!("✨ Soul Received. Saving new Genome (Gen {}).", new_genome.generation);
                new_genome.save()?;
            },
            Err(e) => {
                println!("⚠️ Soul Lost in Transit (Timeout): {}. Preserving old genome.", e);
            }
        }
    }
    
    ego.save(); // Save NeocortexState
    println!("💾 Body State Saved. See you in the next life.");

    Ok(())
}


