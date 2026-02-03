use anyhow::{Error as E, Result};
use candle_core::{Tensor, Device, DType, IndexOp};
use candle_transformers::generation::LogitsProcessor;
use tokenizers::Tokenizer;
use crate::core::thought::{Thought, MindVoice};
use crate::core::quantized_gemma_raw::ModelWeights as Gemma;
use rand::Rng;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;

const MODEL_FILE: &str = "models/gemma-2b-it.Q4_K_M.gguf"; 
const TOKENIZER_FILE: &str = "models/tokenizer.json"; 

pub struct CortexInput {
    pub text: String,
    pub bio_state: String,
    pub somatic_state: String,
    pub long_term_memory: Option<String>,
    pub _cpu_load: f32,
    pub _ram_pressure: f32,
    pub cognitive_impairment: f32,
    pub entropy: f32,
    pub adenosine: f32,
}

pub struct CortexOutput {
    pub text: String,
    pub inference_latency_ms: u64,
}

pub struct CognitiveCore {
    model: Gemma,
    tokenizer: Tokenizer,
    device: Device,
    logits_processor: LogitsProcessor,
    #[allow(dead_code)]
    thought_tx: Sender<Thought>,
    // FIFO BUFFER (Consciousness Stream)
    history: String, 
    // SPEECH GATING
    is_internal_monologue: bool,
}

