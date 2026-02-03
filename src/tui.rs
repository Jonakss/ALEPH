use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, Paragraph, Wrap},
    Frame, Terminal,
};
use std::sync::mpsc::Receiver;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

use crate::core::thought::Thought;

// Estructura de Telemetría que viene del Backend
pub struct Telemetry {
    pub audio_spectrum: crate::senses::ears::AudioSpectrum, // Full Spectrum
    pub entropy: f32,         // 0.0 - 1.0
    pub neuron_active_count: usize, // Memories (Vectors)
    pub system_status: String,// "FLOW", "PANIC", etc.
    pub last_entropy_delta: f32, // Cambio de entropía
    pub fps: f64,             // Backend ticks per second
    pub cpu_load: f32,        // Proprioception
    pub ram_load: f32,        // Proprioception
    pub log_message: Option<String>, // Mensajes del Observador
    pub adenosine: f32, // Sleep Pressure
    pub dopamine: f32,  // Reward
    pub cortisol: f32,  // Stress
    pub insight_intensity: f32, // 0.0 - 1.0 (Flash trigger)
    pub thoughts: Vec<Thought>, // Stream of Consciousness
    pub activity_map: Vec<f32>, // Neuronal activity (100 neurons, 0.0-1.0)
    pub novelty_score: f32, // Last novelty check result
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            audio_spectrum: crate::senses::ears::AudioSpectrum::default(),
            entropy: 0.0,
            neuron_active_count: 0,
            system_status: "INIT".to_string(),
            last_entropy_delta: 0.0,
            fps: 0.0,
            cpu_load: 0.0,
            ram_load: 0.0,
            log_message: None,
            adenosine: 0.0,
            dopamine: 0.5,
            cortisol: 0.0,
            insight_intensity: 0.0,
            thoughts: Vec::new(),
            activity_map: vec![0.0; 100],
            novelty_score: 0.0,
        }
    }
}

// Legacy run_tui removed. Functionality handled by main.rs.

mod avatar;
mod monologue;

// ...

