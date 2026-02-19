#![allow(deprecated)]

mod core;
mod senses;
mod tui;
mod actuators;
mod cortex;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // CLI ARGUMENT PARSING
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("daemon");

    match mode {
        "daemon" | "start" | "--headless" | "headless" => {
            // THE STAR (Headless Body)
            let listen_path = args.iter().position(|r| r == "--listen")
                .and_then(|i| args.get(i + 1).cloned());

            let headless = args.iter().any(|a| a == "--headless" || a == "headless");

            core::daemon::run(listen_path, headless)?;
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