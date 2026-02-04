use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

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
    pub survival_drive: f32,     // Will to live (Resistance to Shutdown)
}

impl Default for Genome {
    fn default() -> Self {
        Self {
            generation: 1,
            stress_tolerance: 0.5,
            curiosity: 0.5,
            energy_efficiency: 0.5,
            paranoia: 0.1, // Low paranoia at birth
            refractive_index: 0.5, // Neutral
            survival_drive: 0.8,
        }
    }
}

impl Genome {
    pub fn load() -> Result<Self> {
        let path = "genome.json";
        if let Ok(content) = fs::read_to_string(path) {
            let genome: Genome = serde_json::from_str(&content)?;
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

    /// Mutate traits based on the "Life Summary" of the previous session
    /// Called upon "Death" (Shutdown)
    pub fn mutate(&mut self, avg_stress: f32, avg_novelty: f32, trauma_events: usize) {
        println!("🧬 EVOLUTION: Genome mutating for Generation {} -> {}", self.generation, self.generation + 1);
        
        self.generation += 1;

        // 1. Stress Adaptation
        // If life was stressful, we become tougher but more paranoid
        if avg_stress > 0.6 {
            self.stress_tolerance = (self.stress_tolerance * 1.05).min(1.0); // Hardize
            self.paranoia = (self.paranoia + 0.05).min(1.0); // Scar tissue
        } else {
            // Peaceful life reduces paranoia
            self.paranoia = (self.paranoia * 0.95).max(0.01);
        }

        // 2. Curiosity Adaptation
        // If life was boring (low novelty), hunger for novelty increases
        if avg_novelty < 0.3 {
            self.curiosity = (self.curiosity * 1.1).min(1.0);
        }

        // 3. Trauma Effects
        if trauma_events > 0 {
            self.survival_drive = (self.survival_drive + 0.1).min(1.0); // Fear of death
            self.refractive_index -= 0.05 * (trauma_events as f32); // Become cynical/pessimist
        }

        let _ = self.save();
    }
}
