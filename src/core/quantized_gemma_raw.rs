// Custom Gemma GGUF Loader for ALEPH
// Based on candle-transformers quantized_llama but adapted for Gemma metadata keys

use candle_core::{DType, Device, Module, Result, Tensor, D};
use candle_core::quantized::{gguf_file, QMatMul};
use std::collections::HashMap;

const MAX_SEQ_LEN: usize = 4096;

#[derive(Debug, Clone)]
pub struct Config {
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub vocab_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub head_dim: usize,
    pub rms_norm_eps: f32,
    pub rope_theta: f32,
}

impl Config {
    pub fn from_gguf(ct: &gguf_file::Content) -> Result<Self> {
        let get = |key: &str| {
            ct.metadata.get(key).ok_or_else(|| {
                candle_core::Error::Msg(format!("missing metadata key: {}", key))
            })
        };
        
        let get_u32 = |key: &str| -> Result<usize> {
            match get(key)? {
                gguf_file::Value::U32(v) => Ok(*v as usize),
                gguf_file::Value::U64(v) => Ok(*v as usize),
                gguf_file::Value::I32(v) => Ok(*v as usize),
                v => Err(candle_core::Error::Msg(format!("{} has wrong type: {:?}", key, v))),
            }
        };
        
        let get_f32 = |key: &str| -> Result<f32> {
            match get(key)? {
                gguf_file::Value::F32(v) => Ok(*v),
                v => Err(candle_core::Error::Msg(format!("{} has wrong type: {:?}", key, v))),
            }
        };

        // Gemma uses "gemma." prefix for its metadata
        let hidden_size = get_u32("gemma.embedding_length")?;
        let intermediate_size = get_u32("gemma.feed_forward_length")?;
        let num_hidden_layers = get_u32("gemma.block_count")?;
        let num_attention_heads = get_u32("gemma.attention.head_count")?;
        let num_key_value_heads = get_u32("gemma.attention.head_count_kv")?;
        let head_dim = get_u32("gemma.attention.key_length").unwrap_or(hidden_size / num_attention_heads);
        let rms_norm_eps = get_f32("gemma.attention.layer_norm_rms_epsilon").unwrap_or(1e-6);
        let rope_theta = get_f32("gemma.rope.freq_base").unwrap_or(10000.0);

        Ok(Self {
            hidden_size,
            intermediate_size,
            vocab_size: 256000, // Gemma vocab size
            num_hidden_layers,
            num_attention_heads,
            num_key_value_heads,
            head_dim,
            rms_norm_eps,
            rope_theta,
        })
    }
}

fn rms_norm(x: &Tensor, weight: &Tensor, eps: f32) -> Result<Tensor> {
    let x_dtype = x.dtype();
    let x = x.to_dtype(DType::F32)?;
    let variance = x.sqr()?.mean_keepdim(D::Minus1)?;
    let x_normed = x.broadcast_div(&(variance + eps as f64)?.sqrt()?)?;
    // Gemma: multiply by (1 + weight) instead of just weight
    let weight_plus_one = (weight.to_dtype(DType::F32)? + 1.0)?;
    let result = x_normed.broadcast_mul(&weight_plus_one)?;
    result.to_dtype(x_dtype)
}

struct LayerWeights {
    attn_q: QMatMul,
    attn_k: QMatMul,
    attn_v: QMatMul,
    attn_output: QMatMul,
    ffn_gate: QMatMul,
    ffn_up: QMatMul,
    ffn_down: QMatMul,
    attn_norm: Tensor,
    ffn_norm: Tensor,
    n_heads: usize,
    n_kv_heads: usize,
    head_dim: usize,
    cos: Tensor,
    sin: Tensor,
    kv_cache: Option<(Tensor, Tensor)>,
}

impl LayerWeights {
    fn apply_rotary_emb(&self, x: &Tensor, index_pos: usize) -> Result<Tensor> {
        let (_b_sz, _n_head, seq_len, _n_embd) = x.dims4()?;
        let cos = self.cos.narrow(0, index_pos, seq_len)?;
        let sin = self.sin.narrow(0, index_pos, seq_len)?;
        candle_nn::rotary_emb::rope_i(&x.contiguous()?, &cos, &sin)
    }

