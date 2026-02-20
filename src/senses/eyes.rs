use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use nokhwa::Camera;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::pixel_format::RgbFormat;
// use image::{ImageBuffer, Rgb};
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
            let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
            
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
                                let width = frame_buffer.resolution().width();
                                let height = frame_buffer.resolution().height();
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
                                // We want a 64x64 Grid (4096 points) for both Embedding and Visualization
                                let grid_w = 64;
                                let grid_size = grid_w * grid_w;
                                let mut visual_grid = vec![0.0; grid_size];
                                
                                if let Some(last) = &last_frame_gray {
                                    if last.len() == gray.len() {
                                        // Map 320x240 -> 64x64
                                        let step_x = width as usize / grid_w;
                                        let step_y = height as usize / grid_w;
                                        
                                        for y in 0..grid_w {
                                            for x in 0..grid_w {
                                                // Sample center pixel of the block (fastest)
                                                // Averaging would be better but slower
                                                let src_x = x * step_x;
                                                let src_y = y * step_y;
                                                let idx = src_y * width as usize + src_x;
                                                
                                                if idx < gray.len() {
                                                    let val = gray[idx];
                                                    let last_val = last[idx];
                                                    
                                                    let diff = (val as i16 - last_val as i16).abs() as f32;
                                                    let bright = val as f32;
                                                    
                                                    // Combined signal: Motion (High) + Brightness (Low baseline)
                                                    let signal = (diff / 30.0) + (bright / 255.0 * 0.1);
                                                    visual_grid[y * grid_w + x] = signal.min(1.0);
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                last_frame_gray = Some(gray);
                                
                                // Send Grid (Daemon will downsample for embedding if needed, or we use grid as embedding)
                                // Current Reservoir expects 64-float embedding?
                                // Let's send the FULL 4096 grid. The Reservoir can sample it or we project it.
                                // Wait, the channel is Sender<Vec<f32>>.
                                // We should send the full grid.
                                if let Err(_) = tx.send(visual_grid) {
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
