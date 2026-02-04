use anyhow::Result;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use crate::core::thought::{Thought, MindVoice};
use crate::core::reservoir::FractalReservoir;
use crate::core::planet::{Planet, CortexInput, CortexOutput};
use crate::core::chemistry::Neurotransmitters;
use crate::core::hippocampus::Hippocampus;
use crate::core::genome::Genome;
use crate::core::satellite::Satellite;
use crate::senses::proprioception::{self, BodyStatus};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn run() -> Result<()> {
    println!("🌟 ALEPH STAR SYSTEM ONLINE (Daemon Mode)");

    // --- CHANNELS ---
    let (tx_thoughts, rx_thoughts) = mpsc::channel::<Thought>();
    
    // --- 0. GENOME (The Seed) ---
    let mut seed = Genome::load()?;
    let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
        format!("🧬 GENOME LOADED: Gen {} | StressRes: {:.2}", seed.generation, seed.stress_tolerance)));

    // --- 1. THE STAR (Biological Ground Truth) ---
    let chemistry = Arc::new(Mutex::new(Neurotransmitters::new()));
    // Reservoir (The Body's Neural Network)
    let mut ego = FractalReservoir::new(500, 0.2); 
    
    // --- 1.5 THE SATELLITE (Observer) ---
    let satellite = Satellite::new(seed.paranoia, seed.refractive_index); 
    
    // Hardware Proprioception
    let (tx_body, rx_body) = mpsc::channel::<BodyStatus>();
    proprioception::spawn_monitor(tx_body);
    let mut last_body_state = BodyStatus { cpu_usage: 0.0, ram_usage: 0.0 };

    // Graceful Shutdown Handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?; 
    
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
    let (_tx_mem, rx_mem_out, rx_mem_log) = Hippocampus::spawn()
        .expect("Hippocampus Failed");

    // --- DAEMON LOOP (The Pulse) ---
    let mut last_tick = Instant::now();
    let mut current_entropy = 0.0;
    
    // Session Stats for Mutation
    let mut session_stress_accum = 0.0;
    let mut session_novelty_accum = 0.0;
    let mut ticks = 0;

    while running.load(Ordering::SeqCst) {
        let delta_time = last_tick.elapsed().as_secs_f32();
        last_tick = Instant::now();

        // A. PHYSICS CHECK (The Star)
        {
            // Proprioception Update
            while let Ok(status) = rx_body.try_recv() {
                last_body_state = status;
            }

            let mut chem = chemistry.lock().unwrap();
            
            // Map Hardware -> Biology
            // CPU Load (0-100) -> Metabolism/HeartRate
            // RAM Load (0.0-1.0) -> Brain Fog
            let cpu_load = last_body_state.cpu_usage; 
            let _ram_load = last_body_state.ram_usage;

            // Star burns fuel
            chem.tick(current_entropy, cpu_load, false, 0.0, ego.current_size(), delta_time);
            
            // Update Stats
            session_stress_accum += chem.cortisol + chem.adenosine;
            ticks += 1;

            // Critical Collapse Check
            if chem.adenosine > 0.95 * seed.stress_tolerance {
                 // BIOLOGICAL STRIKE
                 // Determine refusal (TODO)
            }
        }


        // B. INPUT PROCESSING (Orbit Perturbations)
        // 1. MEMORY & RESERVOIR FEEDBACK
        if let Ok(mem_out) = rx_mem_out.try_recv() {
            let mut chem = chemistry.lock().unwrap();

            // Update Stats
            session_novelty_accum += mem_out.novelty;
            
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
                if let Some(filtered_text) = satellite.filter_input(&mem_out.input_text, current_entropy, attention) {
                    
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
                
                // 2. EMIT THOUGHT
                let _ = tx_thoughts.send(Thought::new(MindVoice::Cortex, final_text));
            }
        }
        
        // Log thoughts to stdout for now (until Client connects)
        while let Ok(thought) = rx_thoughts.try_recv() {
             println!("[{}] {}", thought.voice_label(), thought.text);
        }
        
        // Tick output for memory logs
        while let Ok(log) = rx_mem_log.try_recv() {
            // println!("[MEM] {}", log);
        }

        thread::sleep(Duration::from_millis(100));
    } // End Loop

    // --- DEATH (Shutdown & Mutation) ---
    println!("\n💀 ALEPH DAEMON SHUTTING DOWN...");
    let avg_stress = if ticks > 0 { session_stress_accum / ticks as f32 } else { 0.0 };
    let avg_novelty = if ticks > 0 { session_novelty_accum / ticks as f32 } else { 0.0 };
    
    seed.mutate(avg_stress, avg_novelty, 0); // 0 trauma events for now
    println!("💾 Genome Saved. See you in the next life.");

    Ok(())
}