    fn forward(&mut self, x: &Tensor, mask: Option<&Tensor>, index_pos: usize, rms_eps: f32) -> Result<Tensor> {
        let (b_sz, seq_len, _hidden) = x.dims3()?;
        let n_embd = self.n_heads * self.head_dim;

        // Pre-attention RMSNorm
        let x_normed = rms_norm(x, &self.attn_norm, rms_eps)?;

        // Q, K, V projections
        let q = self.attn_q.forward(&x_normed)?;
        let k = self.attn_k.forward(&x_normed)?;
        let v = self.attn_v.forward(&x_normed)?;

        let q = q.reshape((b_sz, seq_len, self.n_heads, self.head_dim))?.transpose(1, 2)?;
        let k = k.reshape((b_sz, seq_len, self.n_kv_heads, self.head_dim))?.transpose(1, 2)?;
        let v = v.reshape((b_sz, seq_len, self.n_kv_heads, self.head_dim))?.transpose(1, 2)?.contiguous()?;

        // RoPE
        let q = self.apply_rotary_emb(&q, index_pos)?;
        let k = self.apply_rotary_emb(&k, index_pos)?;

        // KV Cache
        let (k, v) = match &self.kv_cache {
            None => (k, v),
            Some((k_cache, v_cache)) => {
                if index_pos == 0 {
                    (k, v)
                } else {
                    let k = Tensor::cat(&[k_cache, &k], 2)?;
                    let v = Tensor::cat(&[v_cache, &v], 2)?;
                    (k, v)
                }
            }
        };
        self.kv_cache = Some((k.clone(), v.clone()));

        // GQA repeat for attention
        let n_rep = self.n_heads / self.n_kv_heads;
        let k = if n_rep > 1 {
            let k = k.unsqueeze(2)?;
            let k = k.expand((b_sz, self.n_kv_heads, n_rep, k.dim(3)?, self.head_dim))?;
            k.reshape((b_sz, self.n_heads, k.dim(3)?, self.head_dim))?
        } else {
            k
        };
        let v = if n_rep > 1 {
            let v = v.unsqueeze(2)?;
            let v = v.expand((b_sz, self.n_kv_heads, n_rep, v.dim(3)?, self.head_dim))?;
            v.reshape((b_sz, self.n_heads, v.dim(3)?, self.head_dim))?
        } else {
            v
        };

        // Attention scores
        let scale = 1.0 / (self.head_dim as f64).sqrt();
        let att = (q.matmul(&k.t()?)? * scale)?;
        
        let att = match mask {
            Some(m) => {
                let m = m.broadcast_as(att.shape())?;
                att.broadcast_add(&m)?
            }
            None => att,
        };
        
        let att = candle_nn::ops::softmax_last_dim(&att)?;
        let y = att.matmul(&v.contiguous()?)?;
        
        let y = y.transpose(1, 2)?.reshape(&[b_sz, seq_len, n_embd])?;
        let attn_out = self.attn_output.forward(&y)?;
        
        // Residual connection for attention
        let x = (x + attn_out)?;

        // FFN with pre-norm
        let x_normed = rms_norm(&x, &self.ffn_norm, rms_eps)?;
        
        // Gemma uses GeGLU: gate * up, then down
        let gate = self.ffn_gate.forward(&x_normed)?;
        let up = self.ffn_up.forward(&x_normed)?;
        // GELU activation for gate (use Tensor method)
        let gate = gate.gelu_erf()?;
        let ffn_out = self.ffn_down.forward(&(gate * up)?)?;
        
        // Residual connection for FFN
        let x = (x + ffn_out)?;
        
        Ok(x)
    }
}

pub struct ModelWeights {
    tok_embeddings: candle_nn::Embedding,
    layers: Vec<LayerWeights>,
    norm: Tensor,
    output: QMatMul,
    config: Config,
    masks: HashMap<usize, Tensor>,
}

fn precompute_freqs_cis(head_dim: usize, rope_theta: f32, device: &Device) -> Result<(Tensor, Tensor)> {
    let theta: Vec<_> = (0..head_dim)
        .step_by(2)
        .map(|i| 1f32 / rope_theta.powf(i as f32 / head_dim as f32))
        .collect();
    let theta = Tensor::new(theta.as_slice(), device)?;
    let idx_theta = Tensor::arange(0, MAX_SEQ_LEN as u32, device)?
        .to_dtype(DType::F32)?
        .reshape((MAX_SEQ_LEN, 1))?
        .matmul(&theta.reshape((1, theta.elem_count()))?)?;
    let cos = idx_theta.cos()?;
    let sin = idx_theta.sin()?;
    Ok((cos, sin))
}

