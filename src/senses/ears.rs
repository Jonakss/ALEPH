use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use crate::core::thought::{Thought, MindVoice};
use rustfft::{FftPlanner, num_complex::Complex};
use std::io::Write;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioSpectrum {
    pub rms: f32,
    pub bass: f32, // 20-250 Hz
    pub mids: f32, // 250-2000 Hz
    pub highs: f32, // 2000-20000 Hz
    #[allow(dead_code)]
    pub speaker_id: Option<String>, // Reservado para multi-speaker
}

pub struct AudioListener {
    _stream: cpal::Stream,
    is_muted: Arc<Mutex<bool>>,
    #[allow(dead_code)]
    attention_threshold: Arc<Mutex<f32>>, // Reservado para Predictive Coding
}

impl AudioListener {
    pub fn new(thought_tx: Sender<Thought>, ears_tx: Sender<String>, spectrum_tx: Sender<AudioSpectrum>) -> Result<Self, anyhow::Error> {
        // 1. Setup Whisper
        let ctx = {
            let _log_gag = gag::Gag::stderr().ok();
            WhisperContext::new_with_params(
                "ggml-base.bin", 
                WhisperContextParameters::default()
            ).expect("failed to load ggml-base.bin")
        };
        
        let state = Arc::new(Mutex::new(ctx));
        let is_muted = Arc::new(Mutex::new(false)); // START LISTENING
        let attention_threshold = Arc::new(Mutex::new(0.00001)); // ULTRA SENSITIVE

        // 2. Setup Audio Input
        let host = cpal::default_host();
        let device = host.default_input_device().expect("no input device available");
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let _ = thought_tx.send(Thought::new(MindVoice::System, format!("Audio: Init at {}Hz", sample_rate)));

        // FFT Config
        let fft_len = 1024;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_len);
        let fft_scratch_len = fft.get_inplace_scratch_len();
        let fft_scratch = Arc::new(Mutex::new(vec![Complex::new(0.0, 0.0); fft_scratch_len]));
        let fft_arc = Arc::new(fft); 

        let audio_buffer = Arc::new(Mutex::new(Vec::new()));
        let is_recording = Arc::new(Mutex::new(false));
        let silence_frames = Arc::new(Mutex::new(0));

        // WORKER THREAD SETUP
        let (audio_work_tx, audio_work_rx) = std::sync::mpsc::channel::<Vec<f32>>();
        let worker_state = state.clone();
        let worker_ears_tx = ears_tx.clone();
        let worker_thought_tx = thought_tx.clone();

        std::thread::spawn(move || {
             while let Ok(samples) = audio_work_rx.recv() {
                  let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                  params.set_language(Some("es"));
                  params.set_print_special(false);
                  params.set_print_progress(false);
                  params.set_print_realtime(false);
                  params.set_print_timestamps(false);
                  
                  // Proper Resampling (Target 16000Hz)
                  let target_rate = 16000;
                  let ratio = sample_rate as f32 / target_rate as f32;
                  let mut resampled = Vec::new();
                  let mut i = 0.0;
                  
                  // 1. Resample
                  while (i as usize) < samples.len() {
                      resampled.push(samples[i as usize]);
                      i += ratio;
                  }
                  
                  // 2. NORMALIZE removed (User requested RAW audio)

                  // Gag output
                  let _print_gag = gag::Gag::stdout().ok();
                  let _err_gag = gag::Gag::stderr().ok();

                  let state = worker_state.lock().unwrap();
                  if let Ok(mut state_session) = state.create_state() {
                        if let Ok(_) = state_session.full(params, &resampled[..]) {
                            // Release gags
                            drop(_print_gag); 
                            drop(_err_gag);

                            let num_segments = state_session.full_n_segments().unwrap();
                            let mut text = String::new();
                            for i in 0..num_segments {
                                if let Ok(segment) = state_session.full_get_segment_text(i) {
                                    text.push_str(&segment);
                                }
                            }
                            text = text.trim().to_string();
                            
                            let triggers = [
                                "[BLANK_AUDIO]", "Subtítulos", "Amara.org", 
                                "...", "??"
                            ];
                            
                            let is_hallucination = text.len() < 2 
                                || triggers.iter().any(|&t| text.contains(t) || text.to_lowercase().contains(&t.to_lowercase()));

                            if !text.is_empty() && !is_hallucination {
                                let _ = worker_thought_tx.send(Thought::new(MindVoice::Cortex, format!("Heard: '{}'", text)));
                                let _ = worker_ears_tx.send(text);
                            }
                        }
                  }
             }
        });

