

#[derive(Debug, Clone)]
pub struct Neurotransmitters {
    pub adenosine: f32, // Sleep Pressure (0.0 - 1.0)
    pub dopamine: f32,  // Engagement/Reward (0.0 - 1.0)
    pub cortisol: f32,  // Stress (0.0 - 1.0)
}

impl Neurotransmitters {
    pub fn new() -> Self {
        Self {
            adenosine: 0.0,
            dopamine: 0.5, // Baseline
            cortisol: 0.0,
        }
    }

    pub fn tick(&mut self, entropy: f32, cpu_load: f32, is_dreaming: bool, is_trauma: bool, current_neurons: usize) {
        // 1. ADENOSINA (Fatiga)
        if is_dreaming {
            // Dormir limpia la fatiga
            self.adenosine -= 0.005; // Recuperación lenta
        } else {
            // Estar despierto cansa
            // Resilience: Brains with more neurons can handle more load
            let resilience = (current_neurons as f32 / 100.0).max(1.0);
            
            let base_fatigue = 0.000005; // Extremely low baseline
            let cognitive_load = entropy * 0.00005; // Reduced 20x
            let metabolic_load = (cpu_load / 100.0) * 0.0001; // Reduced 20x
            
            // Total load divided by resilience
            let total_load = base_fatigue + cognitive_load + metabolic_load;
            self.adenosine += total_load / resilience;
            
            // Trauma cansa mucho más (bypass resilience partially?)
            if is_trauma {
                self.adenosine += 0.005 / resilience;
            }
        }

        // 2. DOPAMINA (Novedad/Recompensa)
        // Decae naturalmente (Aburrimiento) - SLOWER DECAY
        self.dopamine -= 0.0002; 
        
        // Sube con la entropía (Novedad)
        if entropy > 0.1 && entropy < 0.8 {
            self.dopamine += 0.001;
        }

        // 3. CORTISOL (Estrés)
        // More sensitive trigger: CPU > 40% OR High Entropy (Confusion)
        if is_trauma || cpu_load > 40.0 || entropy > 0.9 {
            self.cortisol += 0.005;
        } else {
            self.cortisol -= 0.001; // Slower recovery
        }

        // CLAMPING
        self.adenosine = self.adenosine.clamp(0.0, 1.0);
        self.dopamine = self.dopamine.clamp(0.0, 1.0);
        self.cortisol = self.cortisol.clamp(0.0, 1.0);
    }
}