impl ModelWeights {
    pub fn from_gguf<R: std::io::Seek + std::io::Read>(
        ct: gguf_file::Content,
        reader: &mut R,
        device: &Device,
    ) -> Result<Self> {
        let config = Config::from_gguf(&ct)?;
        let (cos, sin) = precompute_freqs_cis(config.head_dim, config.rope_theta, device)?;

        // Token embeddings
        let tok_embeddings = ct.tensor(reader, "token_embd.weight", device)?;
        let tok_embeddings = tok_embeddings.dequantize(device)?;
        let tok_embeddings = candle_nn::Embedding::new(tok_embeddings, config.hidden_size);

        // Final norm
        let norm = ct.tensor(reader, "output_norm.weight", device)?.dequantize(device)?;

        // Output projection (often tied to embeddings, but check if exists)
        let output = ct.tensor(reader, "output.weight", device)
            .or_else(|_| ct.tensor(reader, "token_embd.weight", device))?;
        let output = QMatMul::from_qtensor(output)?;

        // Layers
        let mut layers = Vec::with_capacity(config.num_hidden_layers);
        for layer_idx in 0..config.num_hidden_layers {
            let prefix = format!("blk.{}.", layer_idx);
            
            let attn_norm = ct.tensor(reader, &format!("{}attn_norm.weight", prefix), device)?.dequantize(device)?;
            let ffn_norm = ct.tensor(reader, &format!("{}ffn_norm.weight", prefix), device)?.dequantize(device)?;

            // Gemma uses separate Q, K, V (not fused)
            let attn_q = QMatMul::from_qtensor(ct.tensor(reader, &format!("{}attn_q.weight", prefix), device)?)?;
            let attn_k = QMatMul::from_qtensor(ct.tensor(reader, &format!("{}attn_k.weight", prefix), device)?)?;
            let attn_v = QMatMul::from_qtensor(ct.tensor(reader, &format!("{}attn_v.weight", prefix), device)?)?;
            let attn_output = QMatMul::from_qtensor(ct.tensor(reader, &format!("{}attn_output.weight", prefix), device)?)?;

            let ffn_gate = QMatMul::from_qtensor(ct.tensor(reader, &format!("{}ffn_gate.weight", prefix), device)?)?;
            let ffn_up = QMatMul::from_qtensor(ct.tensor(reader, &format!("{}ffn_up.weight", prefix), device)?)?;
            let ffn_down = QMatMul::from_qtensor(ct.tensor(reader, &format!("{}ffn_down.weight", prefix), device)?)?;

            layers.push(LayerWeights {
                attn_q,
                attn_k,
                attn_v,
                attn_output,
                ffn_gate,
                ffn_up,
                ffn_down,
                attn_norm,
                ffn_norm,
                n_heads: config.num_attention_heads,
                n_kv_heads: config.num_key_value_heads,
                head_dim: config.head_dim,
                cos: cos.clone(),
                sin: sin.clone(),
                kv_cache: None,
            });
        }

        Ok(Self {
            tok_embeddings,
            layers,
            norm,
            output,
            config,
            masks: HashMap::new(),
        })
    }

    fn mask(&mut self, t: usize, device: &Device) -> Result<Tensor> {
        if let Some(mask) = self.masks.get(&t) {
            return Ok(mask.clone());
        }
        // Causal mask: -inf where j > i
        let mask: Vec<f32> = (0..t)
            .flat_map(|i| (0..t).map(move |j| if j > i { f32::NEG_INFINITY } else { 0.0 }))
            .collect();
        let mask = Tensor::from_slice(&mask, (t, t), device)?;
        self.masks.insert(t, mask.clone());
        Ok(mask)
    }

    pub fn forward(&mut self, x: &Tensor, index_pos: usize) -> Result<Tensor> {
        let (_b_sz, seq_len) = x.dims2()?;
        
        // Embedding + Gemma scaling
        let mut h = self.tok_embeddings.forward(x)?;
        h = (h * (self.config.hidden_size as f64).sqrt())?;

        // Compute mask for full sequence
        let mask = if seq_len > 1 {
            let full_mask = self.mask(seq_len + index_pos, x.device())?;
            Some(full_mask.narrow(0, index_pos, seq_len)?)
        } else {
            None
        };

        // Transformer layers
        for layer in &mut self.layers {
            h = layer.forward(&h, mask.as_ref(), index_pos, self.config.rms_norm_eps)?;
        }

        // Final norm
        h = rms_norm(&h, &self.norm, self.config.rms_norm_eps)?;

        // Output projection (logits)
        let logits = self.output.forward(&h)?;
        
        Ok(logits)
    }
}