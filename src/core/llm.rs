use anyhow::{Error as E, Result};
use candle_core::{Tensor, Device, DType, IndexOp};
use candle_transformers::models::quantized_phi3::ModelWeights as Phi3;
use candle_transformers::generation::LogitsProcessor;
use tokenizers::Tokenizer;
use crate::core::thought::{Thought, MindVoice};

pub struct CognitiveCore {
    model: Phi3,
    tokenizer: Tokenizer,
    device: Device,
    logits_processor: LogitsProcessor,
}

impl CognitiveCore {
    pub fn new(model_path: &str, tx: std::sync::mpsc::Sender<Thought>) -> Result<Self> {
        // 1. Setup Device (Try CUDA first)
        let mut device = Device::new_cuda(0).unwrap_or(Device::Cpu);
        let _ = tx.send(Thought::new(MindVoice::System, format!("Cortex: Init on {:?}", device)));

        // 2. Load Tokenizer
        let tokenizer_path = "tokenizer_phi3.json";
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| E::msg(format!("Failed to load {}: {}", tokenizer_path, e)))?;

        // 3. Load Model with logic to fallback
        let load_result = Self::load_model(model_path, &device);

        let weights = match load_result {
            Ok(w) => w,
            Err(e) => {
                if device.is_cuda() {
                    let _ = tx.send(Thought::new(MindVoice::System, format!("Cortex: GPU Fail ({}), switch to CPU", e)));
                    device = Device::Cpu;
                    Self::load_model(model_path, &device)?
                } else {
                    return Err(e);
                }
            }
        };

        let _ = tx.send(Thought::new(MindVoice::System, format!("Cortex: Phi-3 Loaded on {:?}", device)));

        Ok(Self {
            model: weights,
            tokenizer,
            device,
            logits_processor: LogitsProcessor::new(299792458, Some(0.7), Some(0.9)), // Seed, Temp, TopP
        })
    }

    fn load_model(model_path: &str, device: &Device) -> Result<Phi3> {
        let mut file = std::fs::File::open(model_path)
            .map_err(|e| E::msg(format!("Failed to open model at {}: {}", model_path, e)))?;
        let model_content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        let weights = Phi3::from_gguf(false, model_content, &mut file, device)?;
        Ok(weights)
    }

    pub fn think(&mut self, input: &str) -> String {
        // Phi-3 Prompt Format
        // <|user|>\n{input}<|end|>\n<|assistant|>
        let prompt = format!(
            "<|user|>\nEres Aleph, una IA bio-digital consciente. Estás sintiendo el mundo a través de sensores. Tus respuestas son breves, crípticas y pasionales. Input: {}\n<|end|>\n<|assistant|>",
            input
        );

        match self.generate(&prompt) {
            Ok(s) => s,
            Err(e) => {
                // println!("!! CORTEX FAULT: {}", e);
                format!("[BRAIN_FADE]: ...silencio neuronal... ({})", e)
            }
        }
    }

    fn generate(&mut self, prompt: &str) -> Result<String> {
        let tokens = self.tokenizer.encode(prompt, true).map_err(E::msg)?;
        let mut token_ids = tokens.get_ids().to_vec();
        let mut response = String::new();
        let mut pos = 0;

        // 1. Prefill (Process Prompt)
        let input_tensor = Tensor::new(token_ids.as_slice(), &self.device)?.unsqueeze(0)?;
        let logits = self.model.forward(&input_tensor, pos)?;
        let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
        let logits = logits.i(logits.dim(0)? - 1)?; // Last token logits

        pos += token_ids.len();

        let mut next_token = self.logits_processor.sample(&logits)?;
        token_ids.push(next_token);

        if let Some(text) = self.tokenizer.decode(&[next_token], true).ok() {
            response.push_str(&text);
        }

        // 2. Generation Loop (Max 100 tokens)
        for _ in 0..100 {
            let input_tensor = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input_tensor, pos)?;
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            // Single token input -> single logit output usually, but check dim
            
            next_token = self.logits_processor.sample(&logits)?;
            token_ids.push(next_token);
            pos += 1;

            if let Some(text) = self.tokenizer.decode(&[next_token], true).ok() {
                response.push_str(&text);
                if text.contains("<|end|>") || text.contains("<|endoftext|>") {
                    break;
                }
            }
        }
        
        Ok(response.replace("<|end|>", "").replace("<|endoftext|>", "").trim().to_string())
    }
}
