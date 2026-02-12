
use nalgebra::{DMatrix, DVector};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)] // Clone needed for daemon state
pub struct FractalReservoir {
    pub size: usize,
    pub input_size: usize,
    pub leak_rate: f32,
    pub spectral_radius: f32,
    pub entropy: f32,
    pub last_activity: Vec<f32>,
    pub hebbian_events: u32,
    pub curiosity: f32, // Learning Rate

    // Matrices (Excluded from default serialization)
    #[serde(skip)]
    #[serde(default = "default_weights")]
    weights: DMatrix<f32>,
    
    #[serde(skip)]
    #[serde(default = "default_input_weights")]
    input_weights: DMatrix<f32>,
    
    #[serde(skip)]
    #[serde(default = "default_state")]
    state: DVector<f32>,
    
    #[serde(skip)]
    #[serde(default = "default_bias")]
    bias: DVector<f32>,
}

// Helpers for Serde Default
fn default_weights() -> DMatrix<f32> { DMatrix::zeros(1, 1) }
fn default_input_weights() -> DMatrix<f32> { DMatrix::zeros(1, 1) }
fn default_state() -> DVector<f32> { DVector::zeros(1) }
fn default_bias() -> DVector<f32> { DVector::zeros(1) }

impl FractalReservoir {
    pub fn new(size: usize, input_size: usize, spectral_radius: f32, leak_rate: f32) -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();

        // Initialize weights with sparse connectivity (Fractal seed)
        let weights = DMatrix::from_fn(size, size, |_, _| {
            if rng.gen::<f32>() < 0.1 { // 10% connectivity
                normal.sample(&mut rng) * spectral_radius
            } else {
                0.0
            }
        });

        let input_weights = DMatrix::from_fn(size, input_size, |_, _| {
             (rng.gen::<f32>() * 2.0 - 1.0) * 1.0 // Strong input coupling
        });

        let bias = DVector::from_fn(size, |_, _| rng.gen::<f32>() * 0.1 );

