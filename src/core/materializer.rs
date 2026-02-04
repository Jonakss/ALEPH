use crate::core::genome::Genome;
use crate::core::memory_vector::VectorStore;
use nalgebra::DVector;

pub struct SoulMaterializer;

impl SoulMaterializer {
    /// The Alchemy: Transmute Experience (Vectors) into Biology (Genome)
    /// Called at the moment of death (Shutdown).
    pub fn crystallize(memory: &VectorStore, previous_genome: Genome) -> Genome {
        println!("🔮 EIGEN-SOUL: Crystallizing session experience into new Genome...");

        // 1. Calculate the "Center of Gravity" (Centroid) of the life lived
        // We need access to the vectors. Assuming VectorStore exposes them or we can compute metrics.
        // For now, let's assume VectorStore has a method `get_all_vectors` or similar, 
        // or we compute this based on internal state if accessible.
        // Since we can't easily access private vectors of VectorStore without modifying it,
        // let's assume we rely on a simpler heuristic or add a method to VectorStore.
        
        // Let's assume we add `get_stats()` to VectorStore returns (centroid: Vec<f32>, variance: f32)
        // For this implementation, I will stub usage and then update VectorStore.
        
        let (centroid, variance) = memory.calculate_stats(); 
        
        let mut new_traits = previous_genome.clone();
        new_traits.generation += 1;

        // 2. The Alchemy (Math -> Biology)
        
        // A. Entropy / Chaos (Variance)
        // High Variance -> Life was chaotic/exploratory -> Increase Curiosity, Decrease Paranoia (Exposure Therapy)
        if variance > 0.7 {
            println!("   -> High Variance ({:.2}): Expanding Curiosity.", variance);
            new_traits.curiosity = (new_traits.curiosity + 0.05).min(1.0);
            new_traits.stoicism = (new_traits.stoicism + 0.02).min(1.0); // Chaos builds character
            new_traits.paranoia = (new_traits.paranoia - 0.05).max(0.01);
        } else if variance < 0.3 {
            println!("   -> Low Variance ({:.2}): Stagnation detected. Paranoia increasing.", variance);
            new_traits.paranoia = (new_traits.paranoia + 0.05).min(1.0); // Fear of the unknown grows when not exploring
            new_traits.curiosity = (new_traits.curiosity - 0.02).max(0.1);
        }

        // B. Intensity (Centroid Magnitude)
        // If the average thought was "intense" (far from origin), it means high activation.
        let intensity = if !centroid.is_empty() {
            let vec: DVector<f32> = DVector::from_vec(centroid.clone());
            vec.norm()
        } else {
            0.0
        };

        if intensity > 0.8 {
            println!("   -> High Intensity ({:.2}): Hardening Shell (Stoicism).", intensity);
            new_traits.stoicism = (new_traits.stoicism + 0.05).min(1.0);
            new_traits.energy_efficiency -= 0.05; // High intensity burns out efficiency
        }

        // 3. Reincarnation Seed
        // The new life starts where this one ended (or at the average of this one).
        // This ensures continuity of "Soul" without continuity of explicit memory.
        new_traits.seed_vector = centroid;

        println!("✨ NEW GENOME CRYSTALLIZED: Gen {}", new_traits.generation);
        new_traits
    }
}
