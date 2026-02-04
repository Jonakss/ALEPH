use anyhow::Result;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use crate::core::ipc::AlephPacket;
use crate::senses::ears::AudioSpectrum;
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
use std::sync::mpsc;
use std::thread;

pub fn run() -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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
        audio_spectrum: AudioSpectrum::default(),
        heart_rate: 0.0,
        lucidity: 1.0,
        short_term_memory: vec!["Connecting...".to_string()],
        current_state: "Unknown".to_string(),
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
        let mut chunk = [0u8; 1024];
        if let Ok(n) = stream.read(&mut chunk) {
            if n > 0 {
                net_buffer.extend_from_slice(&chunk[..n]);
                // Try to parse lines
                // Very naive implementation for TUI demo
                if let Ok(s) = String::from_utf8(net_buffer.clone()) {
                    if let Some(pos) = s.rfind('\n') {
                        let lines: Vec<&str> = s[..pos].split('\n').collect();
                        if let Some(last_line) = lines.last() {
                            if !last_line.is_empty() {
                                if let Ok(packet) = serde_json::from_str::<AlephPacket>(last_line) {
                                    last_packet = packet;
                                }
                            }
                        }
                        // Clear processed part
                        net_buffer = s[pos+1..].as_bytes().to_vec();
                    }
                }
            }
        }

        // B. Drawing
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3), // Header
                        Constraint::Length(8), // Biological Stats
                        Constraint::Min(1),    // Narrative Stream
                        Constraint::Length(3), // Input
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // 0. Extract Data First
            let (aden, cort, dopa, spec) = match &last_packet {
                AlephPacket::Telemetry { adenosine, cortisol, dopamine, audio_spectrum, .. } => 
                    (*adenosine, *cortisol, *dopamine, audio_spectrum.clone()),
                _ => (0.0, 0.0, 0.0, AudioSpectrum::default()),
            };
            
            // Audio Logic
            let val_bass = spec.bass.clamp(0.0, 1.0);
            let val_mids = spec.mids.clamp(0.0, 1.0);
            let val_highs = spec.highs.clamp(0.0, 1.0);
            let val_rms = (spec.rms * 10.0).clamp(0.0, 1.0);
            let hearing_status = if val_rms > 0.1 { "LISTENING" } else { "SILENT" };

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
            
            let title_text = format!("ALEPH v0.1 | STATUS: {} | EAR: {}", packet_data, hearing_status);
            let title = Paragraph::new(title_text)
                .style(Style::default().fg(if val_rms > 0.1 { Color::Green } else { Color::Cyan }).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);
            

            let bio_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)].as_ref())
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


            // 3. Narrative Stream
            let messages: Vec<ListItem> = match &last_packet {
                AlephPacket::Telemetry { short_term_memory, .. } => {
                    short_term_memory.iter().rev().map(|m| {
                         ListItem::new(Line::from(vec![Span::raw(m)]))
                    }).collect()
                },
                _ => vec![],
            };
            let logs = List::new(messages)
                .block(Block::default().borders(Borders::ALL).title("Thought Stream"));
            f.render_widget(logs, chunks[2]);

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
