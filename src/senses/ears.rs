use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use crate::core::thought::{Thought, MindVoice};

pub struct AudioListener {
    // We keep the stream alive by holding it here
    _stream: cpal::Stream,
    is_muted: Arc<Mutex<bool>>,
    attention_threshold: Arc<Mutex<f32>>,
}

impl AudioListener {
    pub fn new(thought_tx: Sender<Thought>, ears_tx: Sender<String>) -> Result<Self, anyhow::Error> {
        // 1. Setup Whisper (Load Model)
        let ctx = WhisperContext::new_with_params(
            "ggml-base.bin", 
            WhisperContextParameters::default()
        ).expect("failed to load ggml-base.bin");
        
        // Arc/Mutex for shared state between Audio Thread and Main Thread
        let state = Arc::new(Mutex::new(ctx));
        let is_muted = Arc::new(Mutex::new(true)); // Start Muted (Warmup)
        let attention_threshold = Arc::new(Mutex::new(0.01)); // Sensitivity

        // 2. Setup Audio Input (CPAL)
        let host = cpal::default_host();
        let device = host.default_input_device().expect("no input device available");
        let config = device.default_input_config()?;

        // println!("Default input config: {:?}", config); // BREAKS TUI

        // Buffer for VAD
        let audio_buffer = Arc::new(Mutex::new(Vec::new()));
        let is_recording = Arc::new(Mutex::new(false));
        let silence_frames = Arc::new(Mutex::new(0));

        let state_clone = state.clone();
        let buffer_clone = audio_buffer.clone();
        let recording_limit = is_recording.clone();
        let silence_counter = silence_frames.clone();
        let muted_clone = is_muted.clone();
        let threshold_clone = attention_threshold.clone();
        let thought_tx_err = thought_tx.clone();

        // 3. Audio Thread Closure
        // let err_fn = move |err| eprintln!("an error occurred on stream: {}", err);
        
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                // RUNS ON AUDIO THREAD
                
                // A. Check Metrics
                let rms = (data.iter().map(|s| s * s).sum::<f32>() / data.len() as f32).sqrt();
                let threshold = *threshold_clone.lock().unwrap();
                let muted = *muted_clone.lock().unwrap();

                // B. Gating Logic
                if muted { return; }

                let mut recording = recording_limit.lock().unwrap();
                let mut buffer = buffer_clone.lock().unwrap();
                let mut silence = silence_counter.lock().unwrap();

                if rms > threshold {
                    if !*recording {
                        *recording = true;
                        let _ = thought_tx.send(Thought::new(MindVoice::Sensory, format!("Vibration detected (RMS: {:.3})", rms)));
                        buffer.clear(); // Start fresh
                    }
                    *silence = 0;
                } else if *recording {
                    *silence += 1;
                }

                // C. Recording
                if *recording {
                    buffer.extend_from_slice(data);
                    
                    // Stop if silence prevails (approx 1s of silence at 44k? callback size varies)
                    if *silence > 50 { // ~1-2 seconds of relative silence
                        *recording = false;
                        let _ = thought_tx.send(Thought::new(MindVoice::Sensory, "Silence detected. Processing clip...".to_string()));
                        
                        // D. Process with Whisper (Silenced)
                        let samples = buffer.clone();
                        let ctx_mutex = state_clone.clone();
                        let ears_tx_thread = ears_tx.clone();
                        let thought_tx_thread = thought_tx.clone();

                        std::thread::spawn(move || {
                            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                            params.set_language(Some("es"));
                            params.set_print_special(false);
                            params.set_print_progress(false);
                            params.set_print_realtime(false);
                            params.set_print_timestamps(false);
                            
                            // Naive Resample 48k -> 16k (Every 3rd sample)
                            let resampled: Vec<f32> = samples.iter().step_by(3).cloned().collect(); 

                            let mut state = ctx_mutex.lock().unwrap();
                            let mut state_session = state.create_state().expect("failed to create state");
                            
                            // SILENCE C++ OUTPUT
                            let _print_gag = gag::Gag::stdout().ok();
                            let _err_gag = gag::Gag::stderr().ok();

                            if let Ok(_) = state_session.full(params, &resampled[..]) {
                                drop(_print_gag); // Restore stdout/stderr
                                drop(_err_gag);

                                let num_segments = state_session.full_n_segments().unwrap();
                                let mut text = String::new();
                                for i in 0..num_segments {
                                    if let Ok(segment) = state_session.full_get_segment_text(i) {
                                        text.push_str(&segment);
                                    }
                                }
                                text = text.trim().to_string();
                                
                                // ANTI-HALLUCINATION FILTERS
                                // Whisper often hallucinates these on noise/silence
                                let hallucination_triggers = [
                                    "[BLANK_AUDIO]", "música", "Subtítulos", "Amara.org", 
                                    "...", "??", "Music", "music"
                                ];
                                
                                let is_hallucination = text.len() < 2 
                                    || hallucination_triggers.iter().any(|&t| text.contains(t) || text.to_lowercase().contains(&t.to_lowercase()));

                                if !text.is_empty() && !is_hallucination {
                                    let _ = thought_tx_thread.send(Thought::new(MindVoice::Cortex, format!("Concept recognized: '{}'", text)));
                                    let _ = ears_tx_thread.send(text);
                                } else if is_hallucination {
                                     let _ = thought_tx_thread.send(Thought::new(MindVoice::System, format!("Filtered hallucination: '{}'", text)));
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
