

#[derive(Debug, Clone)]
pub struct Neurotransmitters {
    pub adenosine: f32, // Sleep Pressure (0.0 - 1.0)
    pub dopamine: f32,  // Engagement/Reward (0.0 - 1.0)
    pub cortisol: f32,  // Stress (0.0 - 1.0)
    pub oxytocin: f32,  // Trust/Bonding (0.0 - 1.0) - Social Glue
}

impl Neurotransmitters {
    pub fn new() -> Self {
        Self {
            adenosine: 0.0,
            dopamine: 0.5, // Baseline
            cortisol: 0.0,
            oxytocin: 0.5, // Baseline trust
        }
    }

    pub fn tick(&mut self, entropy: f32, cpu_load: f32, is_dreaming: bool, shock_impact: f32, current_neurons: usize, delta_time: f32) {
        // Normalization factor: all constants were tuned for 60Hz
        let time_scale = delta_time / (1.0 / 60.0);

        // 1. ADENOSINE (Fatigue)
        if is_dreaming {
            // Recovery (Sleep)
            self.adenosine -= 0.001 * time_scale; // Faster recovery
        } else {
            // Decay (Awake) - VERY SLOW base fatigue
            // At 60Hz, this is ~0.0006 per second base. Takes ~28 minutes to reach 100% from 0.
            let base_fatigue = 0.00001 * time_scale; 
            let cognitive_load = entropy * 0.00005 * time_scale; // Much slower cognitive cost
            let resilience = (current_neurons as f32 / 500.0).clamp(0.5, 3.0);
            
            let total_load = (base_fatigue + cognitive_load) / resilience;
            self.adenosine += total_load;
            self.adenosine += shock_impact * 0.02 * time_scale; // Reduced trauma impact
        }

        // 2. DOPAMINE (Novelty/Reward)
        // Decays fast (Boredom is the enemy)
        self.dopamine -= 0.005 * time_scale; // 2.5x Decay rate
        
        // Spikes with Entropic Activity (Novelty)
        if entropy > 0.4 { // Lower threshold for reward
            let reward = (entropy - 0.4) * 0.02 * time_scale;
            self.dopamine += reward;
        }

        // 3. CORTISOL (Stress)
        // Audio Shock / Trauma
        let stress_sources = shock_impact * 5.0; // 2.5x Shock sensitivity
        
        if entropy > 0.8 || cpu_load > 60.0 {
            // Overloaded
            self.cortisol += 0.01 * time_scale + stress_sources;
        } else {
            // Recovery (Calm)
            if shock_impact < 0.01 {
               self.cortisol -= 0.004 * time_scale; // Faster recovery
            }
            self.cortisol += stress_sources;
        }

        // 4. OXYTOCIN (Trust)
        // Decays slowly
        self.oxytocin -= 0.001 * time_scale; 

        // 5. HOMEOSTATIC NOISE (The "Breath" of the system)
        // Prevents static flatlines
        let noise = (entropy * 0.001) - 0.0005;
        self.dopamine += noise;
        self.cortisol += noise;

        // CLAMPING
        self.adenosine = self.adenosine.clamp(0.0, 1.0);
        self.dopamine = self.dopamine.clamp(0.0, 1.0);
        self.cortisol = self.cortisol.clamp(0.0, 1.0);
        self.oxytocin = self.oxytocin.clamp(0.0, 1.0);
    }

    /// Map Hardware Proprioception to Biological States
    pub fn update_from_hardware(&mut self, cpu_load: f32, ram_usage: f32, _battery_level: f32) {
        // 1. CPU -> Metabolism / Heart Rate (Stress floor)
        // High CPU = High Metabolic Burn = Higher minimal Cortisol
        if cpu_load > 80.0 {
            self.cortisol = self.cortisol.max(0.4); // Stress floor
        }
        
        // 2. RAM -> Brain Fog (Adenosine floor in extreme cases)
        // If RAM is full, the "brain" is clogged. 
        if ram_usage > 0.9 {
            self.adenosine = self.adenosine.max(0.8); // Forced fatigue
        }
    }
    
    /// Set adenosine base level from unprocessed memory count (MECHANICAL HONESTY)
    /// This REPLACES the memory component, not adds to it
    // Unused methods removed (set_memory_pressure, is_body_failing, is_recovered_to_wake)
    
    /// Get degradation factor for inference (brain fog)
    pub fn get_cognitive_impairment(&self) -> f32 {
        // 0.0 = no impairment, 1.0 = max impairment
        // Kicks in gradually above 50% adenosine
        if self.adenosine > 0.5 {
            ((self.adenosine - 0.5) * 2.0).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}
