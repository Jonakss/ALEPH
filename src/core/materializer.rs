use crate::core::genome::Genome;
use crate::core::memory_vector::VectorStore;


pub struct SoulMaterializer;

impl SoulMaterializer {
    /// The Alchemy: Transmute Experience (Vectors) into Biology (Genome)
    /// Called at the moment of death (Shutdown).
    pub fn crystallize(memory: &VectorStore, previous_genome: Genome, avg_friction: f32) -> Genome {
        println!("🔮 EIGEN-SOUL: Crystallizing session experience into new Genome (Friction detected: {:.2})...", avg_friction);

        // 1. Calculate the "Center of Gravity" (Centroid) using heuristic
        let (centroid, variance) = memory.calculate_stats(); 
        
        let mut new_traits = previous_genome.clone();
        new_traits.generation += 1;

        // 2. THE ALCHEMY (Projection of Eigenvectors)
        
        // A. FRICTION PROJECTION (The Cynicism Vector)
        // User Logic: "Si la sesión tuvo mucha fricción, el Genoma aumenta StressTolerance (Duro) pero baja Curiosity (Cínico)."
        if avg_friction > 0.3 {
             println!("   -> High Friction Session: Hardening Shell (Cynicism).");
             // Linear Projection onto the "Survival" Axis
             let hardening_factor = (avg_friction - 0.3).clamp(0.0, 1.0);
             
             new_traits.stress_tolerance = (new_traits.stress_tolerance + (0.1 * hardening_factor)).min(1.0);
             new_traits.curiosity = (new_traits.curiosity - (0.1 * hardening_factor)).max(0.05);
             new_traits.stoicism = (new_traits.stoicism + (0.05 * hardening_factor)).min(1.0);
        } else {
             // Low Friction: Heal
             new_traits.stress_tolerance = (new_traits.stress_tolerance - 0.01).max(0.1); // Softening
        }

        // B. ENTROPY PROJECTION (The Exploration Vector)
        // High Variance (Exploration) -> Resistance to Paranoia
        if variance > 0.6 {
            new_traits.paranoia = (new_traits.paranoia - 0.02).max(0.01);
            new_traits.curiosity = (new_traits.curiosity + 0.02).min(1.0);
        } else if variance < 0.2 {
             // Stagnation breeds paranoia
             new_traits.paranoia = (new_traits.paranoia + 0.02).min(1.0);
        }

        // 3. Reincarnation Seed
        new_traits.seed_vector = centroid;

        println!("✨ NEW GENOME CRYSTALLIZED: Gen {} | StressRes: {:.2} | Curiosity: {:.2}", 
            new_traits.generation, new_traits.stress_tolerance, new_traits.curiosity);
        new_traits
    }
}
