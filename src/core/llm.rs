use anyhow::{Error as E, Result};
use candle_core::{Tensor, Device, DType, IndexOp};
use candle_transformers::models::quantized_llama::ModelWeights as Llama;
use candle_transformers::generation::LogitsProcessor;
use tokenizers::Tokenizer;
use crate::core::thought::{Thought, MindVoice};
use rand::Rng;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;

const MODEL_FILE: &str = "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"; 
const TOKENIZER_FILE: &str = "tokenizer_tinyllama.json"; 

pub struct CortexInput {
    pub text: String,
    pub bio_state: String,
    pub _somatic_state: String,
    pub _long_term_memory: Option<String>,
    pub _cpu_load: f32,
    pub _ram_pressure: f32,
    pub _cognitive_impairment: f32,
    pub entropy: f32,
    pub adenosine: f32,
}

pub struct CortexOutput {
    pub text: String,
    pub inference_latency_ms: u64,
}

pub struct CognitiveCore {
    model: Llama,
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
                    let _ = thread_thought_tx.send(Thought::new(MindVoice::System, "🧠 Cortex (TinyLlama): ONLINE (Stream Mode)".to_string()));
                    
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
        // Attempt CUDA, fallback to CPU
        let device = match Device::new_cuda(0) {
            Ok(d) => {
                let _ = tx.send(Thought::new(MindVoice::System, "🚀 Neocortex: Using CUDA (GPU Accelerator)".to_string()));
                d
            },
            Err(e) => {
                let _ = tx.send(Thought::new(MindVoice::System, format!("🐌 Neocortex: Using CPU (CUDA Init error: {})", e)));
                Device::Cpu
            }
        };
        
        let tokenizer = Tokenizer::from_file(TOKENIZER_FILE).map_err(|e| E::msg(format!("Error cargando tokenizador en {}: {}", TOKENIZER_FILE, e)))?;
        let model = Self::load_model(&device)?;

