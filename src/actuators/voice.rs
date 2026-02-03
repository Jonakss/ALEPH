use std::process::{Command, Stdio};
use std::thread;
use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;
use crate::core::thought::{Thought, MindVoice};
use std::io::Write;

// Global Serial Queue
static VOICE_QUEUE: OnceLock<Sender<String>> = OnceLock::new();

/// Initialize the voice subsystem (starts background thread)
fn get_queue() -> &'static Sender<String> {
    VOICE_QUEUE.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<String>();
        
        thread::spawn(move || {
            // Serial Consumer Loop
            while let Ok(text) = rx.recv() {
                // Ignore empty or very short bursts (silence)
                if text.trim().len() < 2 { continue; }

                // Determine if we should mute (simple heuristic check if we had logic, here we just play)
                // RUN PIPER
                let mut piper_child = match Command::new("./piper/piper/piper")
                    .args(&["--model", "./piper/es_ES-sharvard-medium.onnx", "--output_raw"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn() 
                {
                    Ok(child) => child,
                    Err(_) => continue, // Log properly if we had channel access, but for now just skip
                };

                if let Some(mut stdin) = piper_child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }

                if let Some(piper_out) = piper_child.stdout.take() {
                    let _ = Command::new("aplay")
                        .args(&["-r", "22050", "-f", "S16_LE", "-t", "raw"])
                        .stdin(piper_out)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
                let _ = piper_child.wait();
            }
        });
        tx
    })
}

/// Neural Voice Actuator via Piper TTS (Queued)
pub fn speak(text: String, tx_thought: Sender<Thought>) {
    let queue = get_queue();
    // Log intent to speak
    let _ = tx_thought.send(Thought::new(MindVoice::System, format!(">> VOCAL QUEUE: '{}'", text)));
    
    // Send to serial thread
    let _ = queue.send(text);
}
