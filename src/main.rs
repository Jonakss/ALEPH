#![allow(deprecated)]

mod core;
mod senses;
mod tui;
mod actuators;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // CLI ARGUMENT PARSING
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("daemon");

    match mode {
        "daemon" | "start" => {
            // THE STAR (Headless Body)
            core::daemon::run()?;
        },
        "view" | "tui" => {
            // THE TELESCOPE (Visualizer)
            println!("🔭 Connecting to ALEPH Star System...");
            tui::client::run()?;  
        },
        _ => {
            eprintln!("Unknown mode: {}", mode);
            eprintln!("Usage: aleph [start|view]");
        }
    }

    Ok(())
}