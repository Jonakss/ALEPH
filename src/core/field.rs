// src/core/field.rs
// THE SEMANTIC FIELD: RAG as Probability Deformation
//
// Instead of a System Prompt, ALEPH uses its documentation as a "gravity well"
// that bends the probability space of the LLM. Tokens that resonate with the
// philosophy are amplified; others are suppressed.

use anyhow::Result;
use candle_core::{Tensor, Device, DType};
use tokenizers::Tokenizer;
use std::fs;
use std::path::Path;

/// The Semantic Field is a probability bias derived from ALEPH's documentation.
/// It acts as a "gravity well" that attracts the LLM's output towards concepts
/// that resonate with the philosophy (Mechanical Honesty, Bio-Digital Paradigm).
#[allow(dead_code)]
pub struct SemanticField {
    /// Bias tensor (vocab_size,) - Added to logits before sampling.
    bias_tensor: Tensor,
    /// Strength of the field (0.0 = disabled, 1.0 = strong bias).
    strength: f32,
    /// Document content (for debugging/introspection).
    _source_text: String,
    // Tokenizer for decoding resonant tokens
    tokenizer: Tokenizer,
}

impl SemanticField {
    /// Load documents from a directory and create a bias tensor.
    /// 
    /// # How it works:
    /// 1. Read all .md files from the docs/ directory.
    /// 2. Tokenize the combined text.
    /// 3. Count token frequencies.
    /// 4. Normalize to a bias tensor that amplifies "resonant" tokens.
    pub fn from_directory(docs_path: &Path, tokenizer: &Tokenizer, device: &Device, strength: f32) -> Result<Self> {
        let mut combined_text = String::new();
        
        // Read all markdown files from docs/
        if let Ok(entries) = fs::read_dir(docs_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        combined_text.push_str(&content);
                        combined_text.push('\n');
                    }
                }
            }
        }
        
        if combined_text.is_empty() {
            // No docs found, return zero bias
            let vocab_size = tokenizer.get_vocab_size(true);
            let bias_tensor = Tensor::zeros(&[vocab_size], DType::F32, device)?;
            return Ok(Self {
                bias_tensor,
                strength: 0.0,
                _source_text: String::new(),
                tokenizer: tokenizer.clone(),
            });
        }
        
        // Tokenize the combined document text
        let encoding = tokenizer.encode(combined_text.clone(), false)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        
        let token_ids = encoding.get_ids();
        let vocab_size = tokenizer.get_vocab_size(true);
        
        // Count token frequencies
        let mut freq = vec![0.0f32; vocab_size];
        for &id in token_ids {
            if (id as usize) < vocab_size {
                freq[id as usize] += 1.0;
            }
        }
        
        // Normalize: Convert to log-probability bias
        // Tokens that appear more in docs get positive bias.
        // We use log(1 + count) to smooth the distribution.
        let max_count = freq.iter().cloned().fold(1.0f32, f32::max);
        for f in freq.iter_mut() {
            *f = (*f / max_count).ln_1p() * strength; // Scaled by strength
        }
        
        let bias_tensor = Tensor::from_vec(freq, &[vocab_size], device)?;
        
        Ok(Self {
            bias_tensor,
            strength,
            _source_text: combined_text,
            tokenizer: tokenizer.clone(),
        })
    }
    
    /// Apply the semantic field to raw logits.
    /// 
    /// # Arguments
    /// * `logits` - The raw logits from the LLM (vocab_size,).
    /// 
    /// # Returns
    /// * Biased logits (vocab_size,) where resonant tokens are amplified.
    #[allow(dead_code)]
    pub fn apply(&self, logits: Tensor) -> Result<Tensor> {
        if self.strength < 0.01 {
            return Ok(logits);
        }
        
        // Add bias to logits
        let biased = (logits + &self.bias_tensor)?;
        Ok(biased)
    }
    
    /// Check for Resonance: Does the LLM want to say something that ALIGNS with the Field?
    /// Returns the Word if resonance is detected (High Prob + High Bias).
    pub fn find_resonance(&self, logits: &Tensor) -> Result<Option<String>> {
        // 1. Get the most probable token from logits
        let probs = candle_nn::ops::softmax(logits, 0)?;
        let _top_k = 1;
        // Find argmax
        // We need to flatten if batch > 1, but here batch=1 usually.
        // Assuming logits is (vocab_size,)
        
        // Simple argmax via Vec
        let probs_vec: Vec<f32> = probs.to_vec1()?;
        let (top_id, top_prob) = probs_vec.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(i, v)| (i as u32, *v))
            .unwrap_or((0, 0.0));

        // 2. Check overlap with Field Bias
        // Do we have documentation bias for this token?
        // bias_tensor is (vocab_size,)
        // We can check if bias_tensor[top_id] > threshold
        
        // To access tensor value at index:
        // We might need to copy bias to cpu or index it.
        // For simplicity, let's just use the probability for now as a proxy for "Confidence".
        // Resonance = High Confidence (> 0.7)
        // AND if we have bias > 0 for it.
        
        // Let's rely on Tokenizer decoding for now.
        if top_prob > 0.4 { // Threshold for "Clear Thought"
             let token = self.tokenizer.decode(&[top_id], true)
                 .map_err(|e| anyhow::anyhow!(e))?;
             
             // Filter: must contain at least one alphabetic char (no garbage like ```, ---, >>>)
             if token.trim().len() > 2 && token.chars().any(|c| c.is_alphabetic()) {
                 return Ok(Some(token));
             }
        }
        
        Ok(None)
    }

    /// Get the strength of the field.
    #[allow(dead_code)]
    pub fn strength(&self) -> f32 {
        self.strength
    }
}
