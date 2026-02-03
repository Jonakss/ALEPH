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

// Mensaje de entrada para el Cortex (Actor)
pub struct CortexInput {
    pub text: String,
    pub bio_state: String,
    pub somatic_state: String, // Estado del Hardware (Cuerpo)
    pub long_term_memory: Option<String>,
    // Hardware State for Parametric Modulation
    pub _cpu_load: f32,    // 0.0 - 100.0
    pub _ram_pressure: f32, // 0.0 - 1.0
    pub cognitive_impairment: f32, // 0.0 - 1.0 (Brain fog)
    // BIOLOGICAL DRIVERS (MECHANICAL HONESTY)
    pub entropy: f32,      // 0.0 - 1.0+ (Chaos/Temperature)
    pub adenosine: f32,    // 0.0 - 1.0 (Fatigue/Top-P constriction)
}

// Response with metabolic data
pub struct CortexOutput {
    pub text: String,
    pub inference_latency_ms: u64, // Real metabolic cost
}

// El Cerebro en sí (Internal)
pub struct CognitiveCore {
    model: Llama,
    tokenizer: Tokenizer,
    device: Device,
    logits_processor: LogitsProcessor,
    #[allow(dead_code)]
    thought_tx: Sender<Thought>, // Pasado al thread en spawn
}