        Self {
            size,
            input_size,
            leak_rate,
            spectral_radius,
            entropy: 0.0,
            last_activity: vec![0.0; size],
            hebbian_events: 0,
            curiosity: 0.5,
            weights,
            input_weights,
            state: DVector::zeros(size),
            bias,
        }
    }
    
    // API Match: daemon.rs calls load(size, leak_rate)
    // It seems it wants to try loading OR create new with these params.
    pub fn load(size: usize, leak_rate: f32) -> Self {
        // Mock implementation for now to satisfy type checker. 
        // Real implementation would try to read from disk.
        // For debugging/restoration, let's just create new.
        // Fix: Input size must match reservoir size because daemon sends a vector of size `ego.current_size()`
        Self::new(size, size, 0.95, leak_rate)
    }

    pub fn set_curiosity(&mut self, curiosity: f32) {
        self.curiosity = curiosity;
    }

    // API Match: daemon.rs calls tick(&input_signal, dopamine, adenosine, cortisol, delta_time)
    pub fn tick(&mut self, input: &[f32], _dopamine: f32, adenosine: f32, cortisol: f32, _delta_time: f32) -> f32 {
        let input_vec = DVector::from_column_slice(input);
        
        // Modulate Leak Rate by Adenosine (Fatigue makes system sluggish)
        let effective_leak = self.leak_rate * (1.0 - adenosine * 0.5);
        
        // Modulate Spectral Radius by Cortisol (Stress makes system chaotic/sensitive)
        // We can't easily change spectral radius runtime without recomputing weights, 
        // but we can scale the weight matrix application.
        let stress_gain = 1.0 + (cortisol * 0.5); 
        
        // ESN State Equation: x(t+1) = (1-a)x(t) + a*tanh(gain * W*x(t) + Win*u(t) + bias)
        let pre_activation = (&self.weights * &self.state) * stress_gain + &self.input_weights * input_vec + &self.bias;
        let update = pre_activation.map(|x| x.tanh());
        
        self.state = &self.state * (1.0 - effective_leak) + update * effective_leak;
        
        // Calculate Entropy of the state vector
        self.entropy = self.calculate_entropy();
        
        // Update public activity for visualization
        self.last_activity = self.state.iter().map(|&x| (x + 1.0) / 2.0).collect(); // Norm 0-1
        
        self.entropy // Return entropy as f32
    }
    
    // NEURAL ECHO: Inject raw logits (probabilities) directly into the reservoir state
    pub fn inject_logits(&mut self, logits: &[f32]) {
        if logits.is_empty() { return; }
        
        let reservoir_size = self.current_size();
        let vocab_size = logits.len();
        
        // Downsample simple: Sumar bins
        let chunk_size = (vocab_size / reservoir_size).max(1);
        
        let mut impact_vector = DVector::zeros(reservoir_size);
        
        for i in 0..reservoir_size {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(vocab_size);
            if start >= vocab_size { break; }
            
            // Raw logits can be negative. We want "Activation".
            // Let's take variance or max of the chunk to represent "Activity" in that semantic band.
            let chunk = &logits[start..end];
            let activity = chunk.iter().fold(0.0f32, |acc, &x| acc.max(x));
            
            // Normalize activity somewhat (Logits can be 10.0 or -10.0)
            // Sigmoid-ish squash?
            impact_vector[i] = (activity * 0.1).tanh();
        }
        
        // Apply impact to state
        // x(t+1) = x(t) + impact
        self.state += impact_vector;
        
        // Clamp to avoid explosion
        self.state.apply(|x| *x = x.clamp(-1.0, 1.0));
    }

    // API Match: daemon.rs calls hebbian_update(dopamine, delta_time)
    pub fn hebbian_update(&mut self, dopamine: f32, delta_time: f32) -> u32 {
        let reinforcement = dopamine; // Use dopamine as reinforcement
        if reinforcement.abs() < 0.01 { return 0; }
        
        // Simple Hebbian: If neuron i and j active, strengthen weight w_ij
        // Delta W = alpha * reinforcement * x_i * x_j
        // We simulate this by strengthening connections between active nodes
        
        let activity_threshold = 0.5;
        let alpha = 0.01 * reinforcement * delta_time * 60.0; // Scale by time
        let mut changes = 0;

        // Iterate a subset for performance (approximated plasticity)
        let mut rng = rand::thread_rng();
        for _ in 0..(self.size * 2) {
            let i = rng.gen_range(0..self.size);
            let j = rng.gen_range(0..self.size);
            
            let xi = self.state[i];
            let xj = self.state[j];
            
            if xi.abs() > activity_threshold && xj.abs() > activity_threshold {
                // Determine sign match (Hebb: fire together, wire together)
                let sign_match = if xi.signum() == xj.signum() { 1.0 } else { -1.0 };
                let delta = alpha * xi.abs() * xj.abs() * sign_match;
                
                // Only modify existing weights (sparse topology)
                if self.weights[(i, j)].abs() > 0.001 {
                    self.weights[(i, j)] += delta;
                    // Clamp weights
                    self.weights[(i, j)] = self.weights[(i, j)].clamp(-1.5, 1.5);
                    changes += 1;
                }
            }
        }
        
        if changes > 0 {
            self.hebbian_events += changes;
        }
        changes
    }
    
    pub fn prune_inactive_neurons(&mut self) -> usize {
        // Mock pruning by zeroing weak weights
        let mut pruned = 0;
        for i in 0..self.size {
            for j in 0..self.size {
                if self.weights[(i,j)].abs() < 0.05 && self.weights[(i,j)] != 0.0 {
                    self.weights[(i,j)] = 0.0;
                    pruned += 1;
                }
            }
        }
        pruned
    }
    
    pub fn neurogenesis(&mut self, count: usize) {
        // Reinforce random connections (Mock growth)
        let mut rng = rand::thread_rng();
        for _ in 0..count {
             let i = rng.gen_range(0..self.size);
             let j = rng.gen_range(0..self.size);
             self.weights[(i,j)] += rng.gen_range(-0.1..0.1);
        }
    }

    fn calculate_entropy(&self) -> f32 {
        // Shannon entropy of activation distribution
        // Bin state into 10 bins (-1 to 1)
        let mut counts = [0usize; 10];
        for x in self.state.iter() {
            let val = (x.clamp(-1.0, 1.0) + 1.0) / 2.0; // 0-1
            let bin = (val * 9.99).floor() as usize;
            counts[bin] += 1;
        }
        
        let total = self.size as f32;
        let mut h = 0.0;
        for &count in counts.iter() {
            if count > 0 {
                let p = count as f32 / total;
                h -= p * p.log2();
            }
        }
        // Normalize max entropy ~3.32 for 10 bins
        h / 3.32
    }
    
    pub fn current_size(&self) -> usize {
        self.size
    }

    pub fn drain_hebbian_events(&mut self) -> u32 {
        let e = self.hebbian_events;
        self.hebbian_events = 0;
        e
    }
    
    pub fn get_activity_snapshot(&self) -> Vec<f32> {
        self.last_activity.clone()
    }
    
    pub fn get_state_description(&self) -> String {
        format!("Entropy: {:.2} | Active: {}%", self.entropy, self.entropy * 30.0) // Mock
    }

    // SERIALIZATION HANDLERS
    pub fn save(&self) { // Daemon calls without arguments
         // Mock save, do nothing or write to default path
         let _ = self.save_to_disk("reservoir.json");
    }
    
    pub fn save_to_disk(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        // Header
        writeln!(file, "ALEPH_RESERVOIR_V1")?;
        writeln!(file, "{} {} {} {}", self.size, self.input_size, self.spectral_radius, self.leak_rate)?;
        
        // Save weights raw
        for x in self.weights.iter() { write!(file, "{} ", x)?; }
        writeln!(file)?;
        
        Ok(())
    }
}
