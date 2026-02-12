use std::process::{Command, Stdio};
use std::thread;
use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;
use crate::core::thought::Thought;
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
pub fn speak(text: String, _tx_thought: Sender<Thought>) {
    let queue = get_queue();
    // Log intent to speak
    // println!(">> VOCAL QUEUE: '{}'", text);
    // let _ = tx_thought.send(Thought::new(MindVoice::System, format!(">> VOCAL QUEUE: '{}'", text)));
    
    // Send to serial thread
    let _ = queue.send(text);
}

/// Generates a glitch sound (white noise) of a given intensity
pub fn glitch(intensity: f32) {
    thread::spawn(move || {
        // Duration in seconds (0.1s to 0.5s based on intensity)
        let duration = (0.1 + (intensity * 0.4)).min(1.0);
        let sample_rate = 44100;
        let num_samples = (sample_rate as f32 * duration) as usize;
        
        let mut noise_data = Vec::with_capacity(num_samples * 2);
        let mut rng = rand::thread_rng();
        use rand::Rng;

        for _ in 0..num_samples {
            // White noise: Random value between -32767 and 32767
            let sample = (rng.gen::<f32>() * 2.0 - 1.0) * 32767.0 * intensity.min(1.0);
            let sample_i16 = sample as i16;
            noise_data.extend_from_slice(&sample_i16.to_le_bytes());
        }

        let child = Command::new("aplay")
            .args(&["-r", "44100", "-f", "S16_LE", "-t", "raw", "-c", "1"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok();

        if let Some(mut child) = child {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(&noise_data);
            }
            let _ = child.wait();
        }
    });
}
