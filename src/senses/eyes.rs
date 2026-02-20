use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use nokhwa::Camera;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::pixel_format::RgbFormat;
use image::{ImageBuffer, Rgb};
use rand::Rng;

pub struct Eyes {
    tx_vision: Sender<Vec<f32>>,
    running: bool,
}

impl Eyes {
    pub fn new(tx_vision: Sender<Vec<f32>>) -> Self {
        Self {
            tx_vision,
            running: true,
        }
    }

    pub fn run(&self) {
        let tx = self.tx_vision.clone();
        
        thread::spawn(move || {
            println!("👁️  VISUAL CORTEX: Initializing Camera...");
            
            // Attempt to open camera 0
            let index = CameraIndex::Index(0);
            let requested = RequestedFormat::new(RequestedFormatType::AbsoluteHighestFrameRate);
            
            match Camera::new(index, requested) {
                Ok(mut camera) => {
                    if let Err(e) = camera.open_stream() {
                        eprintln!("❌ Camera Stream Error: {}. Falling back to simulation.", e);
                        Self::run_simulation(tx);
                        return;
                    }
                    println!("👁️  VISUAL CORTEX: Online (Real Webcam)");
                    
                    let mut last_frame_gray: Option<Vec<u8>> = None;
                    
                    loop {
                        match camera.frame() {
                            Ok(frame_buffer) => {
                                let width = frame_buffer.width();
                                let height = frame_buffer.height();
                                let raw_data = frame_buffer.buffer(); // RGB data
                                
                                // 1. Convert to Grayscale (Luminance)
                                let mut gray = Vec::with_capacity((width * height) as usize);
                                for chunk in raw_data.chunks(3) {
                                    if chunk.len() == 3 {
                                        let luma = (chunk[0] as f32 * 0.299 + chunk[1] as f32 * 0.587 + chunk[2] as f32 * 0.114) as u8;
                                        gray.push(luma);
                                    }
                                }
                                
                                // 2. Calculate Motion (Frame Diff) + Embedding
                                let embedding_size = 64;
                                let mut embedding = vec![0.0; embedding_size];
                                
                                if let Some(last) = &last_frame_gray {
                                    if last.len() == gray.len() {
                                        let chunk_size = gray.len() / embedding_size;
                                        
                                        for i in 0..embedding_size {
                                            let start = i * chunk_size;
                                            let end = (start + chunk_size).min(gray.len());
                                            let mut diff_sum = 0.0;
                                            let mut bright_sum = 0.0;
                                            
                                            for j in start..end {
                                                let diff = (gray[j] as i16 - last[j] as i16).abs() as f32;
                                                diff_sum += diff;
                                                bright_sum += gray[j] as f32;
                                            }
                                            
                                            // Normalize
                                            let count = (end - start) as f32;
                                            let avg_diff = diff_sum / count; // 0..255
                                            let avg_bright = bright_sum / count; // 0..255
                                            
                                            // Embedding = Brightness modulated by Motion
                                            // High motion = High value. Static = Low value.
                                            // Also encode brightness as a baseline.
                                            let combined = (avg_diff / 20.0) + (avg_bright / 255.0 * 0.2); 
                                            embedding[i] = combined.tanh(); // Normalize -1..1 (approx)
                                        }
                                    }
                                }
                                
                                last_frame_gray = Some(gray);
                                
                                // Send
                                if let Err(_) = tx.send(embedding) {
                                    break;
                                }
                            },
                            Err(e) => {
                                eprintln!("⚠️ Camera Frame Error: {}", e);
                                thread::sleep(Duration::from_millis(100));
                            }
                        }
                        
                        // Limit FPS (don't burn CPU)
                        thread::sleep(Duration::from_millis(50)); // ~20 FPS
                    }
                },
                Err(e) => {
                    eprintln!("❌ No Camera Found: {}. Falling back to simulation.", e);
                    Self::run_simulation(tx);
                }
            }
        });
    }
    
    fn run_simulation(tx: Sender<Vec<f32>>) {
         let mut rng = rand::thread_rng();
         println!("👁️  VISUAL CORTEX: Simulation Mode Active");
         loop {
             let sleep_ms = rng.gen_range(200..800);
             thread::sleep(Duration::from_millis(sleep_ms));
             let dim = 64; 
             let mut embedding = Vec::with_capacity(dim);
             let mode = rng.gen_range(0.0..1.0);
             if mode > 0.8 {
                 for _ in 0..dim { embedding.push(rng.gen::<f32>() * 2.0 - 1.0); }
             } else {
                 for _ in 0..dim { embedding.push(rng.gen::<f32>() * 0.2 - 0.1); }
             }
             if let Err(_) = tx.send(embedding) { break; }
         }
    }
}
