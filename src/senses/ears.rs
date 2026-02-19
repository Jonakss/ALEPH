use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use crate::core::thought::{Thought, MindVoice};
use rustfft::{FftPlanner, num_complex::Complex};

// Symphonia (File Decoding)
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::audio::SampleBuffer;
use std::fs::File;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioSpectrum {
    pub rms: f32,
    pub bass: f32, // 20-250 Hz
    pub mids: f32, // 250-2000 Hz
    pub highs: f32, // 2000-20000 Hz
    #[allow(dead_code)]
    pub speaker_id: Option<String>,
    pub is_voice: bool, 
    // Direct Sensory Projection (64-band spectrogram)
    pub frequency_embedding: Vec<f32>,
}

/// Sensory input mode — determines where audio comes from
#[derive(Debug, Clone)]
pub enum SensoryMode {
    /// Local microphone via CPAL
    Mic,
    /// Audio file via Symphonia decoder
    File(String),
    /// Receive PCM from browser via WebSocket (channel-based)
    WebSocket,
    /// No audio input — text-only perturbation
    Headless,
}

pub struct AudioListener {
    // We hold either a live stream or a thread handle for file playback
    _stream: Option<cpal::Stream>,
    _file_thread: Option<std::thread::JoinHandle<()>>,
    _ws_thread: Option<std::thread::JoinHandle<()>>,
    
    #[allow(dead_code)]
    attention_threshold: Arc<Mutex<f32>>, 
}

impl AudioListener {
    pub fn new(
        thought_tx: Sender<Thought>, 
        ears_tx: Sender<String>, 
        spectrum_tx: Sender<AudioSpectrum>,
        word_embedding_tx: Sender<Vec<f32>>,
        mode: SensoryMode,
        ws_audio_rx: Option<Receiver<Vec<f32>>>,
    ) -> Result<Self, anyhow::Error> {

        // ============================
        // HEADLESS MODE: No audio at all
        // ============================
        if matches!(mode, SensoryMode::Headless) {
            let _ = thought_tx.send(Thought::new(MindVoice::System, "Audio: Headless Mode (No Ears)".to_string()));
            return Ok(Self {
                _stream: None,
                _file_thread: None,
                _ws_thread: None,
                attention_threshold: Arc::new(Mutex::new(0.001)),
            });
        }

        // ============================
        // SHARED SETUP: Whisper + FFT
        // ============================

        // 1. Setup Whisper
        let ctx = {
            let _log_gag = gag::Gag::stderr().ok();
            WhisperContext::new_with_params(
                "models/ggml-base.bin", 
                WhisperContextParameters::default()
            ).expect("failed to load ggml-base.bin")
        };
        
        let state = Arc::new(Mutex::new(ctx));
        let is_muted = Arc::new(Mutex::new(false));
        let whisper_rms_threshold = Arc::new(Mutex::new(0.05));
        let attention_threshold = Arc::new(Mutex::new(0.001));

        // Determine sample_rate based on mode
        let sample_rate: u32 = match &mode {
            SensoryMode::Mic => {
                let host = cpal::default_host();
                let device = host.default_input_device().expect("no input device available");
                let config = device.default_input_config()?;
                config.sample_rate().0
            },
            SensoryMode::WebSocket => 44100, // Browser default
            SensoryMode::File(_) => 44100,   // Will be overridden per-file
            SensoryMode::Headless => unreachable!(),
        };

        let _ = thought_tx.send(Thought::new(MindVoice::System, format!("Audio: Init at {}Hz ({:?})", sample_rate, mode)));

        // 2. FFT Config
        let fft_len = 1024;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_len);
        let fft_arc = Arc::new(fft); 

        let audio_buffer = Arc::new(Mutex::new(Vec::new()));
        let is_recording = Arc::new(Mutex::new(false));
        let silence_frames = Arc::new(Mutex::new(0));
        let peak_rms_during_recording = Arc::new(Mutex::new(0.0f32));

        // WHISPER WORKER THREAD
        let (audio_work_tx, audio_work_rx) = std::sync::mpsc::channel::<Vec<f32>>();
        let worker_state = state.clone();
        let worker_ears_tx = ears_tx.clone();
        let worker_thought_tx = thought_tx.clone();
        let worker_word_embed_tx = word_embedding_tx.clone();

