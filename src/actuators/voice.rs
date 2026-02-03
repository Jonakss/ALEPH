use std::process::{Command, Stdio};
use std::thread;
use std::sync::mpsc::Sender;
use crate::core::thought::{Thought, MindVoice};

/// Neural Voice Actuator via Piper TTS
pub fn speak(text: String, tx: Sender<Thought>) {
    thread::spawn(move || {
        let _ = tx.send(Thought::new(MindVoice::System, format!(">> VOCAL: Speaking: '{}'", text)));

        use std::io::Write;
        
        // 1. Start Piper process
        let mut piper_child = match Command::new("./piper/piper/piper")
            .args(&["--model", "./piper/es_ES-sharvard-medium.onnx", "--output_raw"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Silence logs
            .spawn() 
        {
            Ok(child) => child,
            Err(e) => {
                let _ = tx.send(Thought::new(MindVoice::System, format!("Voice Error (Piper spawn): {}", e)));
                return;
            }
        };

        // 2. Write text to Piper stdin
        if let Some(mut stdin) = piper_child.stdin.take() {
            if let Err(e) = stdin.write_all(text.as_bytes()) {
                 let _ = tx.send(Thought::new(MindVoice::System, format!("Voice Error (write/piper): {}", e)));
                 return;
            }
        } // stdin closes here, signaling EOF to Piper

        // 3. Pipe Piper stdout -> Aplay stdin
        if let Some(piper_out) = piper_child.stdout.take() {
            let _ = Command::new("aplay")
                .args(&["-r", "22050", "-f", "S16_LE", "-t", "raw"])
                .stdin(piper_out)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status(); // Use status() to wait for aplay
        }

        // Wait for piper to finish
        let _ = piper_child.wait();
    });
}