        // Callback Clones
        let buffer_clone = audio_buffer.clone();
        let recording_limit = is_recording.clone();
        let silence_counter = silence_frames.clone();
        let muted_clone = is_muted.clone();
        let threshold_clone = attention_threshold.clone();
        let fft_clone = fft_arc.clone();
        let _scratch_clone = fft_scratch.clone();
        let thought_tx_debug = thought_tx.clone(); // For debug logging
        
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                // A. Check Metrics (RMS) & FFT
                let rms = (data.iter().map(|s| s * s).sum::<f32>() / data.len() as f32).sqrt();
                
                // DEBUG: Write RMS to file every ~100 frames
                static mut FRAME_COUNT: u32 = 0;
                unsafe {
                    FRAME_COUNT += 1;
                    if FRAME_COUNT % 100 == 0 {
                        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/audio_debug.log") {
                            let _ = writeln!(f, "RMS: {:.6}", rms);
                        }
                    }
                }
                
                // FFT Analysis - NON-BLOCKING (skip if mutex busy)
                // FFT Analysis
                // We allocate a local scratch buffer to avoid mutex contention with other threads
                // This ensures visualization stays fluid.
                let mut spectrum_buffer: Vec<Complex<f32>> = data.iter()
                    .take(fft_len)
                    .map(|&s| Complex::new(s, 0.0))
                    .collect();
                
                if spectrum_buffer.len() < fft_len {
                    spectrum_buffer.resize(fft_len, Complex::new(0.0, 0.0));
                }

                // Process FFT (cloned FFT instance is thread-safe)
                fft_clone.process(&mut spectrum_buffer);

                let get_magnitude = |buf: &[Complex<f32>], start: usize, end: usize| -> f32 {
                        if start >= buf.len() || end > buf.len() { return 0.0; }
                        buf[start..end].iter()
                        .map(|c| c.norm())
                        .sum::<f32>() / (end - start).max(1) as f32
                };

                // MECHANICAL HONESTY: No normalización
                // No controlamos lo que el oído siente. Lo que llega, llega.
                // El único control real: taparse las orejas (set_mute).
                let raw_bass = get_magnitude(&spectrum_buffer, 1, 6);
                let raw_mids = get_magnitude(&spectrum_buffer, 6, 46);
                let raw_highs = get_magnitude(&spectrum_buffer, 46, 200);
                
                let scale = fft_len as f32; // Basic scaling
                let (bass, mids, highs) = (
                    raw_bass / scale * 10.0, // Boost for visibility
                    raw_mids / scale * 10.0,
                    raw_highs / scale * 10.0
                );

                let spectrum = AudioSpectrum {
                    rms,
                    bass,
                    mids,
                    highs,
                    speaker_id: None,
                };

                let _ = spectrum_tx.send(spectrum);

                // B. Gating Logic - NON-BLOCKING
                let threshold = threshold_clone.try_lock().map(|t| *t).unwrap_or(0.00001); // Ultra low fallback
                let muted = muted_clone.try_lock().map(|m| *m).unwrap_or(false); // Default to listening

                if muted { return; }

                // C. Recording State - NON-BLOCKING (skip frame if busy)
                let Ok(mut recording) = recording_limit.try_lock() else { return; };
                let Ok(mut buffer) = buffer_clone.try_lock() else { return; };
                let Ok(mut silence) = silence_counter.try_lock() else { return; };

                if rms > threshold {
                    if !*recording {
                        *recording = true;
                        buffer.clear();
                        // DEBUG: Log when recording starts
                        let _ = thought_tx_debug.send(Thought::new(MindVoice::System, format!("🎤 RECORDING (RMS: {:.4})", rms)));
                    }
                    *silence = 0;
                } else if *recording {
                    *silence += 1;
                }

                // D. Recording
                if *recording {
                    buffer.extend_from_slice(data);
                    
                    if *silence > 45 { // ~0.75s silence (more generous)
                        *recording = false;
                        
                        // DEBUG: Log when sending to Whisper
                        let samples_len = buffer.len();
                        let _ = thought_tx_debug.send(Thought::new(MindVoice::System, format!("🎧 SENT TO WHISPER ({} samples)", samples_len)));
                        
                        // E. Send to Worker
                        let samples = buffer.clone();
                        // Detect Worker Death
                        if let Err(_) = audio_work_tx.send(samples) {
                             eprintln!("🔴 CRITICAL: Audio Worker Thread Disconnected!");
                        }
                        buffer.clear();
                    }
                }
            },
            move |err| { eprintln!("Audio Input Error: {}", err); },
            None,
        )?;
        
        stream.play()?;
        
        Ok(Self {
            _stream: stream,
            is_muted,
            attention_threshold,
        })
    }

    pub fn set_mute(&self, mute: bool) {
        let mut m = self.is_muted.lock().unwrap();
        *m = mute;
    }
}
