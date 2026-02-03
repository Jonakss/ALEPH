use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use crate::core::thought::{Thought, MindVoice};
use rustfft::{FftPlanner, num_complex::Complex};

#[derive(Debug, Clone, Default)]
pub struct AudioSpectrum {
    pub rms: f32,
    pub bass: f32, // 20-250 Hz
    pub mids: f32, // 250-2000 Hz
    pub highs: f32, // 2000-20000 Hz
    pub speaker_id: Option<String>, 
}

pub struct AudioListener {
    _stream: cpal::Stream,
    is_muted: Arc<Mutex<bool>>,
    attention_threshold: Arc<Mutex<f32>>,
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
        let is_muted = Arc::new(Mutex::new(true)); 
        let attention_threshold = Arc::new(Mutex::new(0.005)); 

        // 2. Setup Audio Input
        let host = cpal::default_host();
        let device = host.default_input_device().expect("no input device available");
        let config = device.default_input_config()?;

        // FFT Config
        let fft_len = 1024;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_len);
        let fft_scratch_len = fft.get_inplace_scratch_len();
        let fft_scratch = Arc::new(Mutex::new(vec![Complex::new(0.0, 0.0); fft_scratch_len]));
        let fft_arc = Arc::new(fft); // FFT instance is Sync? Fft is trait object usually. Fft is Arc<dyn Fft>.

        let audio_buffer = Arc::new(Mutex::new(Vec::new()));
        let is_recording = Arc::new(Mutex::new(false));
        let silence_frames = Arc::new(Mutex::new(0));

        let state_clone = state.clone();
        let buffer_clone = audio_buffer.clone();
        let recording_limit = is_recording.clone();
        let silence_counter = silence_frames.clone();
        let muted_clone = is_muted.clone();
        let threshold_clone = attention_threshold.clone();
        let fft_clone = fft_arc.clone();
        let scratch_clone = fft_scratch.clone();
        
        let thought_tx_stream = thought_tx.clone();
        let thought_tx_err = thought_tx.clone();
        
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                // A. Check Metrics (RMS) & FFT
                let rms = (data.iter().map(|s| s * s).sum::<f32>() / data.len() as f32).sqrt();
                
                // FFT Analysis (Zero-Pad to 1024)
                let mut spectrum_buffer: Vec<Complex<f32>> = data.iter()
                    .take(fft_len)
                    .map(|&s| Complex::new(s, 0.0))
                    .collect();
                
                if spectrum_buffer.len() < fft_len {
                    spectrum_buffer.resize(fft_len, Complex::new(0.0, 0.0));
                }

                if let Ok(mut scratch) = scratch_clone.lock() {
                     fft_clone.process_with_scratch(&mut spectrum_buffer, &mut scratch);
                }

                // Analyze Buckets
                // Sample Rate ~44-48k. N=1024. Bin Width ~45 Hz.
                // Bass (20-250): Bins 1..6
                // Mids (250-2000): Bins 6..46
                // Highs (2000+): Bins 46..200
                
                let get_magnitude = |buf: &[Complex<f32>], start: usize, end: usize| -> f32 {
                     if start >= buf.len() || end > buf.len() { return 0.0; }
                     buf[start..end].iter()
                        .map(|c| c.norm())
                        .sum::<f32>() / (end - start).max(1) as f32
                };

                let bass = get_magnitude(&spectrum_buffer, 1, 6) * 5.0; // Boost
                let mids = get_magnitude(&spectrum_buffer, 6, 46) * 2.0;
                let highs = get_magnitude(&spectrum_buffer, 46, 200);

                let spectrum = AudioSpectrum {
                    rms,
                    bass,
                    mids,
                    highs,
                    speaker_id: None,
                };

                let _ = spectrum_tx.send(spectrum);

                // B. Gating Logic
                let threshold = *threshold_clone.lock().unwrap();
                let muted = *muted_clone.lock().unwrap();

                if muted { return; }

                let mut recording = recording_limit.lock().unwrap();
                let mut buffer = buffer_clone.lock().unwrap();
                let mut silence = silence_counter.lock().unwrap();

                if rms > threshold {
                    if !*recording {
                        *recording = true;
                        buffer.clear(); 
                    }
                    *silence = 0;
                } else if *recording {
                    *silence += 1;
                }

                // C. Recording
                if *recording {
                    buffer.extend_from_slice(data);
                    
                    if *silence > 50 { 
                        *recording = false;
                        
                        // D. Process with Whisper
                        let samples = buffer.clone();
                        let ctx_mutex = state_clone.clone();
                        let ears_tx_thread = ears_tx.clone();
                        let thought_tx_thread = thought_tx_stream.clone(); 

                        std::thread::spawn(move || {
                            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                            params.set_language(Some("es"));
                            params.set_print_special(false);
                            params.set_print_progress(false);
                            params.set_print_realtime(false);
                            params.set_print_timestamps(false);
                            
                            let resampled: Vec<f32> = samples.iter().step_by(3).cloned().collect(); 

                            let _print_gag = gag::Gag::stdout().ok();
                            let _err_gag = gag::Gag::stderr().ok();

                            let mut state = ctx_mutex.lock().unwrap();
                            if let Ok(mut state_session) = state.create_state() {
                                if let Ok(_) = state_session.full(params, &resampled[..]) {
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
                                        "[BLANK_AUDIO]", "música", "Subtítulos", "Amara.org", 
                                        "...", "??", "Music", "music"
                                    ];
                                    
                                    let is_hallucination = text.len() < 2 
                                        || triggers.iter().any(|&t| text.contains(t) || text.to_lowercase().contains(&t.to_lowercase()));

                                    if !text.is_empty() && !is_hallucination {
                                        let _ = thought_tx_thread.send(Thought::new(MindVoice::Cortex, format!("Concept recognized: '{}'", text)));
                                        let _ = ears_tx_thread.send(text);
                                    }
                                }
                            }
                        });
                        buffer.clear();
                    }
                }
            },
            move |err| { let _ = thought_tx_err.send(Thought::new(MindVoice::System, format!("Audio Input Error: {}", err))); },
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
