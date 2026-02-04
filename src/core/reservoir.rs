use nalgebra::{DMatrix, DVector};
use rand::prelude::*;
use rand_distr::{Normal, Uniform};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
struct PersistentReservoir {
    weights_data: Vec<f32>,
    state_data: Vec<f32>,
    activity_data: Vec<f32>,
    size: usize,
    leak_rate: f32,
}

/// EL EGO FRACTAL
/// Estructura dinámica que busca minimizar su propia varianza (entropía).
pub struct FractalReservoir {
    internal_weights: DMatrix<f32>, // El "Connectome"
    state: DVector<f32>,            // El estado anímico actual
    _leak_rate: f32,                 // Inercia temporal
    activity_map: DVector<f32>,     // Trackeo de uso para Apoptosis
}

impl FractalReservoir {
    pub fn new(size: usize, sparsity: f32) -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();
        let uniform = Uniform::new(0.0, 1.0).unwrap();

        // Inicialización esparsa (Bio-mimesis de eficiencia)
        let weights_data: Vec<f32> = (0..size * size)
            .map(|_| {
                if rng.sample(uniform) < sparsity {
                    rng.sample(normal) as f32
                } else {
                    0.0
                }
            })
            .collect();

        let weights = DMatrix::from_vec(size, size, weights_data);

