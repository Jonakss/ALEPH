use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};

use anyhow::Result;
use tokenizers::Tokenizer;
use hf_hub::{api::sync::Api, Repo, RepoType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

// --- ESTRUCTURA DEL RECUERDO ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryRecord {
    pub text: String,
    pub embedding: Vec<f32>,
    pub timestamp: u64,
    pub context_tags: Vec<String>, 
    #[serde(default)] 
    pub entropy: f32, // Intensity/Importance
    #[serde(default)]
    pub consolidated: bool, // True = Long Term (Disk), False = Volatile (RAM)
}

// --- VECTOR STORE (Base de Datos) ---
pub struct VectorStore {
    pub memories: Vec<MemoryRecord>,
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    file_path: String,
}

impl VectorStore {
    /// Inicializa la BD Vectorial y descarga/carga el modelo BERT MiniLM
    pub fn new() -> Result<Self> {
        let device = Device::new_cuda(0).unwrap_or(Device::Cpu);
        
        let model_id = "sentence-transformers/all-MiniLM-L6-v2";
        let revision = "main"; 

        let api = Api::new()?;
        let repo = api.repo(Repo::with_revision(model_id.to_string(), RepoType::Model, revision.to_string()));

        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors")?;

        let config = std::fs::read_to_string(config_filename)?;
        let config: Config = serde_json::from_str(&config)?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| anyhow::anyhow!(e))?;
        