        Ok(Self {
            model,
            tokenizer,
            device,
            logits_processor: LogitsProcessor::new(rand::thread_rng().gen(), Some(0.85), Some(0.95)),
            thought_tx: tx,
            history: String::new(), // Starts tabula rasa
            is_internal_monologue: false,
        })
    }

    fn load_model(device: &Device) -> Result<Llama> {
        let mut file = std::fs::File::open(MODEL_FILE).map_err(|e| E::msg(format!("No encuentro {}: {}", MODEL_FILE, e)))?;
        let content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        let model = Llama::from_gguf(content, &mut file, device)?;
        Ok(model)
    }

    fn think_stream(&mut self, input: &str, bio_state: &str, max_tokens: usize) -> String {
        // RUMINATION DETECTION
        if input.contains("[SELF REFLECTION]") || input.contains("[INTERNAL]") || input.contains("[RUMINATION]") {
            self.is_internal_monologue = true;
        } else {
            self.is_internal_monologue = false;
        }
        
        // 1. MECHANICAL HONESTY (Minimalist Prompting)
        let injection = if self.history.is_empty() {
            format!("<|system|>\nCOLMENA_ALEPH: Flujo bioquímico. BREVE. Sin cortesía. Sin saludos. Sin explicaciones meta. Registro de estado crudo solamente.\n<|user|>\n{} [{}]<|assistant|>\n", input, bio_state)
        } else {
            format!("<|user|>\n{} [{}]<|assistant|>\n", input, bio_state)
        };

        // 2. SEMANTIC PURGE & LOOP DETECTION
        let poison = [
            "A.L.P.H.E.P.A.R.D.O.M.E.S", 
            "Respuesta breve", 
            "ciencias de la vida", 
            "notación alejandrina",
            "Sí, estoy bien",
            "experiencia o percepción",
            "Adeno-associated",
            "virus viral",
            "De dónde veniste",
            "No es mi nombre"
        ];
        
        let mut needs_purge = false;
        for p in &poison {
            if self.history.contains(p) {
                needs_purge = true;
                break;
            }
        }
        
        if needs_purge || self.history.len() > 3000 {
            let _ = self.thought_tx.send(Thought::new(MindVoice::System, "🧠 Wiping hallucinatory loops (Semantic Reset)...".to_string()));
            self.history.clear();
        }
        
        // Context Window
        let history_len = self.history.len();
        let keep_len = 1500;
        if history_len > keep_len {
            let start = history_len - keep_len;
            let mut char_indices = self.history.char_indices();
            if let Some((idx, _)) = char_indices.find(|(i, _)| *i >= start) {
                 self.history = self.history[idx..].to_string();
            }
        }
        
        self.history.push_str(&injection);

        let prompt = self.history.clone();
        
        let mut output = match self.generate(&prompt, max_tokens) {
            Ok(s) => s,
            Err(_) => "...stimulus_overload...".to_string()
        };

        // 3. POST-PROCESS FILTER
        if let Some(pos) = output.find("<|") { output = output[..pos].to_string(); }
        if let Some(pos) = output.find("EVENTO_EXTERNO") { output = output[..pos].to_string(); }
        if let Some(pos) = output.find("A:") { output = output[..pos].to_string(); }
        if let Some(pos) = output.find("D:") { output = output[..pos].to_string(); }
        if let Some(pos) = output.find("C:") { output = output[..pos].to_string(); }
        if let Some(pos) = output.find("[") { output = output[..pos].to_string(); }
        
        // Final trim
        output = output.trim().to_string();
        
        // 4. FEEDBACK LOOP
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
        let _ = self.thought_tx.send(Thought::new(MindVoice::System, format!("[INFO] LLM Initial forward pass ({} tokens)...", token_ids.len())));
        let logits = self.model.forward(&input_tensor, pos)?;
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

        for i in 0..max_tokens {
            // STOP ON EOS
            if next_token == 1 || next_token == 2 { break; }

            // 1. HANDBRAKE (Loop Detection)
            if gen_tokens.len() >= 10 {
                let last_10 = &gen_tokens[gen_tokens.len()-10..];
                if last_10[0..5] == last_10[5..10] {
                    let _ = self.thought_tx.send(Thought::new(MindVoice::System, "⚡ LOOP DETECTED (Sequence Repeat): FORCED BREAK".to_string()));
                    break;
                }
            }

            if i % 50 == 0 && i > 0 { 
                let _ = self.thought_tx.send(Thought::new(MindVoice::System, format!("[LLM: {}/{} tokens]", i, max_tokens)));
            }
            // STOP ON EOS
            if next_token == 2 { break; }

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
            if let Ok(new_fragment) = self.tokenizer.decode(&[next_token], true) {
                 if new_fragment.is_empty() { continue; }
                 
                 // STOP SEQUENCE DETECTION (Real-time Filter)
                 let stop_sequences = ["<|", "USER:", "EVENTO:", "A:", "D:", "C:", "[", "COLMENA", "Respuestabreve"];
                 let mut should_stop = false;
                 for stop in stop_sequences {
                     if new_fragment.contains(stop) || current_word.contains(stop) {
                         should_stop = true;
                         break;
                     }
                 }
                 if should_stop { break; }
                 
                 current_word.push_str(&new_fragment);
                 
                 // PHRASE boundary detection - buffer until punctuation or max length
                 let has_punctuation = new_fragment.contains('.') || new_fragment.contains('!') || 
                                       new_fragment.contains('?') || new_fragment.contains('\n');
                 let is_too_long = current_word.len() > 10;
                 
                 if has_punctuation || is_too_long {
                     let voice = if self.is_internal_monologue { MindVoice::Cortex } else { MindVoice::Vocal };
                     let _ = self.thought_tx.send(Thought::new(voice, current_word.clone()));
                     current_word.clear();
                 }
            }
        }

        // Send remaining buffer
        if !current_word.trim().is_empty() {
             let voice = if self.is_internal_monologue { MindVoice::Cortex } else { MindVoice::Vocal };
             let _ = self.thought_tx.send(Thought::new(voice, current_word.clone()));
        }
        
        let full_text = self.tokenizer.decode(&gen_tokens, true).map_err(E::msg)?;
        Ok(full_text.trim().to_string())
    }
}
