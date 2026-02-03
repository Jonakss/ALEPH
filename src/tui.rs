use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, Paragraph, Wrap, Sparkline},
    Frame, Terminal,
};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

use crate::core::thought::Thought;

// Estructura de Telemetría que viene del Backend
pub struct Telemetry {
    pub audio_rms: f32,       // 0.0 - 1.0 (volumen)
    pub audio_peak: f32,      // Pico reciente
    pub entropy: f32,         // 0.0 - 1.0
    pub neuron_active_count: usize, // Cuantas neuronas dispararon
    pub system_status: String,// "FLOW", "PANIC", etc.
    pub last_entropy_delta: f32, // Cambio de entropía
    pub fps: f64,             // Backend ticks per second
    pub cpu_load: f32,        // Proprioception
    pub ram_load: f32,        // Proprioception
    pub log_message: Option<String>, // Mensajes del Observador
    pub adenosine: f32, // Sleep Pressure
    pub dopamine: f32,  // Reward
    pub cortisol: f32,  // Stress
    pub thoughts: Vec<Thought>, // Stream of Consciousness
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            audio_rms: 0.0,
            audio_peak: 0.0,
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
            thoughts: Vec::new(),
        }
    }
}

pub fn run_tui(rx_telemetry: Receiver<Telemetry>) -> Result<(), anyhow::Error> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Loop UI
    let res = run_app(&mut terminal, rx_telemetry);

    // 3. Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(_err) = res {
        // println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    rx: Receiver<Telemetry>,
) -> io::Result<()> 
where
    B::Error: Into<io::Error>, // Fix generic error conversion
{
    // Historial para gráficos
    let mut audio_history: Vec<u64> = vec![0; 100];
    let mut entropy_history: Vec<(f64, f64)> = Vec::new(); // (time, value)
    let start_time = Instant::now();
    let window_width = 10.0; // segundos de historia

    let mut last_telemetry = Telemetry::default();

    loop {
        // 1. Consumir Telemetría (Non-blocking)
        // Drenamos el canal para tener el dato más fresco
        let mut new_data = false;
        while let Ok(data) = rx.try_recv() {
            last_telemetry = data;
            new_data = true;
        }

        if new_data {
            // Actualizar históricos
            let rms_scaled = (last_telemetry.audio_rms * 100.0) as u64;
            audio_history.push(rms_scaled);
            if audio_history.len() > 100 {
                audio_history.remove(0);
            }

            let now = start_time.elapsed().as_secs_f64();
            entropy_history.push((now, last_telemetry.entropy as f64));
            // Limpiar datos viejos
            entropy_history.retain(|&(t, _)| t > now - window_width);
        }

        // 2. Dibujar
        terminal.draw(|f| {
            ui(f, &last_telemetry, &audio_history, &entropy_history, start_time.elapsed().as_secs_f64(), window_width);
        }).map_err(Into::into)?; // Convert generic error to io::Error

        // 3. Input Handling (Salir con 'q')
        if event::poll(Duration::from_millis(16))? { // ~60fps UI poll
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
    }
}

mod avatar;
mod monologue;

// ...

pub fn ui(
    f: &mut Frame,
    telemetry: &Telemetry,
    audio_history: &[u64],
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


    // --- PANEL IZQ: AUDIO INPUT ---
    let audio_block = Block::default()
        .title(" 👂 AUDIO ")
        .borders(Borders::ALL);
    
    let sparkline = Sparkline::default()
        .block(audio_block)
        .data(audio_history)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(sparkline, top_chunks[0]);

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
    let avatar_widget = Paragraph::new(face.ascii)
        .block(Block::default().title(" ALEPH ").borders(Borders::ALL))
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

    let text = vec![
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
            Span::raw("CPU Load: "),
            make_bar(telemetry.cpu_load / 100.0, if telemetry.cpu_load > 80.0 { Color::Red } else { Color::Green }),
            Span::raw(format!(" {:.1}%", telemetry.cpu_load)),
        ]),
        Line::from(vec![
            Span::raw("RAM Load: "),
            make_bar(telemetry.ram_load, if telemetry.ram_load > 0.9 { Color::Red } else { Color::Green }),
            Span::raw(format!(" {:.1}%", telemetry.ram_load * 100.0)),
        ]),
    ];
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
    let monologue_widget = monologue::render_monologue(&telemetry.thoughts);
    f.render_widget(monologue_widget, chunks[2]);

    // --- PANEL INFERIOR 2: MONOLOGUE ---
    // Wait, chunks only has 3 items. I need to resize chunks or use a new area.
    // Let's repurpose chunks[2] for logs AND monologue? Or split chunks[2].
}