impl CognitiveCore {
    pub fn spawn(thought_tx: Sender<Thought>) -> Result<(Sender<CortexInput>, Receiver<CortexOutput>)> {
        let (input_tx, input_rx) = channel::<CortexInput>();
        let (output_tx, output_rx) = channel::<CortexOutput>();
        let thread_thought_tx = thought_tx.clone();

        thread::spawn(move || {
            match Self::new(thread_thought_tx.clone()) {
                Ok(mut core) => {
                    let _ = thread_thought_tx.send(Thought::new(MindVoice::System, "🧠 Cortex (Gemma): ONLINE (Stream Mode)".to_string()));
                    
                    loop {
                        let msg = match input_rx.recv() {
                            Ok(m) => m,
                            Err(_) => break,
                        };

                        // 1. Hyperparameters tied to Biology
                        // Entropy -> Temperature (Chaos)
                        // Adenosine -> Top-P (Focus/Tunnel Vision)
                        let safe_entropy = if msg.entropy.is_nan() { 0.5 } else { msg.entropy };
                        let effective_temp: f64 = (0.5 + safe_entropy * 0.4) as f64; // 0.5 - 0.9
                        
                        let effective_top_p: f64 = (0.95 - (msg.adenosine * 0.5)) as f64; // 0.95 - 0.45

                        core.logits_processor = LogitsProcessor::new(
                            rand::thread_rng().gen(),
                            Some(effective_temp),
                            Some(effective_top_p)
                        );
                         
                        let start = std::time::Instant::now();
                         
                        // 2. FIFO STREAM LOGIC (No prompts, just existence)
                        // Adenosine limits length of thought (fatigue = short thoughts)
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            // Max tokens dependent on fatigue
                             let available_tokens = if msg.adenosine > 0.8 { 
                                 15 
                             } else if msg.adenosine > 0.5 { 
                                 40 
                             } else { 
                                 120 
                             };
                             
                             core.think_stream(&msg.text, &msg.bio_state, available_tokens)
                        }));

                        let response = match result {
                             Ok(text) => text,
                             Err(_) => "...sys_err...".to_string()
                        };
                         
                        let latency_ms = start.elapsed().as_millis() as u64;
                        let _ = output_tx.send(CortexOutput { text: response, inference_latency_ms: latency_ms });
                    }
                }
                Err(e) => {
                    let _ = thread_thought_tx.send(Thought::new(MindVoice::System, format!("FATAL: Cortex Init Failed: {}", e)));
                }
            }
        });

        Ok((input_tx, output_rx))
    }

    fn new(tx: Sender<Thought>) -> Result<Self> {
        // Force CPU to avoid CUDA OOM conflict with Whisper
        let device = Device::Cpu;
        
        let tokenizer = Tokenizer::from_file(TOKENIZER_FILE).map_err(E::msg)?;
        let model = Self::load_model(&device)?;

        Ok(Self {
            model,
            tokenizer,
            device,
            logits_processor: LogitsProcessor::new(rand::thread_rng().gen(), Some(0.7), Some(0.9)),
            thought_tx: tx,
            history: String::new(), // Starts tabula rasa
            is_internal_monologue: false,
        })
    }

    fn load_model(device: &Device) -> Result<Gemma> {
        let mut file = std::fs::File::open(MODEL_FILE).map_err(|e| E::msg(format!("No encuentro {}: {}", MODEL_FILE, e)))?;
        let content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        let model = Gemma::from_gguf(content, &mut file, device)?;
        Ok(model)
    }

    fn think_stream(&mut self, input: &str, bio_state: &str, max_tokens: usize) -> String {
        // RUMINATION DETECTION: If input from inner_voice contains [SELF REFLECTION],
        // the ENTIRE output should be internal (not vocalized).
        if input.contains("[SELF REFLECTION]") || input.contains("[INTERNAL]") || input.contains("[RUMINATION]") {
            self.is_internal_monologue = true;
        } else {
            // External stimulus -> vocal output
            self.is_internal_monologue = false;
        }
        
        // 1. INJECTION
        // We inject the current reality into the stream.
        // Format: [STATE] [INPUT] ->
        let injection = format!("\n[ESTADO: {}] [PERCEPCIÓN: {}] -> PENSAMIENTO:", bio_state, input);
        
        // 2. CONTEXT MANAGEMENT (Rolling Buffer)
        // Keep ~1000 chars of history to maintain "Self" but allow drift.
        let history_len = self.history.len();
        let keep_len = 1000;
        if history_len > keep_len {
            let start = history_len - keep_len;
            // Ensure valid UTF-8 boundary
            let mut char_indices = self.history.char_indices();
            if let Some((idx, _)) = char_indices.find(|(i, _)| *i >= start) {
                 self.history = self.history[idx..].to_string();
            }
        }
        
        // Append new injection
        self.history.push_str(&injection);

        // 3. GENERATION (Completion)
        // The model sees the history + injection and completes the "PENSAMIENTO:"
        // We do NOT send the whole history to `generate` if it's huge, 
        // but 1000 chars is fine for Gemma 2B (context ~8k).
        // Let's send the whole history buffer as the "prompt".
        let prompt = self.history.clone();
        
        let output = match self.generate(&prompt, max_tokens) {
            Ok(s) => s,
            Err(_) => "...".to_string()
        };
        
        // 4. FEEDBACK LOOP
        // The thought becomes part of history
        self.history.push_str(&format!(" {} ", output));
        
        output
    }

    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        // Normalize prompt? No, raw stream.
        
        let tokens = self.tokenizer.encode(prompt, true).map_err(E::msg)?;
        let mut token_ids = tokens.get_ids().to_vec();
        if token_ids.is_empty() { return Ok(String::new()); }

        let mut pos = 0;
        
        let input_tensor = Tensor::new(token_ids.as_slice(), &self.device)?.unsqueeze(0)?;
        eprintln!("DEBUG: Initial forward pass started, len={}", token_ids.len());
        let logits = self.model.forward(&input_tensor, pos)?;
        eprintln!("DEBUG: Initial forward pass complete");
        let mut logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
        
        if logits.rank() == 2 {
            let seq_len = logits.dim(0)?;
            logits = logits.i(seq_len - 1)?;
        }
        pos += token_ids.len();

        let mut gen_tokens = Vec::new();
        let mut next_token = self.logits_processor.sample(&logits)?;
        token_ids.push(next_token);
        gen_tokens.push(next_token);

        let mut current_word = String::new();

        for _ in 0..max_tokens {
            let input_tensor = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input_tensor, pos)?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if logits.rank() == 2 {
                let seq_len = logits.dim(0)?;
                logits.i(seq_len - 1)?
            } else {
                logits
            };

            next_token = self.logits_processor.sample(&logits)?;
            token_ids.push(next_token);
            gen_tokens.push(next_token);
            pos += 1;

            // STREAMING TO VOICE (Flag set by think_stream based on input prefix)
            // Use SENTENCE-LEVEL buffering to prevent choppy audio
            if let Ok(new_fragment) = self.tokenizer.decode(&[next_token], false) {
                 current_word.push_str(&new_fragment);
                 
                 // PHRASE boundary detection - buffer until punctuation or max length
                 let has_punctuation = new_fragment.contains('.') || new_fragment.contains('!') || 
                                       new_fragment.contains('?') || new_fragment.contains(',') ||
                                       new_fragment.contains('\n');
                 let is_too_long = current_word.len() > 40;
                 
                 if has_punctuation || is_too_long {
                     let voice = if self.is_internal_monologue { MindVoice::Cortex } else { MindVoice::Vocal };
                     let _ = self.thought_tx.send(Thought::new(voice, current_word.clone()));
                     current_word.clear();
                 }
            }

            if next_token == self.tokenizer.token_to_id("<end_of_turn>").unwrap_or(1) || 
               next_token == self.tokenizer.token_to_id("<eos>").unwrap_or(1) {
                break;
            }
        }
        
        if !current_word.is_empty() {
             let _ = self.thought_tx.send(Thought::new(MindVoice::Vocal, current_word));
        }
        
        let response = self.tokenizer.decode(&gen_tokens, true).map_err(E::msg)?;
        Ok(response.trim().to_string())
    }
}
