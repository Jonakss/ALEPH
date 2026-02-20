use anyhow::Result;
use std::io::{self, Read, Write};
use rand::Rng; // For Glitching
use std::os::unix::net::UnixStream;
use std::time::Duration;
use crate::core::ipc::AlephPacket;
use crate::senses::ears::AudioSpectrum;
use crate::tui::avatar::{self};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph, Gauge},
    Terminal,
};
use std::thread;

pub fn run() -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // SAFETY: Panic Hook to restore terminal if we crash
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        default_hook(info);
    }));

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut last_tick = std::time::Instant::now();
    let mut frames = 0;
    let mut _fps = 0.0;
    let _start_time = std::time::Instant::now();
    let mut entropy_history: Vec<(f64, f64)> = Vec::new();
    let _window_width = 60.0; // 60 seconds of history

    // 2. Connect to Nervous System
    let socket_path = "/tmp/aleph.sock";
    println!("🔌 Connecting to Nervous System at {}...", socket_path);
    
    // Retry loop for connection
    let mut stream = loop {
        match UnixStream::connect(socket_path) {
            Ok(s) => break s,
            Err(_) => {
                thread::sleep(Duration::from_millis(500));
                // We'll just wait, maybe show a "Connecting..." screen in future
                continue;
            }
        }
    };
    stream.set_nonblocking(true)?;
    
    // 3. State
    let mut last_packet = AlephPacket::Telemetry {
        adenosine: 0.0,
        cortisol: 0.0,
        dopamine: 0.0,
        oxytocin: 0.0,
        audio_spectrum: AudioSpectrum::default(),
        heart_rate: 0.0,
        lucidity: 1.0,
        entropy: 0.0,
        reservoir_activity: vec![0.0; 500],
        short_term_memory: vec!["Connecting...".to_string()],
        current_state: "Unknown".to_string(),
        loop_frequency: 60.0,
        cpu_usage: 0.0,
        activations: Vec::new(),
        region_map: Vec::new(),
        visual_cortex: Vec::new(),
        
        // Spatial Topology (Real backend positions)
        neuron_positions: Vec::new(),
    };
    
    // Input Buffer
    let mut input_buffer = String::new();
    
    // Packet Receiver Thread logic is implicit effectively by polling non-blocking stream
    // Actually, reading from stream is stream-based, so we might get partial JSON.
    // For specific "lines", we need a BufReader, but that blocks. 
    // Let's use a simpler approach: Read to buffer, split by newline.
    let mut net_buffer = Vec::new();

    // 4. Main Event Loop
    loop {
        // A. Network Read (Non-blocking)
        let mut chunk = [0u8; 16384]; // 16KB Buffer for Neural State
        if let Ok(n) = stream.read(&mut chunk) {
            if n > 0 {
                net_buffer.extend_from_slice(&chunk[..n]);
                // Try to parse lines using lossy to avoid UTF-8 crash on boundary
                let s = String::from_utf8_lossy(&net_buffer).to_string();
                if let Some(pos) = s.rfind('\n') {
                    let lines: Vec<&str> = s[..pos].split('\n').collect();
                    if let Some(last_line) = lines.last() {
                        if !last_line.is_empty() {
                            match serde_json::from_str::<AlephPacket>(last_line) {
                                Ok(packet) => {
                                    last_packet = packet;
                                    // Update Entropy History
                                    if let AlephPacket::Telemetry { entropy, .. } = &last_packet {
                                        entropy_history.push((last_tick.elapsed().as_secs_f64(), *entropy as f64)); // Use ELAPSED time not absolute
                                        // Keep roughly last 100 points or based on time?
                                        if entropy_history.len() > 200 {
                                             entropy_history.remove(0);
                                        }
                                    }
                                },
                                Err(e) => {
                                    // Inject error into state for visibility
                                    if let AlephPacket::Telemetry { adenosine, cortisol, dopamine, oxytocin, audio_spectrum, heart_rate, lucidity, reservoir_activity, short_term_memory, .. } = &last_packet {
                                         last_packet = AlephPacket::Telemetry {
                                            adenosine: *adenosine,
                                            cortisol: *cortisol,
                                            dopamine: *dopamine,
                                            oxytocin: *oxytocin,
                                            audio_spectrum: audio_spectrum.clone(),
                                            heart_rate: *heart_rate,
                                            lucidity: *lucidity,
                                            reservoir_activity: reservoir_activity.clone(),
                                            short_term_memory: short_term_memory.clone(),
                                            current_state: "Decoding Error".to_string(),
                                            entropy: 0.0, // Placeholder
                                            loop_frequency: *loop_frequency,
                                            cpu_usage: *cpu_usage,
                                            activations: Vec::new(),
                                            region_map: Vec::new(),
                                            reservoir_size: *reservoir_size,
                                            visual_cortex: Vec::new(),
                                            neuron_positions: Vec::new(),
                                        };
                                    }
                                }
                            }
                        }
                    }
                    // Clear processed part
                    net_buffer = s[pos+1..].as_bytes().to_vec();
                } else if net_buffer.len() > 65536 {
                     // Safety drain if no newline found for a long time
                     net_buffer.clear();
                }
                }
            }


        // Calculate FPS
        frames += 1;
        if last_tick.elapsed().as_secs_f32() >= 1.0 {
            _fps = frames as f32 / last_tick.elapsed().as_secs_f32();
            frames = 0;
            last_tick = std::time::Instant::now();
        }

        // B. Drawing
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(7), // Header (Increased for Avatar)
                        Constraint::Length(8), // Biological Stats
                        Constraint::Min(1),    // Narrative Stream
                        Constraint::Length(3), // Input
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // 0. Extract Data First
            let (aden, cort, dopa, oxy, spec, neurons, current_hz) = match &last_packet {
                AlephPacket::Telemetry { adenosine, cortisol, dopamine, oxytocin, audio_spectrum, reservoir_activity, loop_frequency, .. } => 
                    (*adenosine, *cortisol, *dopamine, *oxytocin, audio_spectrum.clone(), reservoir_activity.clone(), *loop_frequency),
                _ => (0.0, 0.0, 0.0, 0.0, AudioSpectrum::default(), vec![], 60.0),
            };
            
            // Audio Logic
            let val_bass = (spec.bass * 500.0).clamp(0.0, 1.0);
            let val_mids = (spec.mids * 500.0).clamp(0.0, 1.0);
            let val_highs = (spec.highs * 500.0).clamp(0.0, 1.0);
            let val_rms = (spec.rms * 1000.0).clamp(0.0, 1.0); // Sensitive scaling (was * 10.0)
            let hearing_status = if spec.rms > 0.001 { "LISTENING" } else { "SILENT" };

             let make_bar_audio = |value: f32, label: &str| -> Line {
                let bar_width = 10;
                let filled = (value * bar_width as f32) as usize;
                let color = if value < 0.33 { Color::Cyan } else if value < 0.66 { Color::Yellow } else { Color::Red };
                let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                Line::from(vec![
                    Span::styled(format!("{:4} ", label), Style::default().fg(Color::White)),
                    Span::styled(bar, Style::default().fg(color)),
                ])
            };

            // 1. Header
            let packet_data = match &last_packet {
                AlephPacket::Telemetry { current_state, .. } => current_state.clone(),
                _ => "Unknown".to_string(),
            };
            
            // Truncate status to avoid breaking layout
            let status_trimmed = if packet_data.chars().count() > 25 { 
                format!("{}...", packet_data.chars().take(24).collect::<String>()) 
            } else { 
                packet_data.clone() 
            };
            
            let title_text = format!("ALEPH v2.0 | {} Hz | ST: {} | EAR: {} ({:.4})", 
                current_hz as u32, status_trimmed, hearing_status, spec.rms);
            
            // Generate Avatar
            // We need to construct a temp struct for the helper or update the helper.
            // Let's make a clear mapping.
            let face_telemetry = crate::tui::Telemetry {
                adenosine: aden,
                cortisol: cort,
                dopamine: dopa,
                system_status: packet_data.clone(),
                ..Default::default()
            };
            let face = avatar::get_face(&face_telemetry);

            // Split Header: Left (Info/EQ), Right (Face)
            let header_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(20), Constraint::Length(20)].as_ref())
                .split(chunks[0]);

            let title_paragraph = Paragraph::new(vec![
                Line::from(title_text),
                make_bar_audio(val_bass, "BASS"),
                make_bar_audio(val_mids, "MIDS"),
                make_bar_audio(val_highs, "HIGH"),
            ])
            .style(Style::default().fg(if val_rms > 0.1 { Color::Green } else { Color::Cyan }).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(title_paragraph, header_layout[0]);
            
            let face_widget = Paragraph::new(face.ascii)
                .style(Style::default().fg(face.color).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("Avatar"));
            f.render_widget(face_widget, header_layout[1]);
            

            let bio_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)].as_ref())
                .split(chunks[1]);

            let g_aden = Gauge::default()
                .block(Block::default().title("Adenosine (Fatigue)").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent((aden * 100.0) as u16);
            f.render_widget(g_aden, bio_chunks[0]);

            let g_cort = Gauge::default()
                .block(Block::default().title("Cortisol (Stress)").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Red))
                .percent((cort * 100.0) as u16);
            f.render_widget(g_cort, bio_chunks[1]);
            
             let g_dopa = Gauge::default()
                .block(Block::default().title("Dopamine (Interest)").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green))
                .percent((dopa * 100.0) as u16);
            f.render_widget(g_dopa, bio_chunks[2]);

             let g_oxy = Gauge::default()
                .block(Block::default().title("Oxytocin (Trust)").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Blue))
                .percent((oxy * 100.0) as u16);
            f.render_widget(g_oxy, bio_chunks[3]);


            // 3. Body: Stream (Left) + Neocortex (Right)
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                .split(chunks[2]);

            // Narrative Stream (Chat Style: Oldest Top, Newest Bottom)
            let messages: Vec<ListItem> = match &last_packet {
                AlephPacket::Telemetry { short_term_memory, adenosine, cortisol, .. } => {
                    let mut rng = rand::thread_rng();
                    short_term_memory.iter().enumerate().map(|(i, m)| {
                         // 1. FADING (Adenosine)
                         // Older messages fade more? Or global fatigue fades everything?
                         // "Global Fatigue": High Adenosine dims the entire consciousness stream.
                         // But also, older memories drift into the dark.
                         
                         let fatigue_dim = (1.0 - *adenosine).max(0.2); // 1.0 -> 0.2 brightness
                         // Also fade based on index (older = dimmer)
                         let recency = (i as f32) / (short_term_memory.len() as f32).max(1.0);
                         let brightness = (fatigue_dim * recency).clamp(0.2, 1.0);
                         
                         let color_val = (255.0 * brightness) as u8;
                         let base_color = Color::Rgb(color_val, color_val, color_val);
                         
                         // 2. GLITCHING (Cortisol / Structural Tremor)
                         // If cortisol is high, text characters might "shift".
                         let display_text = if *cortisol > 0.3 && rng.gen::<f32>() < (*cortisol - 0.3) {
                             // Glitch: Replace random chars
                             m.chars().map(|c| {
                                 if rng.gen::<f32>() < 0.1 {
                                     let glitches = ['!', '?', '#', '@', '%', '&', '*', '0', '1'];
                                     glitches[rng.gen_range(0..glitches.len())]
                                 } else {
                                     c
                                 }
                             }).collect::<String>()
                         } else {
                             m.clone()
                         };

                         ListItem::new(Line::from(vec![Span::styled(display_text, Style::default().fg(base_color))]))
                    }).collect()
                },
                _ => vec![],
            };
            let logs = List::new(messages)
                .block(Block::default().borders(Borders::ALL).title("Consciousness Stream"));
            f.render_widget(logs, body_chunks[0]);
            
            // Neocortex Visualization (Heatmap)
            // 500 neurons. 25 cols x 20 rows roughly.
            let mut neuron_spans = Vec::new();
            let cols = 25;
            for (i, &activity) in neurons.iter().enumerate() {
                if i >= 500 { break; } // Safety
                let color = if activity > 0.8 { Color::Red } 
                           else if activity > 0.5 { Color::Magenta }
                           else if activity > 0.2 { Color::Cyan }
                           else { Color::DarkGray };
                
                let char = if activity > 0.5 { "■" } else { "▪" };
                neuron_spans.push(Span::styled(char, Style::default().fg(color)));
                
                if (i + 1) % cols == 0 {
                    neuron_spans.push(Span::raw("\n"));
                } else {
                    neuron_spans.push(Span::raw(" "));
                }
            }
            // Group spans into lines? Paragraph takes Spans? No, text takes Lines.
            // Let's reconstruct into Lines.
            let mut neuron_lines = Vec::new();
            let mut current_line = Vec::new();
             for (i, &activity) in neurons.iter().enumerate() {
                if i >= 500 { break; }
                let color = if activity > 0.8 { Color::Red } 
                           else if activity > 0.5 { Color::Magenta }
                           else if activity > 0.2 { Color::Cyan }
                           else { Color::DarkGray };
                let char = if activity > 0.5 { "■" } else { "▪" };
                
                current_line.push(Span::styled(char, Style::default().fg(color)));
                current_line.push(Span::raw(" "));
                
                 if (i + 1) % cols == 0 {
                    neuron_lines.push(Line::from(current_line.clone()));
                    current_line.clear();
                }
            }
            if !current_line.is_empty() { neuron_lines.push(Line::from(current_line)); }
            
            let cortex_widget = Paragraph::new(neuron_lines)
                .block(Block::default().title("Neocortex (Reservoir)").borders(Borders::ALL));
            f.render_widget(cortex_widget, body_chunks[1]);

            // 4. Input
            let input = Paragraph::new(format!("> {}", input_buffer))
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Perturb System"));
            f.render_widget(input, chunks[3]);

        })?;

        // C. Input Handling
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => input_buffer.push(c),
                    KeyCode::Backspace => { input_buffer.pop(); },
                    KeyCode::Enter => {
                        // Send Stimulus
                        let stim = AlephPacket::Stimulus { text: input_buffer.clone(), force: 1.0 };
                        if let Ok(json) = serde_json::to_string(&stim) {
                            let msg = format!("{}\n", json);
                            let _ = stream.write_all(msg.as_bytes()); 
                        }
                        input_buffer.clear();
                    },
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
