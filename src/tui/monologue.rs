use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    text::{Line, Span},
};
use crate::core::thought::{Thought, MindVoice};

#[allow(dead_code)]
pub fn render_monologue<'a>(thoughts: &'a [Thought], insight_intensity: f32) -> List<'a> {
    let items: Vec<ListItem> = thoughts
        .iter()
        .rev()
        .take(12) 
        .rev()    
        .map(|t| {
            let (prefix_text, color) = match t.voice {
                MindVoice::Sensory => ("[SENSORY]", Color::Cyan),
                MindVoice::Cortex => ("[CORTEX] ", Color::Green),
                MindVoice::Chem => ("[CHEM]   ", Color::Magenta),
                MindVoice::System => ("[SYSTEM] ", Color::DarkGray),
            };

            let line = Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::DarkGray)),
                Span::styled(prefix_text, Style::default().fg(color)),
                Span::raw(" "),
                Span::raw(&t.text), // Ensure this field exists
            ]);
            
            ListItem::new(line)
        })
        .collect();
    
    // Dynamic Border for Insight
    let border_style = if insight_intensity > 0.05 {
        Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
    } else {
        Style::default()
    };

    List::new(items)
        .block(Block::default()
            .title(" STREAM OF CONSCIOUSNESS ")
            .borders(Borders::ALL)
            .border_style(border_style))
}
