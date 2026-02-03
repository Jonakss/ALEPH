use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph, Wrap},
    Frame,
};

use crate::core::thought::Thought;

// Estructura de Telemetría que viene del Backend
pub struct Telemetry {
    pub audio_spectrum: crate::senses::ears::AudioSpectrum, // Full Spectrum
    pub entropy: f32,         // 0.0 - 1.0
    pub neuron_active_count: usize, // Memories (Vectors)
    pub system_status: String,// "FLOW", "PANIC", etc.
    #[allow(dead_code)]
    pub last_entropy_delta: f32, // Cambio de entropía (reservado para Variable Metabolism)
    pub fps: f64,             // Backend ticks per second
    pub cpu_load: f32,        // Proprioception
    pub ram_load: f32,        // Proprioception
    pub adenosine: f32, // Sleep Pressure
    pub dopamine: f32,  // Reward
    pub cortisol: f32,  // Stress
    pub insight_intensity: f32, // 0.0 - 1.0 (Flash trigger)
    pub thoughts: Vec<Thought>, // Stream of Consciousness
    pub logs: Vec<String>,     // Observer Logs
    pub activity_map: Vec<f32>, // Neuronal activity (100 neurons, 0.0-1.0)
    pub novelty_score: f32, // Last novelty check result
    pub reservoir_state: String, // Description of reservoir mood
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
            adenosine: 0.0,
            dopamine: 0.5,
            cortisol: 0.0,
            insight_intensity: 0.0,
            thoughts: Vec::new(),
            logs: Vec::new(),
            activity_map: vec![0.0; 100],
            novelty_score: 0.0,
            reservoir_state: "Estable".to_string(),
        }
    }
}

mod avatar;
mod monologue;

