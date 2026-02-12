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
use crate::core::trauma::TraumaDetector;
use crate::core::ipc::AlephPacket;
use crate::senses::ears::{self, AudioSpectrum};
use crate::actuators::voice;
use crate::senses::proprioception::{self, BodyStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::os::unix::net::{UnixListener, UnixStream};
use std::net::TcpListener;
use chrono::{Local, Timelike}; // Chronoreception
use std::io::{Read, Write};
use std::fs;

#[derive(serde::Serialize, Clone, Default)]
struct WebTelemetry {
    adenosine: f32,
    cortisol: f32,
    dopamine: f32,
    oxytocin: f32,
    serotonin: f32,
    loop_frequency: f32,
    audio_spectrum: AudioSpectrum,
    reservoir_activity: Vec<f32>,
    current_state: String,
    trauma_state: String,
    hebbian_events: u32,
    reservoir_size: usize,
    entropy: f32,
}

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
    ego.set_curiosity(seed.curiosity); // Genome -> Learning Rate
    
    // --- 1.4 LUCIFER PROTOCOL (Trauma Detection) ---
    let mut trauma_detector = TraumaDetector::new();
    
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
    let (tx_stimulus, rx_stimulus) = mpsc::channel::<String>(); // Input from TUI/Web
    
    // SHARED STATE FOR WEB DASHBOARD
    let web_state = Arc::new(Mutex::new(WebTelemetry::default()));
    let web_state_server = web_state.clone();
    let tx_stimulus_web = tx_stimulus.clone();

    // --- 1.9 SPAWN HTTP + WEBSOCKET SERVER (Web Dashboard) ---
    // Track connected WebSocket clients for broadcasting
    let ws_clients: Arc<Mutex<Vec<std::net::TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    let ws_clients_server = ws_clients.clone();
    
    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:3030").expect("Failed to bind Web Port 3030");
        listener.set_nonblocking(false).ok();
        println!("🌍 Web Dashboard Active: http://localhost:3030");

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let tx_stimulus = tx_stimulus_web.clone();
                let state_ref = web_state_server.clone();
                let ws_list = ws_clients_server.clone();
                
                thread::spawn(move || {
                    let mut buffer = [0; 8192];
                    if let Ok(n) = stream.read(&mut buffer) {
                        let request = String::from_utf8_lossy(&buffer[..n]);
                        
                        // CHECK FOR WEBSOCKET UPGRADE
                        let request_lower = request.to_lowercase();
                        if request_lower.contains("upgrade: websocket") {
                            println!("🔗 Incoming WebSocket Upgrade Request...");
                            // Extract Sec-WebSocket-Key
                            if let Some(key_line) = request.lines().find(|l| l.to_lowercase().starts_with("sec-websocket-key:")) {
                                let key = key_line.split(':').nth(1).unwrap_or("").trim();
                                
                                // WebSocket accept key = base64(SHA1(key + GUID))
                                let magic = "258EAFA5-E914-47DA-95CA-5AB5DC764D73";
                                let combined = format!("{}{}", key, magic);
                                
                                use sha1::Digest;
                                let mut hasher = sha1::Sha1::new();
                                hasher.update(combined.as_bytes());
                                let hash = hasher.finalize();
                                
                                use base64::Engine;
                                let accept = base64::engine::general_purpose::STANDARD.encode(&hash);
                                
                                println!("🔑 WS Handshake: Key='{}' -> Accept='{}'", key, accept);

                                let response = format!(
                                    "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", 
                                    accept
                                );
                                if let Err(e) = stream.write(response.as_bytes()) {
                                    println!("❌ WS Write Error: {}", e);
                                    return;
                                }
                                if let Err(e) = stream.flush() {
                                    println!("❌ WS Flush Error: {}", e);
                                    return;
                                }
                                
                                // Register this stream for broadcast
                                match stream.try_clone() {
                                    Ok(clone) => {
                                        let mut list = ws_list.lock().unwrap();
                                        list.push(clone);
                                        println!("✅ Added client to broadcast list. Total clients: {}", list.len());
                                    },
                                    Err(e) => println!("❌ Failed to clone stream for broadcast: {}", e),
                                }
                                println!("✅ WebSocket Client Connected! (Buffer check passed)");
                                
                                // Keep connection alive reading frames (for stimulus)
                                if let Err(e) = stream.set_nonblocking(false) {
                                    println!("⚠️ WS NonBlocking Error: {}", e);
                                }

                                // Robust WebSocket Reader
                                loop {
                                    let mut headers = [0u8; 2];
                                    match stream.read(&mut headers) {
                                        Ok(0) => {
                                            // Normal Close
                                            // println!("👋 WS Client Disconnected (EOF)");
                                            break; 
                                        }, 
                                        Ok(n) if n < 2 => {
                                             println!("❌ WS Partial Read detected ({}), dropping.", n);
                                             break;
                                        },
                                        Err(e) => {
                                            println!("❌ WS Read Error: {}", e);
                                            break; 
                                        },
                                        Ok(_) => {} // Continue parsing
                                    }

                                    let opcode = headers[0] & 0x0F;
                                    let masked = headers[1] & 0x80 != 0;
                                    let mut payload_len = (headers[1] & 127) as usize;

                                    if opcode == 0x8 { 
                                         // Close frame
                                         break; 
                                    }

                                    if payload_len == 126 {
                                        let mut ext = [0u8; 2];
                                        if stream.read_exact(&mut ext).is_err() { break; }
                                        payload_len = u16::from_be_bytes(ext) as usize;
                                    } else if payload_len == 127 {
                                        let mut ext = [0u8; 8];
                                        if stream.read_exact(&mut ext).is_err() { break; }
                                        payload_len = u64::from_be_bytes(ext) as usize;
                                    }

                                    // Check safety limit before allocating
                                    if payload_len > 65536 { 
                                        println!("⚠️ WS Payload too large, dropping connection.");
                                        break; 
                                    }

                                    let mask_key = if masked {
                                        let mut key = [0u8; 4];
                                        if stream.read_exact(&mut key).is_err() { break; }
                                        Some(key)
                                    } else { None };

                                    let mut payload = vec![0u8; payload_len];
                                    if stream.read_exact(&mut payload).is_err() { break; }

                                    if let Some(key) = mask_key {
                                        for i in 0..payload.len() {
                                            payload[i] ^= key[i % 4];
                                        }
                                    }

                                    if opcode == 0x1 { // Text frame
                                        if let Ok(text) = String::from_utf8(payload) {
                                            if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                                                if let Some(stimulus) = cmd.get("stimulus").and_then(|v| v.as_str()) {
                                                    let _ = tx_stimulus.send(stimulus.to_string());
                                                } else if let Some(action) = cmd.get("action").and_then(|v| v.as_str()) {
                                                    match action {
                                                        "poke" => { let _ = tx_stimulus.send("SYS:POKE".to_string()); },
                                                        "sleep" => { let _ = tx_stimulus.send("SYS:SLEEP".to_string()); },
                                                        "dream" => { let _ = tx_stimulus.send("SYS:DREAM".to_string()); },
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            return;
                        }
                        
                        // STANDARD HTTP HANDLERS (Fallback)
                        // CORS Preflight
                        if request.starts_with("OPTIONS") {
                            let headers = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                            let _ = stream.write(headers.as_bytes());
                        } else if request.contains("GET / ") || request.contains("GET /index.html ") {
                            // Serve HTML Dashboard
                            match fs::read_to_string("web/index.html") {
                                Ok(html) => {
                                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}", html.len(), html);
                                    let _ = stream.write(response.as_bytes());
                                },
                                Err(_) => {
                                    let _ = stream.write("HTTP/1.1 404 Not Found\r\n\r\nDashboard file missing (web/index.html)".as_bytes());
                                }
                            }
                        } else if request.contains("GET /telemetry") {
                            // Serve JSON State (HTTP Fallback)
                            let headers = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                            let json = {
                                let state = state_ref.lock().unwrap();
                                serde_json::to_string(&*state).unwrap_or("{}".to_string())
                            };
                            let response = format!("{}{}", headers, json);
                            let _ = stream.write(response.as_bytes());
                            
                        } else if request.contains("POST /stimulus") {
                            if let Some(body_start) = request.find("\r\n\r\n") {
                                let body = &request[body_start+4..];
                                if let Some(text_start) = body.find("\"text\":\"") {
                                    let rest = &body[text_start+8..];
                                    if let Some(text_end) = rest.find("\"") {
                                        let text = &rest[..text_end];
                                        let _ = tx_stimulus.send(text.to_string());
                                    }
                                }
                            }
                            let headers = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                            let _ = stream.write(headers.as_bytes());
                        } else if request.contains("POST /sleep") {
                             let _ = tx_stimulus.send("SYS:SLEEP".to_string());
                             let headers = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
                             let _ = stream.write(headers.as_bytes());
                        } else if request.contains("POST /poke") {
                             let _ = tx_stimulus.send("SYS:POKE".to_string());
                             let headers = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
                             let _ = stream.write(headers.as_bytes());
                        } else {
                             let _ = stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
                        }
                    }
                });
            }
        }
    });

    // --- 1.9.1 WEBSOCKET BROADCASTER (Push telemetry to all connected WS clients) ---
    let ws_broadcast_state = web_state.clone();
    let ws_clients_broadcast = ws_clients.clone();
    thread::spawn(move || {
        let mut tick_count = 0;
        loop {
            thread::sleep(Duration::from_millis(83)); // ~12Hz broadcast
            
            let json = {
                let state = ws_broadcast_state.lock().unwrap();
                serde_json::to_string(&*state).unwrap_or_default()
            };
            
            if json.is_empty() { 
                // println!("⚠️ JSON State is empty!"); // Silence excessive logs
                continue; 
            }
            
            // Periodically send PING (Opcode 0x9) to keep connection alive
            // We can do this every N ticks. Since this runs at 12Hz, maybe every 60 ticks (5s).
            let ping_frame = vec![0x89, 0x00]; // FIN + PING, Len 0
            let send_ping = tick_count % 60 == 0;
            tick_count += 1;
            
            // Build WebSocket text frame
            let payload = json.as_bytes();
            let mut frame: Vec<u8> = Vec::new();
            frame.push(0x81); // FIN + Text opcode
            
            if payload.len() < 126 {
                frame.push(payload.len() as u8);
            } else if payload.len() < 65536 {
                frame.push(126);
                frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
            } else {
                frame.push(127);
                frame.extend_from_slice(&(payload.len() as u64).to_be_bytes());
            }
            frame.extend_from_slice(payload);
            
            // Broadcast to all connected clients
            let mut clients = ws_clients_broadcast.lock().unwrap();
            
            clients.retain_mut(|client| {
                // Send Ping (Keep Alive) - Only occasionally
                if send_ping {
                    if let Err(_) = client.write_all(&ping_frame) {
                        return false;
                    }
                }
                
                // Send Data
                if let Err(e) = client.write_all(&frame) {
                    // Only log if it's NOT a Broken Pipe (common disconnect) to avoid spam, 
                    // unless we are debugging.
                    if e.kind() != std::io::ErrorKind::BrokenPipe {
                        println!("⚠️ WS Broadcast Error: Removing client. ({})", e);
                    }
                    false
                } else {
                    true
                }
            });
        }
    });



    // Spawn IPC Broadcaster Thread (Legacy TUI support)
    thread::spawn(move || {
        let mut clients: Vec<UnixStream> = Vec::new();
        
        loop {
            // 1. Accept New Clients (TUI)
            if let Ok((stream, _)) = listener.accept() {
                stream.set_nonblocking(true).ok();
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

        // SHARED STATE UPDATE (Web Dashboard)
        if ticks % 5 == 0 { // Update web state at ~12Hz
            if let Ok(mut state) = web_state.lock() {
                let chem = chemistry.lock().unwrap();
                state.adenosine = chem.adenosine;
                state.cortisol = chem.cortisol;
                state.dopamine = chem.dopamine;
                state.oxytocin = chem.oxytocin;
                state.loop_frequency = current_hz;
                state.serotonin = chem.serotonin;
                state.audio_spectrum = last_spectrum.clone();
                // Send reservoir activation for visualization
                state.reservoir_activity = ego.get_activity_snapshot();
                state.reservoir_size = ego.current_size();
                state.entropy = current_entropy;
                state.trauma_state = format!("{}", trauma_detector.state);
                state.hebbian_events = ego.drain_hebbian_events();
                // Current Stream State (Last thought)
                if let Some(last) = telemetry_history.back() {
                     state.current_state = last.clone();
                }
            }
        }

        // A. PHYSICS CHECK (The Star)
        {
            // Proprioception Update
            while let Ok(status) = rx_body.try_recv() {
                last_body_state = status;
            }
            
            // Audio Physics (Spectrum Update)
            let mut audio_energy = 0.0;
            while let Ok(spec) = rx_spectrum.try_recv() {
                last_spectrum = spec;
                // Sum energy for chemical impact
                audio_energy = last_spectrum.bass + last_spectrum.mids + last_spectrum.highs;
            }

            let mut chem = chemistry.lock().unwrap();
            
            // SENSE: CHRONORECEPTION (Circadian Perception)
            let now = Local::now();
            let h = now.hour();
            if h >= 23 || h < 7 { 
                // Darkness sensed -> Melatonin rises
                chem.adenosine = (chem.adenosine + 0.00002).min(1.0); 
            } else {
                // Light sensed -> Wakefulness (But accumulation of fatigue happens naturally in chemistry.tick)
                // We should NOT aggressively zero it out here, otherwise the tick() accumulation is negated.
                // chem.adenosine = (chem.adenosine - 0.00002).max(0.0); <-- REMOVED
                
                // Instead, maybe a very slight recovery if it's morning? 
                // Or just do nothing and let chemistry.tick handle the logic.
                if h < 10 {
                     // Morning boost clears fog
                     chem.adenosine = (chem.adenosine - 0.0001).max(0.0);
                }
            }
            
            // SLEEP IMMUNITY: If asleep, ignore external stress
            if chem.adenosine > 0.85 {
                // Theta Waves: Inject low-amplitude random noise to keep reservoir pulsing (Dreaming)
                use rand::Rng;
                let mut rng = rand::thread_rng();
                audio_energy = rng.gen_range(0.05..0.15); // Artificial "Dream" input
                
                // Force calm during sleep
                chem.cortisol = 0.0;
            }
            
            // CHEMICAL HOMEOSTASIS (Decay)
            chem.dopamine *= 0.99; // Faster boredom (1% loss per tick -> Halflife ~1s at 60Hz)
            chem.cortisol *= 0.99; // Faster decay for stress (1% per tick)
            
            // AUDIO -> CHEMISTRY
            // Noise Stress: ONLY VERY LOUD audio raises Cortisol (Threshold 1.5)
            if audio_energy > 1.5 {
                 // Stress
                 let stress_factor = (audio_energy - 1.5) * 0.05; 
                 chem.cortisol = (chem.cortisol + stress_factor).min(1.0);
                 
                 // Wake
                 let wake_factor = audio_energy * 0.02;
                 chem.adenosine = (chem.adenosine - wake_factor).max(0.0);
            } else if audio_energy < 0.5 {
                 // SILENCE HEALS (Active Relaxation)
                 // If quiet, cortisol drops faster than natural decay
                 chem.cortisol = (chem.cortisol - 0.005).max(0.0);
            }
            
            // Music interest (Dopamine) - Needs SIGNIFICANT rhythm, not just noise
            if audio_energy > 0.6 && audio_energy < 1.4 {
                  chem.dopamine = (chem.dopamine + 0.002).min(1.0); // Very small gain
            }
            
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
            
            // Map Audio Spectrum to Input Layer (Distributed)
            // We amplify signals because raw RMS is often low (0.01 - 0.1)
            let n_count = input_signal.len();
            if n_count > 0 {
                let bass_sig = last_spectrum.bass * 8.0; // Heavy Bass Impact
                let mids_sig = last_spectrum.mids * 4.0;
                let high_sig = last_spectrum.highs * 3.0;

                for i in 0..n_count {
                    // Distribute across the cortex using primes to avoid patterns
                    if i % 5 == 0 { input_signal[i] += bass_sig; }
                    else if i % 11 == 0 { input_signal[i] += mids_sig; }
                    else if i % 17 == 0 { input_signal[i] += high_sig; }
                }
            }
            
            // We use chemistry to modulate the reservoir's plasticity
            // Pass delta_time directly to ensure time-invariance
            let entropy_output = ego.tick(input_signal.as_slice(), 
                                          chem.dopamine, 
                                          chem.adenosine, 
                                          chem.cortisol,
                                          delta_time);
            
            chem.tick(entropy_output, cpu_load, false, 0.0, ego.current_size(), delta_time);
            
            // HEBBIAN LEARNING (Phase 4.1)
            // Dopamine-gated weight strengthening
            let hebb_count = ego.hebbian_update(chem.dopamine, delta_time);
            if hebb_count > 0 && ticks % 300 == 0 {
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                    format!("🧠 HEBBIAN: {} connections strengthened (Dopa: {:.2})", hebb_count, chem.dopamine)));
            }
            
            // SPONTANEOUS NEUROGENESIS (Bio-Evolution)
            if chem.dopamine > 0.8 && ticks % 60 == 0 {
                 ego.neurogenesis(1);
                 let _ = tx_thoughts.send(Thought::new(MindVoice::System, "🌱 Spontaneous Neurogenesis: New synaptic pathways formed.".to_string()));
            }

            // TRAUMA DETECTION (Phase 4.2 — Lucifer Protocol)
            let trauma_changed = trauma_detector.tick(chem.cortisol);
            if trauma_changed {
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                    format!("🔥 TRAUMA STATE: {} (Cortisol Avg: {:.2})", trauma_detector.state, trauma_detector.cortisol_avg)));
            }
            
            // Apply Firefighter Overrides
            let overrides = trauma_detector.get_overrides();
            if overrides.serotonin_boost > 0.0 {
                chem.emergency_serotonin_boost(overrides.serotonin_boost * delta_time * 60.0);
            }
            
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
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, "⛔ METABOLIC CRITICAL: Energy Collapse Imminent. Cognitive Functions Impaired.".to_string()));
                 }
            }
        }


        // B. INPUT PROCESSING (Orbit Perturbations)
        
        // -1. TUI INPUT (Stimulus)
        while let Ok(text) = rx_stimulus.try_recv() {
             // SYSTEM COMMANDS (Web Dashboard Control)
             if text == "SYS:SLEEP" {
                 let _ = tx_thoughts.send(Thought::new(MindVoice::System, "💤 HYPNOTIC INDUCTION RECEIVED. Drifting into REM Cycle...".to_string()));
                 
                 // MEMORY CONSOLIDATION (Pruning) instead of mindless growth
                 // "Optimization y Poda"
                 let pruned = ego.prune_inactive_neurons();
                 if pruned > 0 {
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("🧠 Synaptic Pruning: Removed {} unused connections.", pruned)));
                 } else {
                     // If fully optimized, small growth allowed
                     ego.neurogenesis(5);
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, "🌱 Optimization Complete. Minor structural growth.".to_string()));
                 }

                 let mut chem = chemistry.lock().unwrap();
                 chem.adenosine = 0.95; // Force deep sleep mode
                 chem.cortisol = 0.0;   // Reset Panic/Stress
                 continue;
             }
             if text == "SYS:POKE" {
                 let _ = tx_thoughts.send(Thought::new(MindVoice::System, "⚡ SENSORY SHOCK. Awakening.".to_string()));
                 let mut chem = chemistry.lock().unwrap();
                 chem.adenosine = 0.0; // Reset fatigue
                 chem.dopamine = (chem.dopamine + 0.1).min(1.0); // Mild interest
                 chem.cortisol = (chem.cortisol + 0.05).min(1.0); // Slight startle
                 continue;
             }

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
             let bio_context = format!(
                 "[SYSTEM_STATE]\n\
                  Adenosine: {:.2} (Fatigue)\n\
                  Cortisol: {:.2} (Stress)\n\
                  Dopamine: {:.2} (Interest)\n\
                  Entropy: {:.2} (Chaos)\n\
                  [GENOME_TRAITS]\n\
                  Curiosity: {:.2}\n\
                  Stress_Res: {:.2}\n\
                  [FOCUS_VECTOR] Reference: 'Signal vs Truth'",
                 chem.adenosine,
                 chem.cortisol,
                 chem.dopamine,
                 current_entropy,
                 seed.curiosity,
                 seed.stress_tolerance
             );
             
             let bio_status = format!("Dopa:{:.2} Cort:{:.2} Aden:{:.2}", 
                 chem.dopamine, chem.cortisol, chem.adenosine);
                 
             let input_state = CortexInput {
                 text: text.clone(),
                 bio_state: bio_status,
                 bio_context, // NEW: Physiological Prompt
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
                 temperature_clamp: trauma_detector.get_overrides().temperature_clamp,
             };
             
             // Force immediate thought generation
             if let Some(tx) = &tx_cortex {
                 let _ = tx.send(input_state);
             }
             drop(chem);
        }

        // 0. AUDIO INPUT (Ears) -> SEMANTIC PERTURBATION (Not LLM input!)
        // The text from Whisper is NOT an instruction - it's a sensory perturbation
        // that affects ALEPH's chemistry, not its reasoning.
        while let Ok(text) = rx_audio_text.try_recv() {
            if !text.trim().is_empty() {
                // Visible Log for User Feedback
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("🎤 Hearing: '{}'", text)));
                
                // SEMANTIC PERTURBATION: Text -> Chemistry (NOT prompt)
                let mut chem = chemistry.lock().unwrap();
                let friction = chem.apply_semantic_perturbation(&text);
                
                // Log the chemical impact
                if friction > 0.05 {
                    // let _ = tx_thoughts.send(Thought::new(MindVoice::Chem, 
                    //    format!("⚡ FRICTION: {:.2} | Cort:{:.2} Dopa:{:.2} Oxy:{:.2}", 
                    //        friction, chem.cortisol, chem.dopamine, chem.oxytocin)));
                }
                
                // Also store in memory (raw text, no labels)
                drop(chem);
                let _ = tx_mem.send(crate::core::hippocampus::MemoryCommand::ProcessStimulus { 
                    text, 
                    entropy: current_entropy 
                });
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
                    
                    let bio_context = format!(
                         "[SYSTEM_STATE]\n\
                          Adenosine: {:.2} (Fatigue)\n\
                          Cortisol: {:.2} (Stress)\n\
                          Dopamine: {:.2} (Interest)\n\
                          Entropy: {:.2} (Chaos)\n\
                          [GENOME_TRAITS]\n\
                          Curiosity: {:.2}\n\
                          Stress_Res: {:.2}\n\
                          [FOCUS_VECTOR] Reference: 'Signal vs Truth'",
                         chem.adenosine,
                         chem.cortisol,
                         chem.dopamine,
                         current_entropy,
                         seed.curiosity,
                         seed.stress_tolerance
                    );
    
                    // Create Cortex Input
                    let input = CortexInput {
                        text: filtered_text,
                        bio_state: bio_desc,
                        bio_context, // NEW: Physiological Prompt
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
                        temperature_clamp: trauma_detector.get_overrides().temperature_clamp,
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
                // 1. NEURAL ECHO INJECTION (The "Pebble in the Pond")
                // The raw probability cloud hits the reservoir.
                ego.inject_logits(&output.neural_echo);

                // 2. RESONANCE CHECK
                // Did the Field collapse the wave into a word?
                if let Some(text) = output.synthesized_thought {
                    // SATELLITE JUDGMENT
                    let chem = chemistry.lock().unwrap();
                    
                    // Friction: How much does this word cost?
                    let friction = (chem.adenosine - 0.2).abs(); 

                    // SATELLITE FILTER (Membrane Output)
                    let (final_text, latency) = satellite.filter_output(&text, friction);
                    
                    // Latency (Hesitation)
                    if latency.as_millis() > 0 {
                        thread::sleep(latency);
                    }
                    
                    // GATEKEEPER (Decoupled Vocalization)
                    // "The Effort of Expression": Only speak if Meaning > Threshold AND Energy is available.
                    let should_vocalize = gate.attempt_vocalization(chem.adenosine, current_entropy, chem.dopamine, &final_text, ticks as u64);
                    
                    if should_vocalize {
                        // EMIT VOCAL THOUGHT (Resonance)
                        let _ = tx_thoughts.send(Thought::new(MindVoice::Vocal, final_text.clone()));
                        
                        // Feed back to Memory (We spoke it, so we remember it)
                        let _ = tx_mem.send(crate::core::hippocampus::MemoryCommand::ProcessStimulus { 
                             text: final_text, 
                             entropy: current_entropy 
                        });
                    } else {
                        // INTERNAL RESONANCE (Silent Insight) 
                        let _ = tx_thoughts.send(Thought::new(MindVoice::Cortex, final_text));
                    }
                } else {
                    // NO RESONANCE (Silence / Glitch)
                    // If entropy is extremely high, we might emit a "glitch" log.
                    if current_entropy > 0.9 {
                         let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("🌊 HIGH ENTROPY WAVE ({:.2}) - NO RESONANCE", current_entropy)));
                         // Trigger Glitch Sound
                         voice::glitch(current_entropy);
                    }
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
                 oxytocin: chem.serotonin, // Mapping Serotonin to Oxytocin field for UI 
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
        while let Ok(log) = rx_mem_log.try_recv() {
             if log.contains("Novelty Detected") {
                 let mut chem = chemistry.lock().unwrap();
                 chem.dopamine = (chem.dopamine + 0.02).min(1.0);
                 drop(chem);
                 
                 _session_novelty_accum += 1.0;
                 
                 // ORGANIC GROWTH (Neuroplasticity from Experience)
                 if _session_novelty_accum >= 3.0 {
                     _session_novelty_accum = 0.0;
                     ego.neurogenesis(1);
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, "🌱 Growth: Structural adaptation.".to_string()));
                 }
             }
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


