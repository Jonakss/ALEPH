

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
        // 1. ADENOSINA (Fatiga/Presión de Sueño)
        // Mechanical Honesty: Adenosine is the COST of consciousness
        if is_dreaming {
            // Dormir limpia la fatiga - recuperación REAL
            self.adenosine -= 0.01; // Faster recovery during sleep
        } else {
            // Estar despierto cansa - pero puedes "empujarlo"
            // Like humans pushing through tiredness
            
            // Base fatigue from just being awake
            let base_fatigue = 0.0001;
            
            // Cognitive load from processing (high entropy = hard thinking)
            let cognitive_load = entropy * 0.0003;
            
            // Metabolic load from hardware stress
            let metabolic_load = (cpu_load / 100.0) * 0.0002;
            
            // Resilience: More neurons = slightly better endurance
            let resilience = (current_neurons as f32 / 150.0).clamp(0.5, 2.0);
            
            // Total load
            let total_load = (base_fatigue + cognitive_load + metabolic_load) / resilience;
            self.adenosine += total_load;
            
            // Trauma is exhausting (bypass resilience)
            if is_trauma {
                self.adenosine += 0.01;
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
    
    /// Set adenosine base level from unprocessed memory count (MECHANICAL HONESTY)
    /// This REPLACES the memory component, not adds to it
    pub fn set_memory_pressure(&mut self, unprocessed_ratio: f32) {
        // Memory pressure creates a "floor" of adenosine
        // This is the baseline fatigue from carrying unprocessed experiences
        // The rest comes from tick() (CPU, entropy, trauma)
        
        // Memory contributes 0-50% of adenosine as a floor
        let memory_base = (unprocessed_ratio * 0.5).clamp(0.0, 0.5);
        
        // If current adenosine is below memory floor, raise it
        // If it's above (from other sources), don't lower it
        if self.adenosine < memory_base {
            self.adenosine = memory_base;
        }
    }
    
    /// Check if body is at breaking point (forced sleep)
    pub fn is_body_failing(&self) -> bool {
        // At 95%+ adenosine, the body CANNOT continue
        self.adenosine > 0.95
    }

    /// Check if recovered enough to wake (HYSTERESIS - evita bucle colapso/despertar)
    /// Solo despertar cuando adenosina bajó significativamente, no en el borde del umbral
    pub fn is_recovered_to_wake(&self) -> bool {
        self.adenosine < 0.6
    }
    
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