        std::thread::spawn(move || {
             while let Ok(samples) = audio_work_rx.recv() {
                  let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                  params.set_language(Some("es"));
                  params.set_print_special(false);
                  params.set_print_progress(false);
                  params.set_print_realtime(false);
                  params.set_print_timestamps(false);
                  
                  let target_rate = 16000;
                  let ratio = sample_rate as f32 / target_rate as f32;
                  let mut resampled = Vec::new();
                  let mut i = 0.0;
                  
                  while (i as usize) < samples.len() {
                      resampled.push(samples[i as usize]);
                      i += ratio;
                  }

                  let _print_gag = gag::Gag::stdout().ok();
                  let _err_gag = gag::Gag::stderr().ok();

                  let state = worker_state.lock().unwrap();
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
                                "[BLANK_AUDIO]", "Subtítulos", "Amara.org", 
                                "...", "??"
                            ];
                            
                            let is_hallucination = text.len() < 2 
                                || triggers.iter().any(|&t| text.contains(t) || text.to_lowercase().contains(&t.to_lowercase()));

                            if !text.is_empty() && !is_hallucination {
                                // === WORD EMBEDDING PATHWAY ===
                                // Convert transcribed words into a hash-based 64-dim vector
                                // This hits the Semantic region ~50-200ms after sound
                                // (Whisper inference latency = biologically real processing delay)
                                let embedding = text_to_word_embedding(&text, 64);
                                let _ = worker_word_embed_tx.send(embedding);
                                
                                let _ = worker_thought_tx.send(Thought::new(MindVoice::Sensory, format!("🎧 SEMANTIC ECHO: '{}'", text)));
                                let _ = worker_ears_tx.send(text);
                            }
                        }
                  }
             }
        });

        // ============================
        // 3. PROCESSOR CLOSURE
        // Same for ALL modes — takes &[f32], does FFT + RMS + recording
        // ============================
        let processor = {
            let buffer_clone = audio_buffer.clone();
            let recording_limit = is_recording.clone();
            let silence_counter = silence_frames.clone();
            let threshold_clone = attention_threshold.clone();
            let muted_clone = is_muted.clone();
            let whisper_threshold_clone = whisper_rms_threshold.clone();
            let peak_rms_clone = peak_rms_during_recording.clone();
            let fft_clone = fft_arc.clone();
            let thought_tx_debug = thought_tx.clone();
            let audio_work_tx_clone = audio_work_tx.clone();
            let spectrum_tx_clone = spectrum_tx.clone();

            move |data: &[f32]| {
                // A. RMS
                let rms = (data.iter().map(|s| s * s).sum::<f32>() / data.len() as f32).sqrt();
                
                // B. FFT Analysis
                let mut spectrum_buffer: Vec<Complex<f32>> = data.iter()
                    .take(fft_len)
                    .map(|&s| Complex::new(s, 0.0))
                    .collect();
                
                if spectrum_buffer.len() < fft_len {
                    spectrum_buffer.resize(fft_len, Complex::new(0.0, 0.0));
                }

                fft_clone.process(&mut spectrum_buffer);

                let get_magnitude = |buf: &[Complex<f32>], start: usize, end: usize| -> f32 {
                        if start >= buf.len() || end > buf.len() { return 0.0; }
                        buf[start..end].iter()
                        .map(|c| c.norm())
                        .sum::<f32>() / (end - start).max(1) as f32
                };

                let raw_bass = get_magnitude(&spectrum_buffer, 1, 6);
                let raw_mids = get_magnitude(&spectrum_buffer, 6, 46);
                let raw_highs = get_magnitude(&spectrum_buffer, 46, 200);
                
                let scale = fft_len as f32; 
                let gain = 100.0; 
                let (bass, mids, highs) = (
                    (raw_bass / scale * gain).clamp(0.0, 1.0), 
                    (raw_mids / scale * gain).clamp(0.0, 1.0),
                    (raw_highs / scale * gain).clamp(0.0, 1.0)
                );

                // C. Direct Sensory Embedding (64 bands)
                // Map FFT (512 bins) -> 64 bands (Logarithmic scaling would be better, but linear for now)
                let mut embedding = Vec::with_capacity(64);
                let bin_size = spectrum_buffer.len() / 2 / 64; // ~4 bins per band
                
                for i in 0..64 {
                    let start = i * bin_size;
                    let end = start + bin_size;
                    let mag = get_magnitude(&spectrum_buffer, start, end);
                    // Normalize generally
                    embedding.push((mag / scale * gain * 2.0).clamp(0.0, 1.0)); 
                }

                // Voice Detection
                let gate = threshold_clone.try_lock().map(|t| *t).unwrap_or(0.01);
                let is_loud_enough = rms > gate;
                let voice_profile = mids > highs && mids > bass * 0.5;
                let is_voice = is_loud_enough && voice_profile;

                let spectrum = AudioSpectrum { 
                    rms, 
                    bass, 
                    mids, 
                    highs, 
                    speaker_id: None, 
                    is_voice,
                    frequency_embedding: embedding 
                };
                let _ = spectrum_tx_clone.send(spectrum);

                // Gating
                let threshold = threshold_clone.try_lock().map(|t| *t).unwrap_or(0.00001);
                let muted = muted_clone.try_lock().map(|m| *m).unwrap_or(false);
                if muted { return; }

                // Recording Logic
                let Ok(mut recording) = recording_limit.try_lock() else { return; };
                let Ok(mut buffer) = buffer_clone.try_lock() else { return; };
                let Ok(mut silence) = silence_counter.try_lock() else { return; };
                let Ok(mut peak_rms) = peak_rms_clone.try_lock() else { return; };

                if rms > threshold {
                    if !*recording {
                        *recording = true;
                        *peak_rms = 0.0;
                        buffer.clear();
                        let _ = thought_tx_debug.send(Thought::new(MindVoice::System, format!("🎤 LISTEN (RMS: {:.4})", rms)));
                    }
                    if rms > *peak_rms { *peak_rms = rms; }
                    *silence = 0;
                } else if *recording {
                    *silence += 1;
                }

                if *recording {
                    buffer.extend_from_slice(data);
                    
                    if *silence > 45 {
                        *recording = false;
                        let whisper_threshold = whisper_threshold_clone.try_lock().map(|t| *t).unwrap_or(0.3);
                        
                        if *peak_rms > whisper_threshold {
                             let _ = thought_tx_debug.send(Thought::new(MindVoice::System, format!("🧠 GATE OPEN (Peak: {:.4})", *peak_rms)));
                             let samples = buffer.clone();
                             if let Err(_) = audio_work_tx_clone.send(samples) {
                                  eprintln!("🔴 Worker Disconnected");
                             }
                        }
                        buffer.clear();
                    }
                }
            }
        };

        // ============================
        // 4. MODE-SPECIFIC INPUT SOURCE
        // ============================
        match mode {
            SensoryMode::File(path_str) => {
                // --- FILE MODE ---
                let path = path_str.clone();
                let _ = thought_tx.send(Thought::new(MindVoice::System, format!("📂 Opening Audio File: {}", path)));

                let file_thread = std::thread::spawn(move || {
                    let src = File::open(&path).expect("failed to open media");
                    let mss = MediaSourceStream::new(Box::new(src), Default::default());
                    let hint = Hint::new();
                    let meta_opts: MetadataOptions = Default::default();
                    let fmt_opts: FormatOptions = Default::default();

                    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts).expect("unsupported format");
                    let mut format = probed.format;
                    let track = format.tracks().iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL).expect("no audio track");
                    let _time_base = track.codec_params.time_base;
                    
                    let dec_opts: DecoderOptions = Default::default();
                    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &dec_opts).expect("unsupported codec");

                    let track_id = track.id;
                    let file_sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
                    
                    loop {
                        let packet = match format.next_packet() {
                            Ok(p) => p,
                            Err(symphonia::core::errors::Error::IoError(_)) => break,
                            Err(e) => {
                                eprintln!("Error decoding packet: {}", e);
                                break;
                            }
                        };

                        if packet.track_id() != track_id { continue; }

                        match decoder.decode(&packet) {
                            Ok(decoded) => {
                                let spec = *decoded.spec(); 
                                let capacity = decoded.capacity() as u64;

                                let mut sample_buf = SampleBuffer::<f32>::new(capacity, spec);
                                sample_buf.copy_interleaved_ref(decoded);
                                
                                let samples = sample_buf.samples();
                                let channels = spec.channels.count();
                                
                                let mono_samples: Vec<f32> = samples.chunks(channels)
                                    .map(|chunk: &[f32]| chunk.iter().sum::<f32>() / channels as f32)
                                    .collect();

                                for chunk in mono_samples.chunks(1024) {
                                    processor(chunk); 
                                    let sleep_micros = (chunk.len() as f32 / file_sample_rate as f32 * 1_000_000.0) as u64;
                                    std::thread::sleep(std::time::Duration::from_micros(sleep_micros));
                                }
                            },
                            Err(e) => eprintln!("Error decoding: {}", e),
                        }
                    }
                    println!("📂 File Playback Finished.");
                });

                Ok(Self {
                    _stream: None,
                    _file_thread: Some(file_thread),
                    _ws_thread: None,
                    attention_threshold,
                })
            },

            SensoryMode::WebSocket => {
                // --- WEBSOCKET MODE ---
                // Receive PCM f32 samples from browser via channel
                let _ = thought_tx.send(Thought::new(MindVoice::System, "🌐 Audio: WebSocket Mode (Browser Ears)".to_string()));

                let rx = ws_audio_rx.expect("WebSocket mode requires ws_audio_rx channel");
                
                let ws_thread = std::thread::spawn(move || {
                    while let Ok(samples) = rx.recv() {
                        // Feed browser audio into the same processor pipeline
                        for chunk in samples.chunks(1024) {
                            processor(chunk);
                        }
                    }
                    println!("🌐 WebSocket Audio Channel Closed.");
                });

                Ok(Self {
                    _stream: None,
                    _file_thread: None,
                    _ws_thread: Some(ws_thread),
                    attention_threshold,
                })
            },

            SensoryMode::Mic => {
                // --- MIC MODE ---
                let host = cpal::default_host();
                let device = host.default_input_device().expect("no input device available");
                let config = device.default_input_config()?;

                let stream = device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        processor(data);
                    },
                    move |err| { eprintln!("Audio Input Error: {}", err); },
                    None,
                )?;
                
                stream.play()?;
                
                Ok(Self {
                    _stream: Some(stream),
                    _file_thread: None,
                    _ws_thread: None,
                    attention_threshold,
                })
            },

            SensoryMode::Headless => unreachable!(), // Handled above
        }
    }
}

