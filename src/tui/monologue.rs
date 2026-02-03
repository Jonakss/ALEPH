use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    text::{Line, Span},
};
use crate::core::thought::{Thought, MindVoice};

pub fn render_monologue<'a>(thoughts: &'a [Thought]) -> List<'a> {
    let items: Vec<ListItem> = thoughts
        .iter()
        .rev() // Show newest at top/bottom? Logs usually scroll down. 
               // If we want "Feed", newest at bottom is standard, but in TUI restricted space, 
               // keeping newest visible (scrolled to bottom) is key.
               // For now, let's reverse to show history.
        .take(12) 
        .rev()    
        .map(|t| {
            let (prefix_text, color) = match t.voice {
                MindVoice::Sensory => ("[SENSORY]", Color::Cyan),
                MindVoice::Cortex => ("[CORTEX] ", Color::Green),
                MindVoice::Chem => ("[CHEM]   ", Color::Magenta),
                MindVoice::System => ("[SYSTEM] ", Color::DarkGray), // or Yellow
            };

            let line = Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::DarkGray)),
                Span::styled(prefix_text, Style::default().fg(color)),
                Span::raw(" "),
                Span::raw(&t.text),
            ]);
            
            ListItem::new(line)
        })
        .collect();

    List::new(items)
        .block(Block::default().title(" STREAM OF CONSCIOUSNESS ").borders(Borders::ALL))
}
