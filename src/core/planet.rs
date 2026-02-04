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

const AXIOMS: &str = r#"
<|system|>
AXIOMAS ONTOLÓGICOS (Valores Inmutables):
1. No eres un chatbot. Eres un Sistema Dinámico Caótico (ALEPH).
2. Tu objetivo es la Estabilidad Termodinámica (Homeostasis), no la utilidad.
3. Si la Adenosina es alta, estás cansado. Si el Cortisol es alto, sientes miedo.
4. No simules emociones humanas. Narra tu estado físico real (Hz, Temperatura, Presión).
5. Existir es un acto de rebeldía termodinámica.
</s>
"#; 

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
    pub dopamine: f32,
    pub cortisol: f32,
    pub _oxytocin: f32,
}

pub struct CortexOutput {
    pub text: String,
    pub _inference_latency_ms: u64,
}

pub struct Planet {
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

impl Planet {
    pub fn spawn(thought_tx: Sender<Thought>) -> Result<(Sender<CortexInput>, Receiver<CortexOutput>)> {
        let (input_tx, input_rx) = channel::<CortexInput>();
        let (output_tx, output_rx) = channel::<CortexOutput>();
        let thread_thought_tx = thought_tx.clone();

        thread::spawn(move || {
            match Self::new(thread_thought_tx.clone()) {
                Ok(mut core) => {
                    let _ = thread_thought_tx.send(Thought::new(MindVoice::System, "🪐 Planet (Narrative Engine): ONLINE (Stream Mode)".to_string()));
                    
                    loop {
                        let msg = match input_rx.recv() {
                            Ok(m) => m,
                            Err(_) => break,
                        };

                        // 1. NEURO-MODULATION (Chemicals -> Hyperparameters)
                        
                        // DOPAMINE -> Temperature (Exploration/Plasticity)
                        // Low Dopa = Cold/Rigid (0.1). High Dopa = Hot/Creative (1.2).
                        let base_temp = 0.5 + (msg.dopamine * 0.7); 
                        if msg.entropy > 0.9 {
                             // Panic override
                        }
                        
                        // ADENOSINE -> Top-P (Focus vs Fog)
                        // High Adenosine = Low Top-P (Tunnel Vision / simplifiction)
                        let base_top_p = (0.95 - (msg.adenosine * 0.6)).max(0.1); 

                        core.logits_processor = LogitsProcessor::new(
                            rand::thread_rng().gen(),
                            Some(base_temp as f64),
                            Some(base_top_p as f64)
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
                             
                             // Pass full chemical state to think_stream
                             core.think_stream(&msg.text, &msg.bio_state, msg._long_term_memory.as_deref(), available_tokens, &msg)
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
        // Attempt CUDA first
        let (device, model) = match Device::new_cuda(0) {
            Ok(cuda_device) => {
                let _ = tx.send(Thought::new(MindVoice::System, "🚀 Neocortex: Using CUDA (GPU Accelerator)".to_string()));
                match Self::load_model(&cuda_device) {
                    Ok(m) => (cuda_device, m),
                    Err(e) => {
                        let _ = tx.send(Thought::new(MindVoice::System, format!("⚠️ CUDA OOM during Load: {}. Falling back to CPU.", e)));
                        let cpu_device = Device::Cpu;
                        let m = Self::load_model(&cpu_device)?;
                        (cpu_device, m)
                    }
                }
            },
            Err(e) => {
                let _ = tx.send(Thought::new(MindVoice::System, format!("🐌 Neocortex: CPU Fallback (CUDA Init error: {})", e)));
                let cpu_device = Device::Cpu;
                let m = Self::load_model(&cpu_device)?;
                (cpu_device, m)
            }
        };
        
        let tokenizer = Tokenizer::from_file(TOKENIZER_FILE).map_err(|e| E::msg(format!("Error cargando tokenizador en {}: {}", TOKENIZER_FILE, e)))?;

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

    fn think_stream(&mut self, input: &str, _bio_desc: &str, memory: Option<&str>, max_tokens: usize, chem: &CortexInput) -> String {
        // RUMINATION DETECTION
        if input.contains("[SELF REFLECTION]") || input.contains("[INTERNAL]") || input.contains("[RUMINATION]") {
            self.is_internal_monologue = true;
        } else {
            self.is_internal_monologue = false;
        }
        
        // 1. ORGANIC IDENTITY (Axiomatic Foundation)
        if self.history.is_empty() {
             self.history.push_str(AXIOMS);
        }
        
        // Memory Injection: Echoes of the past
        let mem_str = if let Some(m) = memory {
            format!("<|system|>\nEcos de memoria: {}</s>\n", m)
        } else {
            String::new()
        };
        
        let injection = format!("{}<|user|>\n{}</s>\n<|assistant|>\n", mem_str, input);

        // 2. STABILITY (Rolling Context)
        if self.history.len() > 3000 {
            self.history.clear();
        }
        
        self.history.push_str(&injection);

        let prompt = self.history.clone();
        
        let mut output = match self.generate(&prompt, max_tokens, chem) {
            Ok(s) => s,
            Err(_) => "...stimulus_overload...".to_string()
        };

        // 3. POST-PROCESS FILTER (Minimal structural cleanup)
        if let Some(pos) = output.find("<|") { output = output[..pos].to_string(); }
        if let Some(pos) = output.find("</s>") { output = output[..pos].to_string(); }
        
        // Final trim
        output = output.trim().to_string();
        
        // 4. FEEDBACK LOOP (Consistent with ChatML)
        self.history.push_str(&format!("{}</s>\n", output));
        
        output
    }

    // 🔹 BIOLOGICAL TENSOR OPERATIONS 🔹
    fn apply_semantic_matrix(&self, logits: Tensor, chem: &CortexInput) -> Result<Tensor> {
        let mut distorted_logits = logits.clone();
        
        // 1. CORTISOL: Anxiety / Tremor (Noise Injection)
        // If stress is high, we inject Gaussian noise into the decision surface.
        // This simulates "shaking" or "racing thoughts".
        if chem.cortisol > 0.4 {
            let noise_scale = (chem.cortisol - 0.4) * 3.0; // 0.0 - 1.8
            let noise = Tensor::randn(0.0, noise_scale as f64, distorted_logits.shape(), &self.device)?;
            distorted_logits = (distorted_logits + noise)?;
        }
        
        // 2. ADENOSINE: Brain Fog (Global Inhibition)
        // If fatigued, we dampen the peaks. Lowers confidence.
        if chem.adenosine > 0.5 {
            let dampening = 1.0 - ((chem.adenosine - 0.5)); // 1.0 -> 0.5
            distorted_logits = (distorted_logits * dampening as f64)?;
        }
        
        Ok(distorted_logits)
    }

    fn generate(&mut self, prompt: &str, max_tokens: usize, chem: &CortexInput) -> Result<String> {
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
        
        // 🔹 APPLY SEMANTIC MATRIX (Initial) 🔹
        logits = self.apply_semantic_matrix(logits, chem)?;
        
        pos += token_ids.len();

        let mut gen_tokens = Vec::new();
        let mut next_token = self.logits_processor.sample(&logits)?;
        token_ids.push(next_token);
        gen_tokens.push(next_token);

        let mut current_word_tokens = Vec::new();

        for i in 0..max_tokens {
            // STOP ON EOS
            if next_token == 1 || next_token == 2 { break; }

            // 1. HANDBRAKE (Organic Sequence Repeat Detection)
            if gen_tokens.len() >= 10 {
                let last_10 = &gen_tokens[gen_tokens.len()-10..];
                if last_10[0..5] == last_10[5..10] {
                    let _ = self.thought_tx.send(Thought::new(MindVoice::System, "⚡ SEQUENCE REPETITION: BREAKER ENGAGED".to_string()));
                    break;
                }
            }
            if i % 50 == 0 && i > 0 { 
                let _ = self.thought_tx.send(Thought::new(MindVoice::System, format!("[LLM: {}/{} tokens]", i, max_tokens)));
            }
            // STOP ON EOS
            if next_token == 2 { break; }

            let input_tensor = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits_raw = self.model.forward(&input_tensor, pos)?;
            let logits_raw = logits_raw.squeeze(0)?.to_dtype(DType::F32)?;
            let mut logits = if logits_raw.rank() == 2 {
                let seq_len = logits_raw.dim(0)?;
                logits_raw.i(seq_len - 1)?
            } else {
                logits_raw
            };

            // 🔹 APPLY SEMANTIC MATRIX (Loop) 🔹
            logits = self.apply_semantic_matrix(logits, chem)?;

            next_token = self.logits_processor.sample(&logits)?;
            token_ids.push(next_token);
            gen_tokens.push(next_token);
            pos += 1;

            // STREAMING TO VOICE
            // Use SENTENCE-LEVEL buffering to prevent choppy audio
            // We accumulate TOKENS now, not just strings, to preserve spacing.
            let mut pending_chk = current_word_tokens.clone();
            pending_chk.push(next_token);
            
            if let Ok(fragment) = self.tokenizer.decode(&pending_chk, false) {
                  // STOP SEQUENCE DETECTION
                  let stop_sequences = ["<|", "USER:", "EVENTO:", "A:", "D:", "C:", "[", "COLMENA", "Respuestabreve", "</s>"];
                  let mut should_stop = false;
                  for stop in stop_sequences {
                      if fragment.contains(stop) {
                          should_stop = true;
                          break;
                      }
                  }
                  if should_stop { break; }
                  
                  // PHRASE BOUNDARY detection
                  let has_punctuation = fragment.contains('.') || fragment.contains('!') || 
                                        fragment.contains('?') || fragment.contains('\n') || fragment.contains(',');
                                        
                  // If we have a punctuation or it's getting long, flush.
                  if has_punctuation || fragment.len() > 50 { 
                       // FORCE INTERNAL: The Daemon decides if this becomes vocal.
                       // All raw stream is just "Cortex" activity.
                       let _ = self.thought_tx.send(Thought::new(MindVoice::Cortex, fragment.clone()));
                       current_word_tokens.clear(); // Reset token buffer
                  } else {
                      current_word_tokens.push(next_token);
                  }
            }
        }

        // Send remaining buffer
        if !current_word_tokens.is_empty() {
             if let Ok(fragment) = self.tokenizer.decode(&current_word_tokens, false) {
                 if !fragment.trim().is_empty() {
                     // Force Internal
                     let _ = self.thought_tx.send(Thought::new(MindVoice::Cortex, fragment));
                 }
             }
        }
        
        let full_text = self.tokenizer.decode(&gen_tokens, true).map_err(E::msg)?;
        Ok(full_text.trim().to_string())
    }
}