/// Convert text into a hash-based word embedding vector.
/// 
/// Each word is hashed into a consistent position in the vector space.
/// Multiple words accumulate into the same vector, then it's L2-normalized.
/// 
/// This is NOT a real semantic embedding (like Word2Vec/BERT) — it's a 
/// deterministic encoding that gives each word a unique "fingerprint" in
/// the reservoir's input space. What matters mechanically is:
/// 1. Same word → same activation pattern (consistency)
/// 2. Different words → different patterns (discriminability) 
/// 3. The DELAY is real (Whisper inference time = biological processing cost)
fn text_to_word_embedding(text: &str, dim: usize) -> Vec<f32> {
    let mut embedding = vec![0.0f32; dim];
    
    for word in text.split_whitespace() {
        let word_lower = word.to_lowercase();
        // Simple hash: djb2
        let mut hash: u64 = 5381;
        for byte in word_lower.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        
        // Scatter word energy across multiple dimensions
        for k in 0..4 {
            let idx = ((hash.wrapping_add(k * 7919)) as usize) % dim;
            // Sign alternation based on hash bits
            let sign = if (hash >> (k % 64)) & 1 == 0 { 1.0 } else { -1.0 };
            embedding[idx] += sign * 0.25;
        }
    }
    
    // L2 normalize
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-6 {
        for val in embedding.iter_mut() {
            *val /= norm;
        }
    }
    
    embedding
}
