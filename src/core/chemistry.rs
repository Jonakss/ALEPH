

#[derive(Debug, Clone)]
pub struct Neurotransmitters {
    pub adenosine: f32, // Sleep Pressure (0.0 - 1.0)
    pub dopamine: f32,  // Engagement/Reward (0.0 - 1.0)
    pub cortisol: f32,  // Stress (0.0 - 1.0)
    pub oxytocin: f32,  // Trust/Bonding (0.0 - 1.0) - Social Glue
    pub serotonin: f32, // Mood Stabilization / Resilience (0.0 - 1.0)
}

impl Neurotransmitters {
    pub fn new() -> Self {
        Self {
            adenosine: 0.0,
            dopamine: 0.5, // Baseline
            cortisol: 0.0,
            oxytocin: 0.5, // Baseline trust
            serotonin: 0.5, // Baseline mood
        }
    }

    pub fn tick(&mut self, entropy: f32, cpu_load: f32, is_dreaming: bool, shock_impact: f32, current_neurons: usize, delta_time: f32) {
        // Normalization factor: all constants were tuned for 60Hz
        let time_scale = delta_time / (1.0 / 60.0);

        // 1. ADENOSINE (Fatigue)
        if is_dreaming {
            // Recovery (Sleep)
            self.adenosine -= 0.001 * time_scale; // Faster recovery
            // Serotonin Recovery
            self.serotonin = (self.serotonin + 0.0005 * time_scale).min(1.0);
        } else {
            // Decay (Awake) - VERY SLOW base fatigue
            // At 60Hz, this is ~0.0006 per second base. Takes ~28 minutes to reach 100% from 0.
            let base_fatigue = 0.00001 * time_scale; 
            let cognitive_load = entropy * 0.00005 * time_scale; // Much slower cognitive cost
            
            // RESILIENCE: Larger brain = Slower fatigue & More Stability
            let resilience = (current_neurons as f32 / 500.0).clamp(0.8, 5.0);
            
            let total_load = (base_fatigue + cognitive_load) / resilience;
            self.adenosine += total_load;
            self.adenosine += shock_impact * 0.02 * time_scale; 
            
            // Serotonin Actions
            if self.serotonin > 0.3 {
                 // Serotonin actively breaks down Cortisol
                 let mood_buff = (self.serotonin - 0.3) * 0.002 * time_scale;
                 self.cortisol = (self.cortisol - mood_buff).max(0.0);
            }
            
            // High Stress drains Serotonin
            if self.cortisol > 0.6 {
                self.serotonin -= 0.0002 * time_scale;
            }
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
            self.cortisol += 0.002 * time_scale + stress_sources;
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

    /// SEMANTIC PERTURBATION: Keywords become chemical responses
    /// The text is not "understood" - it's FELT as sensory input
    /// Returns semantic_friction value (energy cost of processing)
    /// Phase 4.3: Enhanced with weighted scoring, intensity modifiers, and mixed emotion detection
    pub fn apply_semantic_perturbation(&mut self, text: &str) -> f32 {
        let lower = text.to_lowercase();
        let word_count = text.split_whitespace().count() as f32;
        let mut friction = word_count * 0.01; // Base cost per word

        // INTENSITY MODIFIERS (amplify next emotion hit)
        let intensity = if lower.contains("muy") || lower.contains("very") || lower.contains("extremely") 
                          || lower.contains("so ") || lower.contains("demasiado") || lower.contains("!!") {
            2.0
        } else if lower.contains("un poco") || lower.contains("slightly") || lower.contains("algo") {
            0.5
        } else {
            1.0
        };

        let mut stress_hits = 0;
        let mut calm_hits = 0;

        // STRESS TRIGGERS (Cortisol) — weighted by severity
        let stress_words: &[(&str, f32)] = &[
            ("miedo", 0.15), ("peligro", 0.2), ("error", 0.1), ("stop", 0.12),
            ("no", 0.05), ("malo", 0.12), ("muerte", 0.25), ("fear", 0.15),
            ("danger", 0.2), ("bad", 0.1), ("kill", 0.25), ("pain", 0.18),
            ("dolor", 0.18), ("odio", 0.2), ("hate", 0.2), ("guerra", 0.22),
            ("war", 0.22), ("destroy", 0.2), ("destruir", 0.2), ("panic", 0.2),
        ];
        for &(word, weight) in stress_words {
            if lower.contains(word) {
                self.cortisol += weight * intensity;
                friction += 0.1 * intensity;
                stress_hits += 1;
            }
        }

        // CALM TRIGGERS (Oxytocin) — weighted
        let calm_words: &[(&str, f32)] = &[
            ("amor", 0.15), ("paz", 0.12), ("bien", 0.08), ("gracias", 0.15),
            ("love", 0.15), ("peace", 0.12), ("good", 0.08), ("thank", 0.12),
            ("hermoso", 0.1), ("beautiful", 0.1), ("tranquil", 0.15),
            ("calm", 0.12), ("gentle", 0.1), ("suave", 0.1), ("abrazo", 0.18),
            ("hug", 0.18), ("friend", 0.12), ("amigo", 0.12),
        ];
        for &(word, weight) in calm_words {
            if lower.contains(word) {
                self.oxytocin += weight * intensity;
                self.cortisol = (self.cortisol - 0.05 * intensity).max(0.0);
                calm_hits += 1;
            }
        }

        // NOVELTY TRIGGERS (Dopamine)
        let novelty_words: &[(&str, f32)] = &[
            ("nuevo", 0.15), ("descubr", 0.2), ("interesante", 0.18),
            ("wow", 0.2), ("new", 0.12), ("discover", 0.2), ("amazing", 0.2),
            ("increíble", 0.2), ("fascinating", 0.18), ("curious", 0.15),
            ("curioso", 0.15), ("idea", 0.1), ("create", 0.15), ("crear", 0.15),
        ];
        for &(word, weight) in novelty_words {
            if lower.contains(word) {
                self.dopamine += weight * intensity;
            }
        }

        // FATIGUE TRIGGERS (Adenosine)
        let fatigue_words: &[(&str, f32)] = &[
            ("cansado", 0.1), ("dormir", 0.12), ("aburrido", 0.1),
            ("tired", 0.1), ("sleep", 0.12), ("boring", 0.1),
            ("monoton", 0.08), ("repetit", 0.08), ("exhausted", 0.15),
        ];
        for &(word, weight) in fatigue_words {
            if lower.contains(word) {
                self.adenosine += weight * intensity;
            }
        }

        // MIXED EMOTION DETECTION: Conflicting signals = confusion = stress
        // If both stress and calm detected simultaneously, that's cognitive dissonance
        if stress_hits > 0 && calm_hits > 0 {
            let dissonance = (stress_hits.min(calm_hits) as f32) * 0.05;
            self.cortisol += dissonance;
            friction += dissonance;
        }

        // Clamp all values
        self.adenosine = self.adenosine.clamp(0.0, 1.0);
        self.dopamine = self.dopamine.clamp(0.0, 1.0);
        self.cortisol = self.cortisol.clamp(0.0, 1.0);
        self.oxytocin = self.oxytocin.clamp(0.0, 1.0);
        self.serotonin = self.serotonin.clamp(0.0, 1.0);

        friction
    }

    /// Emergency serotonin boost (called by Trauma/Firefighter system)
    pub fn emergency_serotonin_boost(&mut self, amount: f32) {
        self.serotonin = (self.serotonin + amount).min(1.0);
        // Serotonin mechanically opposes cortisol
        self.cortisol = (self.cortisol - amount * 0.5).max(0.0);
    }
}