pub fn ui(
    f: &mut Frame,
    telemetry: &Telemetry,
    entropy_history: &[(f64, f64)],
    curr_time: f64,
    window_width: f64,
    log_scroll: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Top: Audio, Entropy, Avatar
            Constraint::Percentage(60), // Bottom: Metrics (L) + Timeline (R)
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
    
    // --- PANEL IZQ: AUDIO SPECTRUM (FFT) ---
    let audio_block = Block::default()
        .title(" 👂 ACOUSTIC SPECTRUM ")
        .borders(Borders::ALL);

    let spectrum = &telemetry.audio_spectrum;
    let val_bass = spectrum.bass.clamp(0.0, 1.0);
    let val_mids = spectrum.mids.clamp(0.0, 1.0);
    let val_highs = spectrum.highs.clamp(0.0, 1.0);
    let val_rms = (spectrum.rms * 10.0).clamp(0.0, 1.0);

    let make_bar_audio = |value: f32, label: &str| -> Line {
        let bar_width = 12;
        let filled = (value * bar_width as f32) as usize;
        let color = if value < 0.33 { Color::Cyan } else if value < 0.66 { Color::Yellow } else { Color::Red };
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
        Line::from(vec![
            Span::styled(format!("{:4} ", label), Style::default().fg(Color::White)),
            Span::styled(bar, Style::default().fg(color)),
            Span::styled(format!(" {:3.0}%", value * 100.0), Style::default().fg(Color::DarkGray)),
        ])
    };

    let audio_paragraph = Paragraph::new(vec![
        make_bar_audio(val_bass, "BASS"),
        make_bar_audio(val_mids, "MIDS"),
        make_bar_audio(val_highs, "HIGH"),
        make_bar_audio(val_rms, " RMS"),
    ]).block(audio_block);
    f.render_widget(audio_paragraph, top_chunks[0]);

    // --- PANEL CENTER: RESERVOIR STATE ---
    let chart = Chart::new(vec![
        Dataset::default()
            .name("Entropy")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Magenta))
            .data(entropy_history),
    ])
    .block(Block::default().title(" 🧠 ENTROPY ").borders(Borders::ALL))
    .x_axis(Axis::default().bounds([curr_time - window_width, curr_time]).labels(vec![Span::raw("")]))
    .y_axis(Axis::default().bounds([0.0, 1.2]).labels(vec![Span::raw("0.0"), Span::raw("1.0")]));
    f.render_widget(chart, top_chunks[1]);

    // --- PANEL RIGHT: AVATAR ---
    let face = avatar::get_face(telemetry);
    let avatar_widget = Paragraph::new(face.ascii)
        .block(Block::default()
            .title(" ALEPH ")
            .borders(Borders::ALL)
            .border_style(if telemetry.entropy > 0.8 { Style::default().fg(Color::Red).add_modifier(Modifier::RAPID_BLINK) } else { Style::default() }))
        .style(Style::default().fg(face.color).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(avatar_widget, top_chunks[2]);

    // --- BOTTOM PANEL: Metrics (Left) + Timeline (Right) ---
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Telemetry & Neuro-metabolism
            Constraint::Percentage(60), // Unified Neural Timeline
        ])
        .split(chunks[1]);

    // METRICS RENDER
    let gauge_color = match telemetry.system_status.as_str() {
        "PANIC" => Color::Red,
        "DREAMING" => Color::Magenta,
        _ => Color::Green,
    };
    
    let make_val_bar = |val: f32, color: Color| -> Span {
        let width: usize = 12;
        let filled = (val.clamp(0.0, 1.0) * width as f32) as usize;
        let s = format!("{}{}", "█".repeat(filled), "░".repeat(width - filled));
        Span::styled(s, Style::default().fg(color))
    };

    let mut stats_lines = vec![
        Line::from(vec![Span::raw("Mood:   "), Span::styled(telemetry.reservoir_state.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("Status: "), Span::styled(telemetry.system_status.clone(), Style::default().fg(gauge_color).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw(format!("Tick:   {:.1} Hz", telemetry.fps))]),
        Line::from(vec![Span::raw(format!("Brain:  {} nrns", telemetry.neuron_active_count))]),
        Line::from(""),
        Line::from(vec![Span::styled("--- METABOLISM ---", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("DOPA: "), make_val_bar(telemetry.dopamine, Color::Cyan), Span::raw(format!(" {:.0}%", telemetry.dopamine * 100.0))]),
        Line::from(vec![Span::raw("CORT: "), make_val_bar(telemetry.cortisol, Color::Magenta), Span::raw(format!(" {:.0}%", telemetry.cortisol * 100.0))]),
        Line::from(vec![Span::raw("ADEN: "), make_val_bar(telemetry.adenosine, Color::DarkGray), Span::raw(format!(" {:.0}%", telemetry.adenosine * 100.0))]),
        Line::from(""),
        Line::from(vec![Span::styled("--- HARDWARE ---", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("CPU:  "), make_val_bar(telemetry.cpu_load / 100.0, if telemetry.cpu_load > 85.0 { Color::Red } else { Color::Green }), Span::raw(format!(" {:.1}%", telemetry.cpu_load))]),
        Line::from(vec![Span::raw("RAM:  "), make_val_bar(telemetry.ram_load, if telemetry.ram_load > 0.9 { Color::Red } else { Color::Green }), Span::raw(format!(" {:.1}%", telemetry.ram_load * 100.0))]),
        Line::from(""),
        Line::from(vec![Span::styled("--- ACTIVITY ---", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw(format!("Novelty: {:.0}%", (telemetry.novelty_score) * 100.0))]),
    ];

    // Neural Grid (10x10)
    for row in 0..10 {
        let mut spans = Vec::new();
        for col in 0..10 {
            let val = telemetry.activity_map.get(row * 10 + col).copied().unwrap_or(0.0);
            let block = if val < 0.25 { "░" } else if val < 0.5 { "▒" } else if val < 0.75 { "▓" } else { "█" };
            let color = if val < 0.33 { Color::DarkGray } else if val < 0.66 { Color::Cyan } else { Color::Yellow };
            spans.push(Span::styled(block, Style::default().fg(color)));
        }
        stats_lines.push(Line::from(spans));
    }

    let metrics_paragraph = Paragraph::new(stats_lines)
        .block(Block::default().title(" Internal State ").borders(Borders::ALL));
    f.render_widget(metrics_paragraph, bottom_chunks[0]);

    // TIMELINE RENDER
    let mut timeline_lines = Vec::new();
    
    // Mix Logs and Thoughts logically or just thoughts then logs?
    // User wants a better log, so let's show thoughts as the main feed and logs as system markers.
    for t in &telemetry.thoughts {
        let (label, color) = match t.voice {
            crate::core::thought::MindVoice::Sensory => ("[EAR]", Color::Cyan),
            crate::core::thought::MindVoice::Cortex => ("[CPU]", Color::Green),
            crate::core::thought::MindVoice::Chem => ("[BIO]", Color::Magenta),
            crate::core::thought::MindVoice::System => ("[SYS]", Color::DarkGray),
        };
        timeline_lines.push(Line::from(vec![
            Span::styled(format!("{:5} ", label), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw(&t.text),
        ]));
    }
    
    // Observer log (optional, only if not already summarized by thoughts)
    for l in &telemetry.logs {
        timeline_lines.push(Line::from(vec![
            Span::styled("[OBS] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(l, Style::default().fg(Color::Gray)),
        ]));
    }

    let border_color = if telemetry.insight_intensity > 0.5 { Color::Yellow } else { Color::White };
    let timeline_paragraph = Paragraph::new(timeline_lines)
        .block(Block::default()
            .title(format!(" 📜 NEURAL TIMELINE (Scroll: {}) ", log_scroll))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)))
        .wrap(Wrap { trim: true })
        .scroll((log_scroll as u16, 0));
        
    f.render_widget(timeline_paragraph, bottom_chunks[1]);
}