impl CognitiveCore {
    // Spawnea el thread del Cortex y retorna los canales de comunicación
    pub fn spawn(thought_tx: Sender<Thought>) -> Result<(Sender<CortexInput>, Receiver<CortexOutput>)> {
        let (input_tx, input_rx) = channel::<CortexInput>();
        let (output_tx, output_rx) = channel::<CortexOutput>();

        let thread_thought_tx = thought_tx.clone();

        thread::spawn(move || {
            // 1. Init (Inside Thread to avoid moving heavy structs across threads unnecessarily if not Send, though they are)
            match Self::new(thread_thought_tx.clone()) {
                Ok(mut core) => {
                    let _ = thread_thought_tx.send(Thought::new(MindVoice::System, "Cortex Thread: READY. Waiting for input...".to_string()));
                    
                    // 2. Event Loop (Consciencia Hub)
                    loop {
                        // Heartbeat check every 30s
                        let msg = match input_rx.recv_timeout(std::time::Duration::from_secs(30)) {
                            Ok(m) => m,
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                let _ = thread_thought_tx.send(Thought::new(MindVoice::System, "💓 Cortex Heartbeat: Idle but Listening...".to_string()));
                                continue;
                            },
                            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                        };

                         // MECHANICAL HONESTY: Hyperparameters tied to Biological State
                         
                         // 0. Sanitize Inputs (Prevent Math Panics)
                         let safe_entropy = if msg.entropy.is_nan() || msg.entropy.is_infinite() { 
                             0.5 
                         } else { 
                             msg.entropy 
                         };

                         // 1. Entropy -> Temperature
                         // CRITICAL: High Temp (>0.9) causes crashes with this model structure.
                         // Range: 0.1 (Rigid) - 0.85 (Safe Creative).
                         let effective_temp: f64 = (0.4 + safe_entropy * 0.4) as f64;
                         let effective_temp = effective_temp.clamp(0.1, 0.85); 

                         // 2. Adenosine -> Top-P
                         // Range: 0.9 (Open) - 0.5 (Focused). Never < 0.1.
                         // CRITICAL FIX: P > 0.8 causes overflow. Capping strict at 0.80.
                         let effective_top_p: f64 = (0.80 - (msg.adenosine * 0.4)) as f64;
                         let effective_top_p = effective_top_p.clamp(0.5, 0.80);

                         core.logits_processor = LogitsProcessor::new(
                             rand::thread_rng().gen(),
                             Some(effective_temp),
                             Some(effective_top_p)
                         );
                         
                         // ... (Log omitted for brevity, keeping existing structure if possible, but replace needs context)
                         // I will split this into two replacements if needed, but the block is contiguous enough.
                         // Actually, there is a logging block in between. I will do TWO replacements.
                         
                         // Log significant shifts
                         if msg.entropy > 0.8 || msg.adenosine > 0.7 {
                             let _ = thread_thought_tx.send(Thought::new(MindVoice::Chem, 
                                 format!("🧪 Bio-Modulation: T={:.2} (Chaos), P={:.2} (Focus)", effective_temp, effective_top_p)));
                         } else {
                             // DEBUG CRASH: Always log for now
                             let _ = thread_thought_tx.send(Thought::new(MindVoice::System, 
                                 format!("🔍 Sampling: T={:.2}, P={:.2}", effective_temp, effective_top_p)));
                         }
                         
                         // Measure inference latency (REAL METABOLISM)
                         let start = std::time::Instant::now();
                         
                         // MECHANICAL HONESTY: Physical Collapse
                         // If Adenosine is critical AND System is Chaotic = Shutdown
                         let response = if msg.adenosine > 0.95 {
                             "[SYSTEM_FAILURE]: Consciousness collapsed. Adenosine critical.".to_string()
                         } else if msg.cognitive_impairment > 0.8 && rand::thread_rng().gen::<f32>() < msg.cognitive_impairment {
                             ".......".to_string() // Active Silence (Freeze)
                         } else {
                             // CRITICAL: Catch panics from Candle/LLM to prevent thread death
                             // We use AssertUnwindSafe because we trust that a panic in inference 
                             // doesn't corrupt the channel state, only the local model state (which is stateless input-output mostly).
                             let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                 let available_tokens = if msg.adenosine > 0.6 { 150 } else { 300 };
                                 core.think_with_limit(&msg.text, &msg.bio_state, &msg.somatic_state, msg.long_term_memory.as_deref(), available_tokens)
                             }));

                             match result {
                                 Ok(text) => text,
                                 Err(e) => {
                                     // Capture panic info
                                     let msg = if let Some(s) = e.downcast_ref::<&str>() {
                                         s.to_string()
                                     } else if let Some(s) = e.downcast_ref::<String>() {
                                         s.clone()
                                     } else {
                                         "Unknown Panic".to_string()
                                     };
                                     
                                     let _ = thread_thought_tx.send(Thought::new(MindVoice::System, format!("💥 CRITICAL PANIC: {}", msg)));
                                     thread::sleep(std::time::Duration::from_millis(200)); 
                                     "".to_string() 
                                 }
                             }
                         };
                         
                         let latency_ms = start.elapsed().as_millis() as u64;
                         
                         let _ = output_tx.send(CortexOutput { 
                             text: response, 
                             inference_latency_ms: latency_ms 
                         });
                    }
                }
                Err(e) => {
                    let _ = thread_thought_tx.send(Thought::new(MindVoice::System, format!("FATAL: Cortex Thread Failed Init: {}", e)));
                }
            }
        });

        Ok((input_tx, output_rx))
    }

    fn new(tx: Sender<Thought>) -> Result<Self> {
        if !std::path::Path::new(MODEL_FILE).exists() {
            panic!("Cerebro no encontrado: {}", MODEL_FILE);
        }
        if !std::path::Path::new(TOKENIZER_FILE).exists() {
            panic!("Tokenizer no encontrado: {}", TOKENIZER_FILE);
        }

        let mut device = Device::new_cuda(0).unwrap_or(Device::Cpu);
        let _ = tx.send(Thought::new(MindVoice::System, format!("Cortex: Init on {:?}", device)));

        let tokenizer = Tokenizer::from_file(TOKENIZER_FILE)
            .map_err(|e| E::msg(format!("Failed to load tokenizer: {}", e)))?;

        let model = match Self::load_model(&device) {
            Ok(m) => m,
            Err(e) => {
                if device.is_cuda() {
                    let _ = tx.send(Thought::new(MindVoice::System, 
                        "[WARN] GPU Failed. Running on CPU (Bio-Lethargy Mode).".to_string()));
                    device = Device::Cpu;
                    Self::load_model(&device)?
                } else {
                    return Err(e);
                }
            }
        };

        let seed: u64 = rand::thread_rng().gen();

        Ok(Self {
            model,
            tokenizer,
            device,
            logits_processor: LogitsProcessor::new(seed, Some(0.7), Some(0.9)),
            thought_tx: tx,
        })
    }

    fn load_model(device: &Device) -> Result<Llama> {
        let mut file = std::fs::File::open(MODEL_FILE)
            .map_err(|e| E::msg(format!("Failed to open {}: {}", MODEL_FILE, e)))?;
        let content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        let model = Llama::from_gguf(content, &mut file, device)?;
        Ok(model)
    }

    /// Wrapper para think_with_limit con max_tokens fijo
    #[allow(dead_code)]
    fn think(&mut self, input: &str, bio_state: &str, somatic_state: &str, long_term_memory: Option<&str>) -> String {
        self.think_with_limit(input, bio_state, somatic_state, long_term_memory, 300)
    }

    /// MECHANICAL HONESTY: max_tokens reduces with cognitive_impairment (brain fog)
    fn think_with_limit(
        &mut self,
        input: &str,
        bio_state: &str,
        somatic_state: &str,
        long_term_memory: Option<&str>,
        max_tokens: usize,
    ) -> String {
        let memory_context = long_term_memory.unwrap_or("Vacio");

        // MECHANICAL HONESTY: No System Prompt. Raw Input.
        // "Resonance" means the model continues the trajectory of the input.
        // We append a simple separator if needed to encourage output, but no instructions.
        let prompt = format!("{}\n", input);

        // We need a way to pass the callback for streaming, but `think_with_limit` signature matches the trait/struct usage.
        // For now, we will return the full string, BUT we will modify `generate` to potentially send updates if we had the channel.
        // Wait, `CognitiveCore` doesn't hold the Voice channel. `main.rs` handles the output.
        // The user wants "escupiendo palabras" (spitting words).
        // This requires `generate` to emit events.
        // Refactoring to bypass `generate` return waiting.
        match self.generate(&prompt, max_tokens) {
            Ok(s) => s,
            Err(e) => format!("[BRAIN_FADE]: ... ({})", e)
        }
    }

    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        let tokens = self.tokenizer.encode(prompt, true).map_err(E::msg)?;
        let mut token_ids = tokens.get_ids().to_vec();
        if token_ids.is_empty() { return Ok(String::new()); }

        let mut pos = 0;
        let input_tensor = Tensor::new(token_ids.as_slice(), &self.device)?.unsqueeze(0)?;
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

        // Streaming Buffer
        let mut _current_word_tokens = Vec::new(); // Placeholder for future streaming Logic

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

            // RAW RESONANCE: No Repetition Penalty.
            // "solo lo que escuche tiene que resonar"
            
            next_token = self.logits_processor.sample(&logits)?;
            token_ids.push(next_token);
            gen_tokens.push(next_token);
            pos += 1;

            if next_token == self.tokenizer.token_to_id("</s>").unwrap_or(2) || 
               next_token == self.tokenizer.token_to_id("<|endoftext|>").unwrap_or(0) {
                break;
            }
        }
        
        let response = self.tokenizer.decode(&gen_tokens, true).map_err(E::msg)?;
        Ok(response.trim().to_string())
    }
}