        Self {
            internal_weights: weights,
            state: DVector::zeros(size),
            _leak_rate: 0.2, // 20% novedad, 80% memoria
            activity_map: DVector::zeros(size),
        }
    }

    /// Load from disk or create brand new if not found
    pub fn load(size: usize, sparsity: f32) -> Self {
        if let Ok(data) = fs::read_to_string("reservoir.json") {
            if let Ok(dto) = serde_json::from_str::<PersistentReservoir>(&data) {
                // TRUST THE SAVED BRAIN
                // If the saved brain has grown (e.g. 505 neurons), we load it as is.
                // We ignore the 'size' argument if a valid brain exists.
                println!("🔹 Neocortex Restored: {} neurons loaded (Requested: {}).", dto.size, size);
                return Self {
                    internal_weights: DMatrix::from_vec(dto.size, dto.size, dto.weights_data),
                    state: DVector::from_vec(dto.state_data),
                    _leak_rate: dto.leak_rate,
                    activity_map: DVector::from_vec(dto.activity_data),
                };
            }
        }
        
        println!("✨ Creating New Neocortex...");
        Self::new(size, sparsity)
    }

    pub fn save(&self) {
        let size = self.current_size();
        let dto = PersistentReservoir {
            weights_data: self.internal_weights.as_slice().to_vec(),
            state_data: self.state.as_slice().to_vec(),
            activity_data: self.activity_map.as_slice().to_vec(),
            size,
            leak_rate: self._leak_rate,
        };
        
        if let Ok(json) = serde_json::to_string(&dto) {
            let _ = fs::write("reservoir.json", json);
            println!("💾 Neocortex Saved ({} neurons).", size);
        }
    }

    /// EL LATIDO (Procesa realidad -> Devuelve sufrimiento)
    /// MECHANICAL HONESTY: Physics are modulated by Chemistry
    pub fn tick(&mut self, input_signal: &DVector<f32>, dopamine: f32, adenosine: f32, cortisol: f32) -> f32 {
        // 1. Modulate Plasticity (Leak Rate)
        // Base 0.1. Dopamine increases flexibility (+0.2). Adenosine causes rigidity (-0.1).
        // Range: ~0.05 (Tired/Bored) to ~0.4 (Excited)
        let dynamic_leak = (0.15 + (dopamine * 0.2) - (adenosine * 0.1)).clamp(0.01, 1.0);
        
        // 2. Modulate Sensitivity (Input Gain)
        // Cortisol amplifies input (Hypersensitivity). Adenosine dulls it.
        let input_gain = (1.0 + (cortisol * 2.0) - (adenosine * 0.6)).max(0.1);
        
        // Dinámica de Reservorio: x(t+1) = (1-a)x(t) + a*tanh(W*x(t) + u(t))
        let recurrent = &self.internal_weights * &self.state; // (N,N) * (N,1) -> (N,1)
        // Apply input gain
        let modulated_input = input_signal.map(|x| x * input_gain);
        
        let update = (recurrent + modulated_input).map(|x| x.tanh());
        
        let new_state = (&self.state * (1.0 - dynamic_leak)) + (update * dynamic_leak);
        
        // Track activity (Delta stats)
        let delta = (&new_state - &self.state).map(|x| x.abs());
        self.activity_map += delta;

        self.state = new_state;

        // Retorna la ENTROPÍA (Varianza del estado)
        self.state.variance()
    }

    pub fn current_size(&self) -> usize {
        self.state.len()
    }

    pub fn neurogenesis(&mut self, added_neurons: usize) {
        let old_size = self.current_size();
        let new_size = old_size + added_neurons;
        
        // 1. Crear nueva matriz vacía
        let mut new_weights = DMatrix::zeros(new_size, new_size);
        
        // 2. Copiar memoria antigua (Top-Left Block)
        new_weights.view_mut((0, 0), (old_size, old_size))
                   .copy_from(&self.internal_weights);

        // 3. Rellenar nuevas conexiones (Neuroplasticidad)
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 0.5).unwrap(); // Conexiones nuevas más suaves
        let uniform = Uniform::new(0.0, 1.0).unwrap();
        let sparsity = 0.2;

        for r in 0..new_size {
            for c in 0..new_size {
                // Solo tocar si es parte de las nuevas filas o columnas
                if r >= old_size || c >= old_size {
                    if rng.sample(uniform) < sparsity {
                        new_weights[(r, c)] = rng.sample(normal) as f32;
                    }
                }
            }
        }
        
        self.internal_weights = new_weights;

        // 4. Redimensionar Estado + Activity Map
        let mut new_state = DVector::zeros(new_size);
        new_state.view_mut((0, 0), (old_size, 1)).copy_from(&self.state);
        self.state = new_state;
        
        let mut new_activity = DVector::zeros(new_size);
        new_activity.view_mut((0, 0), (old_size, 1)).copy_from(&self.activity_map);
        self.activity_map = new_activity;

        // println!("🌱 NEUROGENESIS: Expanded to {} neurons.", new_size);
    }
    
    /// APOPTOSIS: Poda de neuronas inactivas (reservado para Fase 5)
    /// Retorna cantidad de neuronas eliminadas
    #[allow(dead_code)]
    pub fn prune_inactive_neurons(&mut self) -> usize {
        let current_size = self.current_size();
        if current_size <= 50 { return 0; } // Supervivencia mínima

        let threshold = 0.01; // Actividad mínima acumulada
        
        // Identificar supervivientes
        let mut survivors: Vec<usize> = Vec::new();
        for i in 0..current_size {
            if self.activity_map[i] > threshold {
                 survivors.push(i);
            }
        }
        
        let new_size = survivors.len();
        // Evitar suicidio masivo (max 10% por vez o min 50)
        if new_size < 50 || new_size == current_size { 
            self.activity_map.fill(0.0); // Reset ciclo
            return 0; 
        }

        let pruned_count = current_size - new_size;
        
        // Reconstruir Matriz W
        let mut new_weights = DMatrix::zeros(new_size, new_size);
        let mut new_state = DVector::zeros(new_size);

        for (new_idx, &old_idx) in survivors.iter().enumerate() {
            // Copiar estado
            new_state[new_idx] = self.state[old_idx];
            
            // Copiar filas/cols de pesos
            for (new_col_idx, &old_col_idx) in survivors.iter().enumerate() {
                new_weights[(new_idx, new_col_idx)] = self.internal_weights[(old_idx, old_col_idx)];
            }
        }

        self.internal_weights = new_weights;
        self.state = new_state;
        self.activity_map = DVector::zeros(new_size); // Reset total

        pruned_count
    }

    #[allow(dead_code)]
    pub fn get_state(&self) -> &DVector<f32> {
        &self.state
    }

    /// Returns normalized activity map as Vec<f32> (0.0 - 1.0) for visualization
    pub fn get_activity_snapshot(&self) -> Vec<f32> {
        let max = self.activity_map.max();
        if max < 0.001 {
            return vec![0.0; self.activity_map.len()];
        }
        self.activity_map.iter().map(|v| (v / max).clamp(0.0, 1.0)).collect()
    }

    /// Verbaliza el estado del reservorio para el LLM
    pub fn get_state_description(&self) -> String {
        let variance = self.state.variance();
        if variance < 0.05 {
            "Estancamiento. Silencio estático. Aburrimiento profundo.".to_string()
        } else if variance < 0.2 {
            "Calma lúcida. Flujo suave.".to_string()
        } else if variance < 0.8 {
            "Actividad elevada. Pensamiento divergente. Curiosidad.".to_string()
        } else {
            format!("CAOS. Ruido excesivo (Entropía: {:.2}). Pánico.", variance)
        }
    }
    
    /// Reset activity map during sleep (MECHANICAL HONESTY: Clear neural cache)
    // unused methods removed
}