        // Cargar Weights con var_builder (DType Fixed)
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &device)? };
        let model = BertModel::load(vb, &config)?;

        let store = Self {
            memories: Vec::new(),
            model,
            tokenizer,
            device,
            file_path: "memories.json".to_string(),
        };
        
        // store.load_from_disk(); // EGO DEATH: We do not load past lives.
        // println!("🧠 Hippocampus Loaded: {} memories.", store.memories.len());
        
        Ok(store)
    }

    /// Genera el Embedding (Vector) de un texto
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let tokens = self.tokenizer.encode(text, true).map_err(|e| anyhow::anyhow!(e))?;
        let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
        let token_type_ids = Tensor::new(tokens.get_type_ids(), &self.device)?.unsqueeze(0)?;

        let embedding = self.model.forward(&token_ids, &token_type_ids, None)?;
        
        // Mean Pooling (Promedio de los vectores de tokens)
        let (_n_sentence, n_tokens, _hidden_size) = embedding.dims3()?;
        let embeddings = (embedding.sum(1)? / (n_tokens as f64))?;
        let embeddings_vec = embeddings.flatten_all()?.to_vec1::<f32>()?;
        
        // Normalize (para Cosine Similarity)
        let magnitude: f32 = embeddings_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized = embeddings_vec.iter().map(|x| x / magnitude).collect();
        
        Ok(normalized)
    }

    /// Guarda un recuerdo nuevo (RAM ONLY - Volatile)
    /// Guarda un recuerdo nuevo (RAM ONLY - Volatile)
    #[allow(dead_code)]
    pub fn add(&mut self, text: String, tags: Vec<String>, entropy: f32) -> Result<()> {
        let embedding = self.embed(&text)?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        let record = MemoryRecord {
            text,
            embedding,
            timestamp,
            context_tags: tags,
            entropy,
            consolidated: false,
        };
        
        self.memories.push(record);
        // Removed self.save_to_disk() -> Volatile until Consolidated
        Ok(())
    }

    /// Optimized add: Allows passing an already computed embedding
    pub fn add_precalculated(&mut self, text: String, embedding: Vec<f32>, tags: Vec<String>, entropy: f32) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let record = MemoryRecord {
            text,
            embedding,
            timestamp,
            context_tags: tags,
            entropy,
            consolidated: false,
        };
        self.memories.push(record);
        Ok(())
    }

    /// Recupera memorias similares (Semantic Search)
    /// Recupera memorias similares (Semantic Search)
    #[allow(dead_code)]
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<(String, f32)>> {
        if self.memories.is_empty() {
            return Ok(Vec::new());
        }

        let query_vec = self.embed(query)?;
        
        let mut scores: Vec<(usize, f32)> = self.memories.iter().enumerate().map(|(i, mem)| {
            let cosine_sim: f32 = mem.embedding.iter().zip(&query_vec)
                .map(|(a, b)| a * b) 
                .sum();
            (i, cosine_sim)
        }).collect();

        // Sort desc
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let results = scores.into_iter()
            .take(top_k)
            .map(|(idx, score)| (self.memories[idx].text.clone(), score))
            .collect();
            
        Ok(results)
    }

    /// Detecta si el input es nuevo o repetitivo (Habituation)
    /// Retorna la similitud máxima encontrada (0.0 = Nuevo, 1.0 = Idéntico)
    /// Detecta si el input es nuevo o repetitivo (Habituation)
    /// Retorna la similitud máxima encontrada (0.0 = Nuevo, 1.0 = Idéntico)
    #[allow(dead_code)]
    pub fn get_max_similarity(&self, text: &str) -> Result<f32> {
        if self.memories.is_empty() { return Ok(0.0); }
        
        let query_vec = self.embed(text)?;
        
        let max_sim = self.memories.iter()
            .map(|mem| {
                 mem.embedding.iter().zip(&query_vec).map(|(a, b)| a * b).sum::<f32>()
            })
            .fold(0.0f32, |acc, x| f32::max(acc, x));
            
        Ok(max_sim)
    }

    /// Sueño: Poda memorias irrelevantes y guarda en disco las importantes
    pub fn consolidate_memories(&mut self) -> Result<usize> {
        let initial_count = self.memories.len();
        
        // Criterio de Consolidación: Entropía > 0.7 OR Reciente (< 1 min)? No, Sleep Cycle clears day.
        // User rule: "Solo los recuerdos asociados a estados de alta intensidad (Entropía > 0.7) se guardarán"
        
        // Criterio: Entropía > 0.7 (Alta intensidad)
        self.memories.retain(|m| {
            if m.entropy > 0.7 {
                true // Keep (will be saved)
            } else {
                false // Prune (Forget weak memories)
            }
        });
        
        // Mark all remaining as consolidated
        for mem in &mut self.memories {
            mem.consolidated = true;
        }
        
        let final_count = self.memories.len();
        self.save_to_disk()?; 
        
        Ok(initial_count - final_count) // Retorna cuantos olvidó
    }

    pub fn volatile_count(&self) -> usize {
        self.memories.iter().filter(|m| !m.consolidated).count()
    }

    fn save_to_disk(&self) -> Result<()> {
        let json = serde_json::to_string(&self.memories)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }

    /// MECHANICAL HONESTY: Persistence - save identity to disk without consolidation
    /// Called periodically so Aleph retains "past" across restarts
    pub fn save(&self) -> Result<()> {
        self.save_to_disk()
    }

    // load_from_disk removed (unused)
    pub fn memory_count(&self) -> usize {
        self.memories.len()
    }

    /// Calculates Centroid (Mean Vector) and Variance (Spread) of all memories.
    /// Used by SoulMaterializer for crystallization.
    pub fn calculate_stats(&self) -> (Vec<f32>, f32) {
        if self.memories.is_empty() {
            return (vec![], 0.0);
        }

        let dim = self.memories[0].embedding.len();
        let mut sum_vec = vec![0.0; dim];
        
        // 1. Calculate Centroid
        for mem in &self.memories {
            for (i, val) in mem.embedding.iter().enumerate() {
                sum_vec[i] += val;
            }
        }
        
        let count = self.memories.len() as f32;
        let centroid: Vec<f32> = sum_vec.iter().map(|&x| x / count).collect();

        // 2. Calculate Variance (Average Euclidean Distance from Centroid)
        let mut total_dist_sq = 0.0;
        for mem in &self.memories {
            let dist_sq: f32 = mem.embedding.iter().zip(&centroid)
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            total_dist_sq += dist_sq.sqrt(); // Sum of distances (not squared) to be intuitive "Spread"
        }
        
        let variance = total_dist_sq / count;

        (centroid, variance)
    }
}
