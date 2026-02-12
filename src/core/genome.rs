use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;
// use rand::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Genome {
    pub generation: u32,
    
    // --- TRAITS (0.0 - 1.0) ---
    pub stress_tolerance: f32,   // Resistance to Cortisol/Adenosine
    pub curiosity: f32,          // Sensitivity to Novelty (Dopamine gain)
    pub energy_efficiency: f32,  // Metabolic burn rate
    pub paranoia: f32,           // Membrane sensitivity (Inflammation threshold)
    pub refractive_index: f32,   // Interpretation bias (0.5 = Neutral, <0.5 Pessimist, >0.5 Optimist)
    
    // --- INSTINCTS ---
    // --- INSTINCTS ---
    pub survival_drive: f32,     // Will to live
    
    // --- EIGEN-SOUL ---
    pub stoicism: f32,           // Resistance to emotional volatility
    pub seed_vector: Vec<f32>,   // The crystallization of the previous life
}

impl Default for Genome {
    fn default() -> Self {
        Self {
            generation: 1,
            stress_tolerance: 0.5,
            curiosity: 0.5,
            energy_efficiency: 0.5,
            paranoia: 0.1,
            refractive_index: 0.5,
            survival_drive: 0.8,
            stoicism: 0.1,
            seed_vector: vec![0.0; 384], // Default embedding size (e.g., all-MiniLM-L6-v2)
        }
    }
}

impl Genome {
    pub fn load() -> Result<Self> {
        let path = "genome.json";
        if let Ok(content) = fs::read_to_string(path) {
            let mut genome: Genome = serde_json::from_str(&content)?;
            
            // GENETIC REPAIR: If user cleared the vector manually
            if genome.seed_vector.is_empty() {
                 println!("🧬 DNA DAMAGE DETECTED: Empty Seed Vector. Regenerating Sequence...");
                 // Generate random noise for new seed
                 let _rng = rand::thread_rng();
                 use rand::Rng; // Ensure Rng is in scope or just map
                 genome.seed_vector = (0..384).map(|_| rand::thread_rng().gen_range(-0.1..0.1)).collect();
            }
            
            Ok(genome)
        } else {
            // Genesis
            let genome = Genome::default();
            genome.save()?;
            Ok(genome)
        }
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write("genome.json", json)?;
        Ok(())
    }

    // mutate removed (unused)
}
