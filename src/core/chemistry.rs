

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

    pub fn tick(&mut self, entropy: f32, cpu_load: f32, is_dreaming: bool, is_trauma: bool, current_neurons: usize, delta_time: f32) {
        // Normalization factor: all constants were tuned for 60Hz
        // We want (rate * delta_time) to equal (constant) when delta_time is 1/60
        let time_scale = delta_time / (1.0 / 60.0);

        // 1. ADENOSINA (Fatiga/Presión de Sueño)
        if is_dreaming {
            // Dormir limpia la fatiga - recuperación REAL
            // Was 0.05 (too fast). Now 0.0005 -> ~30s to recover full bar at 60Hz
            self.adenosine -= 0.0005 * time_scale; 
        } else {
            // Base fatigue from just being awake
            let base_fatigue = 0.0001 * time_scale;
            
            // Cognitive load from processing (high entropy = hard thinking)
            let cognitive_load = entropy * 0.0003 * time_scale;
            
            // Metabolic load from hardware stress
            let metabolic_load = (cpu_load / 100.0) * 0.0002 * time_scale;
            
            // Resilience: More neurons = slightly better endurance
            let resilience = (current_neurons as f32 / 150.0).clamp(0.5, 2.0);
            
            // Total load
            let total_load = (base_fatigue + cognitive_load + metabolic_load) / resilience;
            self.adenosine += total_load;
            
            // Trauma is exhausting (bypass resilience)
            if is_trauma {
                self.adenosine += 0.01 * time_scale;
            }
        }

        // 2. DOPAMINA (Novedad/Recompensa)
        // Decae naturalmente (Aburrimiento) - Faster decay
        self.dopamine -= 0.002 * time_scale; 
        
        // Sube con la actividad del reservorio (Solo si es intensa, indicando 'Eureka' o esfuerzo)
        if entropy > 0.7 && entropy < 0.9 {
            self.dopamine += 0.005 * time_scale;
        }

        // 3. CORTISOL (Estrés)
        // Relaxing thresholds: CPU > 80% (Hyper-focus stress) or extreme entropy
        if is_trauma || cpu_load > 85.0 || entropy > 0.95 {
            self.cortisol += 0.003 * time_scale; // Slower climb
        } else {
            self.cortisol -= 0.002 * time_scale; // Faster recovery
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
        self.adenosine < 0.3 // Deeper sleep required to wake
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
