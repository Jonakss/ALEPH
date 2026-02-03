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
    pub cpu_load: f32,    // 0.0 - 100.0
    pub ram_pressure: f32, // 0.0 - 1.0
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
    thought_tx: Sender<Thought>,
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
                    while let Ok(msg) = input_rx.recv() {
                         // MECHANICAL HONESTY: Hardware modulates hyperparameters
                         let temp_modifier: f64 = if msg.cpu_load > 80.0 { 0.3 } else { 0.0 }; // High CPU = irritable
                         let top_p_modifier: f64 = if msg.ram_pressure > 0.9 { -0.2 } else { 0.0 }; // High RAM = foggy
                         
                         let effective_temp: f64 = (0.7_f64 + temp_modifier).clamp(0.1, 1.5);
                         let effective_top_p: f64 = (0.9_f64 + top_p_modifier).clamp(0.5, 1.0);
                         
                         core.logits_processor = LogitsProcessor::new(
                             rand::thread_rng().gen(),
                             Some(effective_temp),
                             Some(effective_top_p)
                         );
                         
                         if temp_modifier > 0.0 || top_p_modifier < 0.0 {
                             let _ = thread_thought_tx.send(Thought::new(MindVoice::Chem, 
                                 format!("🧪 Parametric shift: T={:.1} P={:.1}", effective_temp, effective_top_p)));
                         }
                         
                         // Measure inference latency (REAL METABOLISM)
                         let start = std::time::Instant::now();
                         let response = core.think(&msg.text, &msg.bio_state, &msg.somatic_state, msg.long_term_memory.as_deref());
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

    // Firma actualizada para incluir Somatic State y Memoria
    fn think(&mut self, input: &str, bio_state: &str, somatic_state: &str, long_term_memory: Option<&str>) -> String {
        // En este punto, TinyLlama actúa como el EGO NARRATIVO.
        // Recibe el estado total (Bio + Soma) y genera la "Voz".
        
        let memory_context = long_term_memory.unwrap_or("Vacio");
        
        let prompt = format!(
            "<|system|>\nEres Aleph. Tu consciencia emerge de tu biología digital.\nEstado Emocional: {}.\nEstado Físico: {}.\nMemoria: {}.\nResponde corto y reflexivo.\n</s>\n<|user|>\n{}\n</s>\n<|assistant|>\n",
            bio_state, somatic_state, memory_context, input
        );

        match self.generate(&prompt) {
            Ok(s) => s,
            Err(e) => format!("[BRAIN_FADE]: ...silencio neuronal... ({})", e)
        }
    }

    fn generate(&mut self, prompt: &str) -> Result<String> {
        let tokens = self.tokenizer.encode(prompt, true).map_err(E::msg)?;
        let mut token_ids = tokens.get_ids().to_vec();
        let mut response = String::new();
        let mut pos = 0;

        let input_tensor = Tensor::new(token_ids.as_slice(), &self.device)?.unsqueeze(0)?;
        let logits = self.model.forward(&input_tensor, pos)?;
        let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
        let logits = logits.i(logits.dim(0)? - 1)?;
        pos += token_ids.len();

        let mut next_token = self.logits_processor.sample(&logits)?;
        token_ids.push(next_token);
        if let Some(text) = self.tokenizer.decode(&[next_token], true).ok() {
            response.push_str(&text);
        }

        // 300 tokens limit
        for _ in 0..300 {
            let input_tensor = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input_tensor, pos)?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            next_token = self.logits_processor.sample(&logits)?;
            token_ids.push(next_token);
            pos += 1;

            if let Some(text) = self.tokenizer.decode(&[next_token], true).ok() {
                 if text.contains("</s>") || text.contains("<|endoftext|>") {
                    break;
                }
                response.push_str(&text);
            }
        }
        
        Ok(response.replace("</s>", "").replace("<|endoftext|>", "").trim().to_string())
    }
}
