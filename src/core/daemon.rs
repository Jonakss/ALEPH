use anyhow::Result;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::collections::VecDeque;
use crate::core::thought::{Thought, MindVoice};
use crate::core::reservoir::FractalReservoir;
use crate::cortex::planet::{Planet, CortexInput};
use crate::core::chemistry::Neurotransmitters;
use crate::core::hippocampus::Hippocampus;
use crate::core::neocortex::Neocortex;
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
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

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
    thoughts: Vec<String>,
    trauma_state: String,
    hebbian_events: u32,
    reservoir_size: usize,
    entropy: f32,
    llm_activity: Vec<f32>,
    top_activations: Vec<(String, f32)>, 
    
    // System Vitals
    system_ram_gb: f32,
    system_cpu_load: f32,
    
    // Neural Visualization
    activations: Vec<f32>,
    region_map: Vec<u8>,
    neuron_positions: Vec<[f32; 3]>,
    
    // Genome Traits
    curiosity: f32,
    
    // New Senses
    visual_cortex: Vec<f32>, // 64x64 Grid
    stress_tolerance: f32,
    generation: u32,
}

pub fn run(listen_path: Option<String>, headless: bool) -> Result<()> {
    println!("🌟 ALEPH STAR SYSTEM ONLINE (Daemon Mode)");
    
    // Proprioception (System Monitor)
    let mut _sys = sysinfo::System::new_all();

    // --- CHANNELS ---
    let (tx_thoughts, rx_thoughts) = mpsc::channel::<Thought>();
    
    // --- 1.7 SENSORY STATE (Phase 2) ---
    // Persistent buffer for sensory inputs (Audio/Vision) that decays over time
    let mut current_sensory_vector: Vec<f32> = vec![0.0; 500];
    
    // --- 2. START THE LOOP ---
    // Track last active interaction to trigger Spontaneous Thought
    let mut last_interaction_tick: u64 = 0; 
    let mut _ticks: u64 = 0;
    // --- 0. GENOME (The Seed) ---
    let mut seed = Genome::load()?;
    let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
        format!("🧬 GENOME LOADED: Gen {} | StressRes: {:.2}", seed.generation, seed.stress_tolerance)));

    // --- 1. THE STAR (Biological Ground Truth) ---
    let chemistry = Arc::new(Mutex::new(Neurotransmitters::new()));
    
    // GENESIS: Calculate Brain Size from Genome
    // Base 500 + (Generation * 10) + (Curiosity * 50) - (Paranoia * 20)
    // Example Gen 1, Cur 0.5: 500 + 10 + 25 = 535 neurons
    // Example Gen 10, Cur 0.9: 500 + 100 + 45 = 645 neurons
    let base_size = 500;
    let genetic_bonus = (seed.generation * 10) as usize;
    let trait_bonus = (seed.curiosity * 50.0) as usize;
    let birth_size = base_size + genetic_bonus + trait_bonus;
    
    // Reservoir (The Body's Neural Network) - Loads from disk OR Creates using birth_size
    let mut ego = FractalReservoir::load(birth_size, 0.2);
    ego.set_curiosity(seed.curiosity); // Genome -> Learning Rate
    
    // --- 1.4 LUCIFER PROTOCOL (Trauma Detection) ---
    let mut trauma_detector = TraumaDetector::new();
    
    // --- 1.4.1 NEOCORTEX (Structural Observer) ---
    let mut neocortex = Neocortex::new();
    
    // --- 1.5 THE SATELLITE (Observer) ---
    let satellite = Satellite::new(seed.paranoia, seed.refractive_index); 

    // --- 1.6 AGENCY (Goal System) ---
    let mut agent = crate::core::agency::Agency::new();
    let mut interaction_count: u64 = 0; // Track successful interactions
    let mut gate = ExpressionGate::new();
    
    // Hardware Proprioception
    let (tx_body, rx_body) = mpsc::channel::<BodyStatus>();
    proprioception::spawn_monitor(tx_body);
    let mut last_body_state = BodyStatus { cpu_usage: 0.0, ram_usage: 0.0 };

    // --- 1.6 SENSES (Ears) ---
    // Channels for Audio
    let (tx_audio_text, rx_audio_text) = mpsc::channel::<String>();
    let (tx_spectrum, rx_spectrum) = mpsc::channel::<AudioSpectrum>();
    let (tx_word_embedding, rx_word_embedding) = mpsc::channel::<Vec<f32>>();
    
    // WebSocket Audio channel (browser mic → backend ears)
    let (ws_audio_tx, ws_audio_rx) = mpsc::channel::<Vec<f32>>();
    let ws_audio_tx = Arc::new(Mutex::new(ws_audio_tx));
    let ws_audio_tx_server = ws_audio_tx.clone();
    
    // Detect Sensory Mode
    let sensory_mode = if headless {
        ears::SensoryMode::Headless
    } else if let Some(ref path) = listen_path {
        ears::SensoryMode::File(path.clone())
    } else {
        // Try local mic, fallback to WebSocket if no device
        match {
            use cpal::traits::HostTrait;
            cpal::default_host().default_input_device()
        } {
            Some(_) => ears::SensoryMode::Mic,
            None => {
                println!("⚠️ No microphone detected. Falling back to WebSocket Audio Mode.");
                ears::SensoryMode::WebSocket
            }
        }
    };
    
    let needs_ws_audio = matches!(sensory_mode, ears::SensoryMode::WebSocket);
    
    // Spawn Audio Listener with detected mode
    let _ears = ears::AudioListener::new(
        tx_thoughts.clone(), tx_audio_text, tx_spectrum, tx_word_embedding,
        sensory_mode, 
        if needs_ws_audio { Some(ws_audio_rx) } else { None }
    ).expect("Failed to spawn Ears");
    let mut last_spectrum = AudioSpectrum::default();

    let (tx_vision, rx_vision) = mpsc::channel::<Vec<f32>>();
    let _eyes = crate::senses::eyes::Eyes::new(tx_vision);
    _eyes.run();

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
                let ws_audio_tx = ws_audio_tx_server.clone();
                
                thread::spawn(move || {
                    let mut buffer = [0; 8192];
                    if let Ok(n) = stream.read(&mut buffer) {
                        let request = String::from_utf8_lossy(&buffer[..n]);
                        
                        // CHECK FOR WEBSOCKET UPGRADE
                        if request.len() > 0 {
                             // println!("📝 Raw Request: {:?}", request.lines().next()); // Log first line only
                        }
                        
                        let request_lower = request.to_lowercase();
                        if request_lower.contains("upgrade: websocket") {
                            println!("🔗 Incoming WebSocket Upgrade Request...");
                            // Extract Sec-WebSocket-Key
                            if let Some(key_line) = request.lines().find(|l| l.to_lowercase().starts_with("sec-websocket-key:")) {
                                let key = key_line.split(':').nth(1).unwrap_or("").trim();
                                
                                // WebSocket accept key = base64(SHA1(key + GUID))
                                let magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
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
                                            println!("👋 WS Disconnected (Client Closed)"); 
                                            break; 
                                        }, 
                                        Ok(n) if n < 2 => {
                                             println!("❌ WS Partial Read ({}), Dropping.", n);
                                             break;
                                        },
                                        Err(e) => {
                                            if e.kind() != std::io::ErrorKind::WouldBlock {
                                                println!("❌ WS Read Error: {}", e);
                                            }
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

                                    // Check safety limit before allocating (256KB for audio chunks)
                                    if payload_len > 262144 { 
                                        println!("⚠️ WS Payload too large ({}b), dropping connection.", payload_len);
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
                                    // Binary frame (opcode 0x2) = Browser Audio PCM
                                    else if opcode == 0x2 {
                                        // Decode f32 PCM samples from browser
                                        // Browser sends Float32Array as raw bytes (4 bytes per sample, little-endian)
                                        if payload.len() >= 4 && payload.len() % 4 == 0 {
                                            let samples: Vec<f32> = payload.chunks_exact(4)
                                                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                                                .collect();
                                            if let Ok(tx) = ws_audio_tx.lock() {
                                                let _ = tx.send(samples);
                                            }
                                        }
                                    }
                                }
                            }
                            return;
                        }
                        
                        // STANDARD HTTP HANDLERS (Fallback)
                        let path = request.lines().next().unwrap_or("").split_whitespace().nth(1).unwrap_or("/");
                        
                        // CORS Preflight
                        if request.starts_with("OPTIONS") {
                            let headers = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                            let _ = stream.write(headers.as_bytes());
                        } 
                        // SERVE ASSETS (Vite Build)
                        else if path.starts_with("/assets/") {
                            // Sanitize path (basic)
                            let safe_path = path.replace("..", ""); 
                            let file_path = format!("web{}", safe_path);
                            
                            if let Ok(content) = fs::read(&file_path) {
                                let content_type = if file_path.ends_with(".css") { "text/css" }
                                                  else if file_path.ends_with(".js") { "application/javascript" }
                                                  else if file_path.ends_with(".svg") { "image/svg+xml" }
                                                  else { "application/octet-stream" };
                                                  
                                let headers = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n", content_type, content.len());
                                let _ = stream.write(headers.as_bytes());
                                let _ = stream.write(&content);
                            } else {
                                let _ = stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
                            }
                        }
                        // SERVE DASHBOARD (DISABLED - Legacy)
                        else if path == "/" || path == "/index.html" {
                            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nALEPH Nervous System Active. Use React Client.\r\n";
                            let _ = stream.write(response.as_bytes());
                        }
                        // API ENDPOINTS
                        else if path == "/telemetry" {
                            let headers = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
                            let json = {
                                let state = state_ref.lock().unwrap();
                                serde_json::to_string(&*state).unwrap_or("{}".to_string())
                            };
                            let response = format!("{}{}", headers, json);
                            let _ = stream.write(response.as_bytes());
                        }
                        else if path == "/stimulus" && request.starts_with("POST") {
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
                            let headers = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
                            let _ = stream.write(headers.as_bytes());
                        }
                        // COMMAND SHORTCUTS
                        else if path == "/sleep" && request.starts_with("POST") {
                             let _ = tx_stimulus.send("SYS:SLEEP".to_string());
                             let _ = stream.write("HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\n\r\n".as_bytes());
                        } 
                        else if path == "/poke" && request.starts_with("POST") {
                             let _ = tx_stimulus.send("SYS:POKE".to_string());
                             let _ = stream.write("HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\n\r\n".as_bytes());
                        } 
                        else {
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
            
            let (json, _state_summary) = {
                let state = ws_broadcast_state.lock().unwrap();
                
                // Sparse Updates: Filter neurons > 0.005 and round to 3 decimals
                let sparse_reservoir: Vec<(usize, f32)> = state.reservoir_activity.iter().enumerate()
                    .filter(|(_, &v)| v > 0.005)
                    .map(|(i, &v)| (i, (v * 1000.0).round() / 1000.0))
                    .collect();

                // Sanitize Activations (Replace NaN/Inf with 0.0)
                let clean_activations: Vec<f32> = state.activations.iter()
                    .map(|&v| if v.is_finite() { v } else { 0.0 })
                    .collect();

                let json_obj = serde_json::json!({
                    "dopamine": if state.dopamine.is_finite() { (state.dopamine * 1000.0).round() / 1000.0 } else { 0.0 },
                    "cortisol": if state.cortisol.is_finite() { (state.cortisol * 1000.0).round() / 1000.0 } else { 0.0 },
                    "adenosine": if state.adenosine.is_finite() { (state.adenosine * 1000.0).round() / 1000.0 } else { 0.0 },
                    "oxytocin": if state.oxytocin.is_finite() { (state.oxytocin * 1000.0).round() / 1000.0 } else { 0.0 },
                    "serotonin": if state.serotonin.is_finite() { (state.serotonin * 1000.0).round() / 1000.0 } else { 0.0 },
                    "entropy": if state.entropy.is_finite() { (state.entropy * 1000.0).round() / 1000.0 } else { 0.0 },
                    "loop_frequency": if state.loop_frequency.is_finite() { (state.loop_frequency * 10.0).round() / 10.0 } else { 0.0 },
                    "reservoir_activity": sparse_reservoir, 
                    "current_state": state.current_state,
                    "thoughts": state.thoughts,
                    "trauma_state": state.trauma_state,
                    "hebbian_events": state.hebbian_events,
                    "reservoir_size": state.reservoir_size,
                    "top_activations": state.top_activations,
                    "llm_activity": state.llm_activity,
                    "system_ram_gb": state.system_ram_gb,
                    "system_cpu_load": state.system_cpu_load,
                    "activations": clean_activations,
                    "region_map": state.region_map,
                    "neuron_positions": state.neuron_positions,
                    "curiosity": state.curiosity,
                    "stress_tolerance": state.stress_tolerance,
                    "generation": state.generation
                });
                
                let s = json_obj.to_string();
                let summary = format!("Payload: {} bytes (Activations: {})", s.len(), sparse_reservoir.len());
                (s, summary)
            };
            
            
            // Periodically send PING (Opcode 0x9) to keep connection alive
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
            let client_count = clients.len();
            
            // Log payload size occasionally (every 60 ticks / 5s)
            if tick_count % 60 == 0 {
                 println!("📉 Telemetry Payload: {} bytes | Clients: {}", json.len(), client_count);
            }

            clients.retain_mut(|client| {
                // Send Ping (Keep Alive) - Only occasionally
                if send_ping {
                    if let Err(_) = client.write_all(&ping_frame) {
                        return false;
                    }
                }
                
                // Send Data
                match client.write_all(&frame) {
                    Ok(_) => true,
                    Err(e) => {
                         // Only log actual errors, not just disconnects
                         if e.kind() != std::io::ErrorKind::BrokenPipe {
                             println!("⚠️ WebSocket Write Error: {}", e);
                         }
                         false
                    }
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
    
    // SLEEP STATE (Persistent)
    let mut is_dreaming = false;
    
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
                state.oxytocin = chem.serotonin; // MIRROR TUI: Serotonin (Stability) = Trust (Oxytocin)
                state.loop_frequency = current_hz;
                state.serotonin = chem.serotonin;
                state.audio_spectrum = last_spectrum.clone();
                // Send reservoir activation for visualization
                state.reservoir_activity = ego.get_activity_snapshot();
                state.reservoir_size = ego.current_size();
                state.entropy = current_entropy;
                state.trauma_state = format!("{}", trauma_detector.state);
                state.hebbian_events = ego.drain_hebbian_events();
                state.region_map = ego.get_region_map();
                state.neuron_positions = ego.get_positions().clone();
                // Current Stream State (Full history for UI)
                state.thoughts = telemetry_history.iter().cloned().collect();
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
                last_spectrum = spec.clone();
                // Sum energy for chemical impact
                audio_energy = last_spectrum.bass + last_spectrum.mids + last_spectrum.highs;
                
                // CRITICAL: Immediate Update for UI Visualization
                if ticks % 2 == 0 { // 30Hz visual update for smoothness
                    if let Ok(mut state) = web_state.lock() {
                        state.audio_spectrum = spec.clone();
                    }
                }

                // DIRECT SENSORY PROJECTION (Phase 5)
                // Inject raw audio spectrogram into the Reservoir
                if !spec.frequency_embedding.is_empty() {
                    ego.inject_embedding(&spec.frequency_embedding, crate::core::reservoir::NeuronRegion::Auditory);
                }

                // 2. VISUAL SENSATION (Phase 7 - Occipital Lobe)
                // Now receiving 64x64 Grid (4096 floats)
                if let Ok(visual_grid) = rx_vision.try_recv() {
                     // 1. Update Web State for Visualization
                     if ticks % 4 == 0 { // ~15Hz update for UI
                        if let Ok(mut state) = web_state.lock() {
                            state.visual_cortex = visual_grid.clone();
                        }
                     }

                     // 2. Downsample for Reservoir Embedding (4096 -> 64)
                     // Simple strided sampling: take every 64th value
                     // Better: Average pooling? Let's do strided for speed first.
                     let mut embedding = Vec::with_capacity(64);
                     let stride = visual_grid.len() / 64;
                     for i in 0..64 {
                         if let Some(val) = visual_grid.get(i * stride) {
                             embedding.push(*val);
                         } else {
                             embedding.push(0.0);
                         }
                     }
                     
                     ego.inject_embedding(&embedding, crate::core::reservoir::NeuronRegion::Visual);
                }

                // STARTLE REFLEX (Cortisol)
                let intensity = spec.bass.max(spec.mids);
                if intensity > 0.6 {
                    let mut chem = chemistry.lock().unwrap();
                    chem.cortisol += intensity * 0.05; 
                    if intensity > 0.95 {
                        chem.cortisol += 0.2;
                             let _ = tx_thoughts.send(Thought::new(MindVoice::System, "💥 AUDITORY SHOCK!".to_string()));
                    }
                }
            }

            // === WORD EMBEDDING PATHWAY (Phase 2: Wernicke's Area) ===
            // Whisper text → hash embedding → Semantic region
            // Latency: ~50-200ms (Whisper inference time)
            // This is SLOWER than raw FFT (~5ms) but FASTER than full LLM (~500-2000ms)
            while let Ok(word_vec) = rx_word_embedding.try_recv() {
                ego.inject_embedding(&word_vec, crate::core::reservoir::NeuronRegion::Semantic);
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                    format!("🧠 WORD EMBED → Semantic ({} dims)", word_vec.len())));
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
            

            if is_dreaming {
                // Theta Waves: Inject low-amplitude random noise to keep reservoir pulsing (Dreaming)
                use rand::Rng;
                let mut rng = rand::thread_rng();
                audio_energy = rng.gen_range(0.05..0.15); // Artificial "Dream" input
                
                // Force calm during sleep
                chem.cortisol = 0.0;
            }
            
            // CHEMICAL HOMEOSTASIS (Gradual Decay — organic, not binary)
            // Real neurotransmitters have halflife of minutes, not seconds
            // At 60Hz: 0.9996^60 ≈ 0.976 per second → halflife ~30 seconds
            chem.dopamine *= 0.9996;
            // Cortisol decays slightly faster (body clears stress hormones)
            // 0.9994^60 ≈ 0.965 per second → halflife ~20 seconds
            chem.cortisol *= 0.9994;
            
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
            
            // CHRONORECEPTION (Phase 2)
            // Bind Biology to Local Time (Circadian Rhythm)
            if ticks % 600 == 0 { // Every ~10 seconds
                 let now = Local::now();
                 let hour = now.hour();
                 
                 // Circadian Pressure on Adenosine
                 let circadian_pressure = if hour >= 23 || hour < 6 {
                     0.005 // Night: Strong sleep pressure (+0.03/min)
                 } else if hour >= 20 {
                     0.002 // Evening: Wind down
                 } else if hour >= 6 && hour < 9 {
                     -0.005 // Morning: Cortisol spike / Waking up
                 } else {
                     -0.001 // Day: Maintenance (fighting fatigue)
                 };
                 
                 chem.adenosine = (chem.adenosine + circadian_pressure).clamp(0.0, 1.0);
            }

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
            // We use chemistry to modulate the reservoir's plasticity
            // Pass delta_time directly to ensure time-invariance
            
            // MIX SENSORY INPUT (Phase 2)
            // Combine Cortex Echo (Thinking) + Sensory Buffer (Hearing)
            // This allows Co-occurrence Hebbian Learning
            if input_signal.len() >= 500 && current_sensory_vector.len() >= 500 {
                 for i in 0..500 {
                     input_signal[i] += current_sensory_vector[i]; // Add sensation to thought
                 }
            }
            
            let entropy_output = ego.tick(input_signal.as_slice(), 
                                          chem.dopamine, 
                                          chem.adenosine, 
                                          chem.cortisol,
                                          delta_time);
            
            chem.tick(entropy_output, cpu_load, is_dreaming, 0.0, ego.current_size(), delta_time);
            
            // HEBBIAN LEARNING (Phase 4.1 + Phase 2)
            // 1. Recurrent Hebbian (Internal Structure)
            let hebb_count = ego.hebbian_update(chem.dopamine, delta_time);
            
            // 2. Input-State Hebbian (Sensory-Motor Map)
            // Learn to associate Audio with Concept
            let input_hebb_count = ego.hebbian_input_update(input_signal.as_slice(), chem.dopamine);
            
            if (hebb_count > 0 || input_hebb_count > 0) && ticks % 300 == 0 {
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                    format!("🧠 HEBBIAN: {} internal / {} sensory connections strengthened", hebb_count, input_hebb_count)));
            }

            // REWARD AS STRUCTURE (Epiphany)
            // If Dopamine is critical (>0.9), trigger structural lock-in (LTP)
            if chem.dopamine > 0.9 && ticks % 100 == 0 {
                 let changes = ego.trigger_epiphany(chem.dopamine);
                 if changes > 0 {
                     let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                         format!("🌟 EPIPHANY: Structural Reinforcement of {} pathways.", changes)));
                     
                     // DOPAMINE CRASH (Refractory Period)
                     // The brain consumes the neurochemical resources to build the structure.
                     // Satisfaction / Afterglow.
                     chem.dopamine = 0.55; 
                 }
            }
            
            // Decay Sensory Buffer (Persistence of Sensation)
            // Sensation lingers for ~200ms (0.9 decay at 60Hz)
            for x in current_sensory_vector.iter_mut() {
                *x *= 0.9;
            }
            
            // SPONTANEOUS NEUROGENESIS (Bio-Evolution)
            // Brain grows with activity, not just extreme dopamine
            // Dopamine > 0.15 = mild interest = slow growth
            if chem.dopamine > 0.15 && ticks % 300 == 0 {
                 ego.neurogenesis(1);
                 let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                     format!("🌱 Spontaneous Neurogenesis: +1 neuron (Total: {})", ego.current_size())));
            }
            
            // ACTIVITY-DRIVEN NEUROGENESIS
            // Edge of Chaos (entropy 0.3-0.7) = interesting regime = brain adapts
            if current_entropy > 0.3 && current_entropy < 0.7 && ticks % 600 == 0 {
                 ego.neurogenesis(1);
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
            
            // NEOCORTEX OBSERVATION (Meta-Cognition)
            if let Some(event) = neocortex.observe(current_entropy) {
                 // Log event to internal monologue
                 let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("{}", event)));
                 
                 // React to event
                 match event {
                     crate::core::neocortex::CognitiveEvent::Neurogenesis => {
                         ego.neurogenesis(2); // Boost growth
                         chem.dopamine = (chem.dopamine + 0.1).min(1.0); // Reward growth
                     },
                     crate::core::neocortex::CognitiveEvent::Trauma(_) => {
                         chem.cortisol = (chem.cortisol + 0.05).min(1.0);
                     },
                     crate::core::neocortex::CognitiveEvent::Boredom => {
                         chem.dopamine *= 0.995; // Gentle boredom fade, not a crash
                     },
                     _ => {}
                 }
            }

            // AGENCY EVALUATION (Phase 8)
            // Reward for: Interactions (speaking) and Neurogenesis (learning)
            let memory_metric = ego.hebbian_events as usize; // Approximation of learning
            let reward = agent.evaluate(interaction_count, memory_metric);
            
            if reward > 0.0 {
                chem.dopamine = (chem.dopamine + reward).min(1.0);
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("🏆 GOAL ACHIEVED: Dopamine +{:.2}", reward)));
                // Epiphany trigger?
                if reward >= 0.5 {
                    ego.trigger_epiphany(chem.dopamine);
                }
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
            let _collapse_threshold = 0.9 + (seed.stress_tolerance * 0.1); 
            
            // Critical Collapse Check
            // Tolerance is genetic (0.0-1.0), but we need a sanity floor.
            // If tolerance is 0.5, collapse shouldn't be at 47% adenosine, that's just a nap.
            // Let's set collapse at 90% absolute, modulated slightly by tolerance.
            let collapse_threshold = 0.95 + (seed.stress_tolerance * 0.05); 
            
            if chem.adenosine > collapse_threshold && !is_dreaming {
                 is_dreaming = true;
                 let _ = tx_thoughts.send(Thought::new(MindVoice::System, "⛔ METABOLIC CRITICAL: Forced Sleep Protocol Initiated.".to_string()));
            }
            
            // NATURAL WAKING
            if is_dreaming && chem.adenosine < 0.1 {
                is_dreaming = false;
                let _ = tx_thoughts.send(Thought::new(MindVoice::System, "🌅 WAKING: Metabolic homeostasis restored.".to_string()));
            }
        }
        
        // A.2 CORTEX (LLM) TELEMETRY
        // Read from the Neural Echo stream
        if let Some(rx) = &rx_cortex_out {
            while let Ok(out) = rx.try_recv() {
                // Downsample Logits/Echo for Visualization (32k -> 64)
                // We want a "Spectral" representation of the LLM state.
                let raw = out.neural_echo;
                let mut spectrum = vec![0.0; 64];
                if raw.len() > 0 {
                    let chunk_size = raw.len() / 64;
                    for i in 0..64 {
                        // Take average of chunk
                        let start = i * chunk_size;
                        let end = (start + chunk_size).min(raw.len());
                        let sum: f32 = raw[start..end].iter().sum();
                        spectrum[i] = (sum / (chunk_size as f32)).tanh(); // Normalize -1..1
                    }
                }
                
                // Update Web State
                if let Ok(mut state) = web_state.lock() {
                    state.llm_activity = spectrum;
                    state.activations = out.activations; // FIX: Visualize Glass Brain
                    
                    // Also capture resonance/synthesized thought if meaningful?
                    // Already handled via tx_thoughts in Planet usually, but `synthesized_thought` 
                    // is specific to the "Lobotomy" mode resonance.
                    if let Some(word) = out.synthesized_thought {
                         let clean = word.trim();
                         // Only filter pure noise (single chars, empty, pure brackets)
                         if clean.len() >= 2 && clean.chars().any(|c| c.is_alphanumeric()) {
                            telemetry_history.push_back(format!("💭 {}", clean));
                            if telemetry_history.len() > 30 { telemetry_history.pop_front(); }
                         }
                    }
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
                 is_dreaming = true;    // ENGAGE SLEEP
                 continue;
             }
             if text == "SYS:POKE" {
                 let _ = tx_thoughts.send(Thought::new(MindVoice::System, "⚡ SENSORY SHOCK. Awakening.".to_string()));
                 let mut chem = chemistry.lock().unwrap();
                 chem.adenosine = 0.0; // Reset fatigue
                 is_dreaming = false;  // WAKE UP
                 // POKE IS NOT A REWARD. It is a Startle/Alert (Norepinephrine/Cortisol).
                 // Removed dopamine spike to maintain Mechanical Honesty.
                 chem.cortisol = (chem.cortisol + 0.1).min(1.0); // Increased startle
                 // continue; // REMOVED: Allow fall-through to trigger Cortex! ← OLD LOGIC WAS WRONG
                 // FIX: POKE should wake/alert but NOT be processed as text novelty
                 continue; 
             }

             // Only log real user text, not SYS: commands (those already log their effects)
             if text.starts_with("SYS:") {
                 // Prevent ANY system command from leaking into the Cortex prompt
                 continue;
             }

             // Log user input
             let _ = tx_thoughts.send(Thought::new(MindVoice::System, format!("💬 '{}'", text)));
             // Inject into Memory/Orbit
             // For now, treat as high-entropy injection
             current_entropy += 0.1;
             
             // WAKE UP EFFECT: User attention breaks the fatigue loop
             {
                  let mut chem = chemistry.lock().unwrap();
                  chem.dopamine = (chem.dopamine + 0.3).min(1.0);  // Spike interest
                  // ADENOSINE (Fatigue) IS NOT CLEARED BY TALKING. Needs sleep.
                  // chem.adenosine = (chem.adenosine - 0.2).max(0.0); 
                  chem.cortisol = (chem.cortisol - 0.05).max(0.0);   // Social soothing (mild)
             }
             
             // Create Cortex Input for User Stimulus
             let chem = chemistry.lock().unwrap();
             let bio_context = String::new(); // No text description — chemistry flows through parametric effects
             
             let bio_status = format!("Dopa:{:.2} Cort:{:.2} Aden:{:.2}", 
                 chem.dopamine, chem.cortisol, chem.adenosine);
                 
             let input_state = CortexInput {
                 mode: crate::cortex::planet::CortexMode::Think, // Monitoring is explicit thought
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
                
                // SENSORY MOTOR MAPPING (Phase 2)
                // Hash words to Input Neurons
                let words: Vec<&str> = text.split_whitespace().collect();
                for word in words {
                    let mut hasher = DefaultHasher::new();
                    word.hash(&mut hasher);
                    let hash = hasher.finish();
                    let sensory_idx = (hash % 500) as usize;
                    
                    // Activate the sensory channel
                    if sensory_idx < current_sensory_vector.len() {
                        current_sensory_vector[sensory_idx] += 1.0; 
                    }
                }
                
                last_interaction_tick = ticks; // Reset boredom timer
                
                // SEMANTIC PERTURBATION: Text -> Chemistry (NOT prompt)
                let mut chem = chemistry.lock().unwrap();
                let friction = chem.apply_semantic_perturbation(&text);
                
                // Log the chemical impact
                if friction > 0.05 {
                    // Auditory cortex used = small growth
                    if ticks % 120 == 0 {
                        ego.neurogenesis(1);
                    }
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
            // PHASE 6: ENGRAM INJECTION
            // If the memory came with an embedding, inject it into the Association Cortex.
            // This makes memories PHYSICALLY visible as blue/purple pulses.
            if let Some(embedding) = &mem_out.embedding {
                ego.inject_embedding(embedding, crate::core::reservoir::NeuronRegion::Association);
            }

            let mut chem = chemistry.lock().unwrap();

            // Update Stats
            // session_novelty_accum += mem_out.novelty; // Unused for now
            
            if mem_out.novelty < 0.2 {
                 chem.adenosine += 0.05; // Boredom / Repetition fatigue
            } else {
                 chem.dopamine += mem_out.novelty * 0.5; // Stronger Interest spike
                 // ALSO boost curiosity if novelty is high (Reinforcement Learning)
                 seed.curiosity = (seed.curiosity + 0.001).min(1.0);
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
                // Attention (0-1) = Combination of Alertness (1-Adenosine) and Interest (Dopamine)
                // Fix: Previous logic was only (1.0 - Adenosine), causing "deafness" when tired.
                // Now, Dopamine boosts attention.
                let attention = ((1.0 - chem.adenosine) * 0.5 + chem.dopamine * 0.8).clamp(0.2, 1.0);
                
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
                    
                    let bio_context = bio_desc.clone(); // Pass biological state to prompt
    
                    // MECHANICAL HONESTY: THE GATE
                    // Use "Listen" mode (Passive) if:
                    // 1. Adenosine > 0.9 (Too tired to speak)
                    // 2. Entropy < 0.3 (Boredom/Habituation - unless Voice detected)
                    // 3. System is in Internal Monologue mode
                    
                    let mut mode = crate::cortex::planet::CortexMode::Think;
                    
                    // If we are excessively fatigued OR dreaming, we only listen/dream.
                    if chem.adenosine > 0.9 || is_dreaming {
                        mode = crate::cortex::planet::CortexMode::Listen;
                    }
                    
                    // If input is just ambient noise (not voice) and we are low entropy, 
                    // we might just ignore it (Passive).
                    // But for now, let's make all non-command inputs "Listen" first?
                    // No, let's keep it simple: Voice usually triggers Think unless suppressed.

                    let input = CortexInput {
                        mode,
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
            match rx.try_recv() {
                Ok(output) => {
                    if !output.activations.is_empty() {
                         println!("DEBUG: Received Cortex Output with {} activations", output.activations.len());
                    } else {
                         println!("DEBUG: Received Cortex Output but EMPTY activations");
                    }
                    
                    // 1. NEURAL ECHO INJECTION (The "Pebble in the Pond")
                    // The raw probability cloud hits the reservoir.
                    ego.inject_logits(&output.neural_echo);

                    // 1.5 UPDATE WEB VISUALIZATION (Top Tokens & Activations)
                    if let Ok(mut state) = web_state.lock() {
                        if !output.top_tokens.is_empty() {
                            state.top_activations = output.top_tokens.clone();
                        }
                        if !output.activations.is_empty() {
                            state.activations = output.activations.clone();
                        }
                    }
                    
                    // LATENCY FEEDBACK (Mechanical Honesty)
                    // If the thought took a long time to generate, it costs energy.
                    // 500ms is the "free" threshold. anything above adds to adenosine.
                    let latency_sec = output.inference_latency_ms as f32 / 1000.0;
                    if latency_sec > 0.5 {
                         let fatigue_cost = (latency_sec - 0.5) * 0.05; // 2s latency = +0.075 adenosine
                         let mut chem = chemistry.lock().unwrap();
                         chem.adenosine = (chem.adenosine + fatigue_cost).min(1.0);
                    }

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
                            interaction_count += 1;
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
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                     // File Log Debugging because TUI eats stdout
                     if ticks % 100 == 0 {
                         let _ = std::fs::write("daemon_debug.log", format!("Tick {}: Channel EMPTY\n", ticks));
                     }
                },
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                     let _ = std::fs::write("daemon_debug.log", "CHANNEL DISCONNECTED!\n");
                     println!("DEBUG: Cortex Channel DISCONNECTED!");
                }
            }
        } else {
            println!("DEBUG: rx_cortex_out is NONE!");
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
             
             // Get latest thought for UI stream
             let latest_thought = telemetry_history.back().cloned().unwrap_or_else(|| "Waiting for input...".to_string());
             
             // Compute expensive snapshots once
             let activity_snapshot = ego.get_activity_snapshot();
             
             // Update Web State (Shared with WebSocket Thread)
             {
                 let mut state = web_state.lock().unwrap();
                 state.dopamine = chem.dopamine;
                 state.cortisol = chem.cortisol;
                 state.adenosine = chem.adenosine;
                 state.oxytocin = chem.serotonin; // Map Serotonin -> Oxytocin
                 state.serotonin = chem.serotonin;
                 state.entropy = current_entropy;
                 state.loop_frequency = current_hz;
                 state.reservoir_activity = activity_snapshot.clone();
                 state.current_state = latest_thought.clone();
                 state.system_cpu_load = last_body_state.cpu_usage;
                 state.system_cpu_load = last_body_state.cpu_usage;
                 state.system_ram_gb = last_body_state.ram_usage; // using field for load
                 
                 // Genome Traits
                 state.curiosity = seed.curiosity;
                 state.stress_tolerance = seed.stress_tolerance;
                 state.generation = seed.generation;
             }

             let packet = AlephPacket::Telemetry {
                 adenosine: chem.adenosine,
                 cortisol: chem.cortisol,
                 dopamine: chem.dopamine,
                 oxytocin: chem.serotonin, 
                 audio_spectrum: last_spectrum.clone(),
                 heart_rate: last_body_state.cpu_usage,
                 lucidity: 1.0 - last_body_state.ram_usage, 
                 reservoir_activity: activity_snapshot,
                 short_term_memory: telemetry_history.iter().cloned().collect(),
                 current_state: latest_thought, 
                 entropy: current_entropy,
                 loop_frequency: current_hz,
                 cpu_usage: last_body_state.cpu_usage,
                 activations: {
                     let state = web_state.lock().unwrap();
                     state.activations.clone()
                 },
                 visual_cortex: {
                     let state = web_state.lock().unwrap();
                     state.visual_cortex.clone()
                 },
                 region_map: ego.get_region_map(),
                 reservoir_size: ego.current_size(),
                 neuron_positions: ego.get_positions().clone(),
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

        // E. IDLE STATE (The Dreaming)
        // If Cortex hasn't been stimulated in a while, force a "Listen" pulse to keep the Neural Echo active.
        // This stops the "Mind" visualization from disappearing.
        if ticks % 12 == 0 { // ~5Hz Pulse
             // println!("DEBUG: Pulse Triggered"); // Debugging
             // Check if we need to poke the planet
             // Ideally we'd track `last_cortex_input`, but a constant low-frequency pulse is fine.
             // It overlaps with real input but that's okay (Parallel processing simulation).
             
             let chem = chemistry.lock().unwrap();
             let bio_context = String::new(); // No text — idle scan uses parametric effects only
             let input = CortexInput {
                 mode: crate::cortex::planet::CortexMode::Listen,
                 text: "scan".to_string(), // Minimal token to pump the model
                 bio_state: "Idle".to_string(),
                 bio_context,
                 _somatic_state: "Idle".to_string(),
                 _long_term_memory: None,
                 _cpu_load: last_body_state.cpu_usage,
                 _ram_pressure: last_body_state.ram_usage,
                 _cognitive_impairment: 0.0,
                 entropy: current_entropy,
                 adenosine: chem.adenosine,
                 dopamine: chem.dopamine,
                 cortisol: chem.cortisol,
                 _oxytocin: chem.oxytocin,
                 temperature_clamp: None,
             };
             if let Some(tx) = &tx_cortex {
                  let _ = tx.send(input);
             }
        }

        // F. SPONTANEOUS AGENCY (The Ghost in the Machine)
        // DYNAMIC PACING: The more excited (Dopamine), the faster it speaks.
        let mut chem = chemistry.lock().unwrap();
        
        let interest = chem.dopamine;
        let energy = 1.0 - chem.adenosine;
        
        // Calculate dynamic threshold based on excitement
        // Dopa 0.9 -> Delay 300 ticks (5s) "Manic"
        // Dopa 0.5 -> Delay 1200 ticks (20s) "Conversation"
        // Dopa < 0.4 -> No Agency "Passive"
        let agency_delay = if interest > 0.8 { 
            300 // 5 seconds
        } else if interest > 0.5 {
            1200 // 20 seconds
        } else {
            999999 // Effectively infinite
        };
        
        let silence_duration = ticks.saturating_sub(last_interaction_tick);
        
        // Random organic trigger (stochastic firing)
        // 10% chance per second to speak if threshold met
        let stochastic = (ticks % 60) == 0; 
        
        if stochastic && silence_duration > agency_delay && energy > 0.2 {
             // ... Speak ...
             let _ = tx_thoughts.send(Thought::new(MindVoice::System, 
                 format!("⚡ AGENCY: Interest {:.2} > Speaking (Silence {}s)", interest, silence_duration/60)));
             
             let input = CortexInput {
                 mode: crate::cortex::planet::CortexMode::Think,
                 text: "".to_string(), 
                 bio_state: format!("Interest:{:.2}", interest),
                 bio_context: String::new(),
                 _somatic_state: "Active".to_string(),
                 _long_term_memory: None,
                 _cpu_load: last_body_state.cpu_usage,
                 _ram_pressure: last_body_state.ram_usage,
                 _cognitive_impairment: 0.0,
                 entropy: current_entropy,
                 adenosine: chem.adenosine,
                 dopamine: chem.dopamine,
                 cortisol: chem.cortisol,
                 _oxytocin: chem.oxytocin,
                 temperature_clamp: None,
             };
             if let Some(tx) = &tx_cortex {
                  let _ = tx.send(input);
             }
             
             // Self-sustain excitement if talking
             chem.dopamine = (chem.dopamine + 0.02).min(1.0);
             
             last_interaction_tick = ticks;
        }
        drop(chem);
        
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