pub fn ui(
    f: &mut Frame,
    telemetry: &Telemetry,
    // audio_history removed
    entropy_history: &[(f64, f64)],
    curr_time: f64,
    window_width: f64,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35), // Input & Reservoir
            Constraint::Percentage(35), // Status & Logs
            Constraint::Percentage(30), // Monologue (Stream of Consciousness)
        ])
        .split(f.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Audio Left
            Constraint::Percentage(55), // Reservoir Center
            Constraint::Percentage(15), // Avatar Right 
        ])
        .split(chunks[0]);
    
    // ... (Top Panel Render matches previous) ...
    // Note: I need to preserve the render logic for top_chunks, which I didn't replace.
    // The previous tool call ended at line 271, so I am replacing the TOP part of function `ui`.


    // --- PANEL IZQ: AUDIO SPECTRUM (FFT) ---
    let audio_block = Block::default()
        .title(" 👂 ACOUSTIC SPECTRUM ")
        .borders(Borders::ALL);

    // Prepare Data for Gradient Bars
    let spectrum = &telemetry.audio_spectrum;
    
    // Normalize values 0.0 - 1.0 (FFT already normalized in ears.rs)
    let val_bass = spectrum.bass.clamp(0.0, 1.0);
    let val_mids = spectrum.mids.clamp(0.0, 1.0);
    let val_highs = spectrum.highs.clamp(0.0, 1.0);
    let val_rms = (spectrum.rms * 10.0).clamp(0.0, 1.0); // RMS is typically 0.0-0.1

    // Generate gradient bar with color intensity
    let make_bar = |value: f32, label: &str| -> Line {
        let bar_width = 12;
        let filled = (value * bar_width as f32) as usize;
        
        let color = if value < 0.33 {
            Color::Cyan
        } else if value < 0.66 {
            Color::Yellow
        } else {
            Color::Red
        };
        
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
        
        Line::from(vec![
            Span::styled(format!("{:4} ", label), Style::default().fg(Color::White)),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(format!(" {:3.0}%", value * 100.0), Style::default().fg(Color::DarkGray)),
        ])
    };

    let audio_lines = vec![
        make_bar(val_bass, "BASS"),
        make_bar(val_mids, "MIDS"),
        make_bar(val_highs, "HIGH"),
        make_bar(val_rms, " RMS"),
    ];

    let audio_paragraph = Paragraph::new(audio_lines)
        .block(audio_block)
        .alignment(ratatui::layout::Alignment::Left);
        
    f.render_widget(audio_paragraph, top_chunks[0]);

    // --- PANEL CENTER: RESERVOIR STATE ---
    let reservoir_block = Block::default()
        .title(" 🧠 ENTROPY ")
        .borders(Borders::ALL);
    
    let datasets = vec![
        Dataset::default()
            .name("Entropy")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Magenta))
            .data(entropy_history),
    ];

    let chart = Chart::new(datasets)
        .block(reservoir_block)
        .x_axis(Axis::default()
            .bounds([curr_time - window_width, curr_time])
            .labels(vec![Span::raw("")]))
        .y_axis(Axis::default()
            .bounds([0.0, 1.2])
            .labels(vec![Span::raw("0.0"), Span::raw("1.0")])); // Min 2 labels to avoid div/0
    f.render_widget(chart, top_chunks[1]);

    // --- PANEL RIGHT: AVATAR ---
    let face = avatar::get_face(telemetry);
    
    // Glitch Effect (High Entropy)
    let avatar_border_style = if telemetry.entropy > 0.8 {
        Style::default().fg(Color::Red).add_modifier(Modifier::RAPID_BLINK)
    } else {
        Style::default()
    };

    let avatar_widget = Paragraph::new(face.ascii)
        .block(Block::default()
            .title(" ALEPH ")
            .borders(Borders::ALL)
            .border_style(avatar_border_style))
        .style(Style::default().fg(face.color).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center); // Centrado
        
    f.render_widget(avatar_widget, top_chunks[2]);

    // --- PANEL CENTRAL: STATUS & LOGS ---
    // Split Chunk[1] into Status (Up) and Logs (Down) or Left/Right?
    // Let's do Left (Metrics) and Right (Logs).
    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Metrics
            Constraint::Percentage(40), // Observer Log
        ])
        .split(chunks[1]);

    // ... (Render Gauges & Text Stats to mid_chunks[0]) ...
    // NOTE: Need to verify if I can easily splice this. 
    // The previous code had a specific structure for status_chunks. I will adapt.
     
    // Gauges
    let entropy_percent = (telemetry.entropy * 100.0).clamp(0.0, 100.0) as u16;
    let gauge_color = match telemetry.system_status.as_str() {
        "PANIC" => Color::Red,
        "MAC. ZONE" => Color::Blue,
        "DREAMING" => Color::Magenta, // Sueño = Púrpura
        "ONLINE" => Color::Green,
        _ => Color::Green,
    };
    let status_style = Style::default().fg(gauge_color).add_modifier(Modifier::BOLD);

    let gauge = Gauge::default()
        .block(Block::default().title(" System Entropy ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(gauge_color))
        .percent(entropy_percent);
    
    // Inner split for Gauge + Text in Left Panel?
    let status_left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(mid_chunks[0]);
        
    f.render_widget(gauge, status_left_chunks[0]);

    // Derived Metrics & Text Stats (Use previous logic)
    // Helper for visual bars
    let make_bar = |val: f32, color: Color| -> Span {
        let width: usize = 15;
        let filled = (val.clamp(0.0, 1.0) * width as f32) as usize;
        let empty = width.saturating_sub(filled);
        let s = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        Span::styled(s, Style::default().fg(color))
    };

    let mut text = vec![
        Line::from(vec![Span::raw("System Status: "), Span::styled(telemetry.system_status.clone(), status_style)]),
        Line::from(vec![Span::raw(format!("Tick Rate: {:.1} Hz", telemetry.fps))]),
        Line::from(vec![Span::raw(format!("Brain Size: {} neurons", telemetry.neuron_active_count))]),
        Line::from(""),
        Line::from(vec![Span::styled("--- NEURO-METABOLISM ---", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![
            Span::raw("DOPAMINE: "),
            make_bar(telemetry.dopamine, Color::Cyan),
            Span::raw(format!(" {:.0}% (Interest)", telemetry.dopamine * 100.0)),
        ]),
        Line::from(vec![
            Span::raw("CORTISOL: "),
            make_bar(telemetry.cortisol, Color::Magenta),
            Span::raw(format!(" {:.0}% (Stress)", telemetry.cortisol * 100.0)),
        ]),
        Line::from(vec![
            Span::raw("ADENOSINE:"),
            make_bar(telemetry.adenosine, Color::DarkGray),
            Span::raw(format!(" {:.0}% (Fatigue)", telemetry.adenosine * 100.0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("NOVELTY:  "),
            make_bar(1.0 - telemetry.novelty_score, Color::Yellow), // Inverse: low sim = novel
            Span::raw(format!(" {:.0}% (Surprise)", (1.0 - telemetry.novelty_score) * 100.0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("CPU Load: "),
            make_bar(telemetry.cpu_load / 100.0, if telemetry.cpu_load > 80.0 { Color::Red } else { Color::Green }),
            Span::raw(format!(" {:.1}%", telemetry.cpu_load)),
        ]),
        Line::from(vec![
            Span::raw("RAM Load: "),
            make_bar(telemetry.ram_load, if telemetry.ram_load > 0.9 { Color::Red } else { Color::Green }),
            Span::raw(format!(" {:.1}%", telemetry.ram_load * 100.0)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("--- NEURAL ACTIVITY (10x10) ---", Style::default().add_modifier(Modifier::BOLD))]),
    ];
    
    // Build 10x10 heatmap from activity_map (100 neurons)
    for row in 0..10 {
        let mut spans: Vec<Span> = Vec::new();
        for col in 0..10 {
            let idx = row * 10 + col;
            let val = telemetry.activity_map.get(idx).copied().unwrap_or(0.0);
            let block = if val < 0.25 { "░" } else if val < 0.5 { "▒" } else if val < 0.75 { "▓" } else { "█" };
            let color = if val < 0.33 { Color::DarkGray } else if val < 0.66 { Color::Cyan } else { Color::Yellow };
            spans.push(Span::styled(block, Style::default().fg(color)));
        }
        text.push(Line::from(spans));
    }
    let stats = Paragraph::new(text)
        .block(Block::default().title(" Telemetry ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(stats, status_left_chunks[1]);

    // --- PANEL RIGHT (MID): LOGS ---
    let log_text = telemetry.log_message.clone().unwrap_or_else(|| "System Nominal.".to_string());
    let logs = Paragraph::new(format!("OBSERVER LOG:\n{}", log_text))
        .block(Block::default().title(" Logs ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(logs, mid_chunks[1]);

    // --- PANEL INFERIOR (BOTTOM): MONOLOGUE ---
    let monologue_widget = monologue::render_monologue(&telemetry.thoughts, telemetry.insight_intensity);
    f.render_widget(monologue_widget, chunks[2]);

    // --- PANEL INFERIOR 2: MONOLOGUE ---
    // Wait, chunks only has 3 items. I need to resize chunks or use a new area.
    // Let's repurpose chunks[2] for logs AND monologue? Or split chunks[2].
}
