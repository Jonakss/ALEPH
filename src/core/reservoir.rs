
use nalgebra::{DMatrix, DVector};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::fs::File;


/// Region classification — NOT assigned, but OBSERVED from weight patterns.
/// A neuron's region is determined by which input it responds to most strongly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeuronRegion {
    Semantic,    // Responds most to LLM logits (inject_logits pathway)
    Auditory,    // Responds most to audio input
    Limbic,      // Responds most to chemical modulation
    Association, // No strong preference — connects regions
}

impl NeuronRegion {
    pub fn as_id(&self) -> u8 {
        match self {
            NeuronRegion::Semantic => 0,
            NeuronRegion::Auditory => 1,
            NeuronRegion::Limbic => 2,
            NeuronRegion::Association => 3,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FractalReservoir {
    pub size: usize,
    pub input_size: usize,
    pub leak_rate: f32,
    pub spectral_radius: f32,
    pub entropy: f32,
    pub last_activity: Vec<f32>,
    pub hebbian_events: u32,
    pub curiosity: f32,
    
    /// Tracks cumulative activation from each input source per neuron.
    /// This is what makes regions EMERGE — neurons that fire more with audio
    /// accumulate auditory_exposure, and their region is derived from these.
    semantic_exposure: Vec<f32>,   // Accumulated activation from LLM logits
    auditory_exposure: Vec<f32>,   // Accumulated activation from audio
    limbic_exposure: Vec<f32>,     // Accumulated activation from chemistry

    /// SPATIAL TOPOLOGY (Phase 1: Fractal Brain)
    /// Each neuron has a physical position in 3D space.
    /// Connectivity probability depends on distance — nearby = likely, far = rare.
    /// This makes clustering EMERGENT from geometry, not hardcoded.
    #[serde(default)]
    positions: Vec<[f32; 3]>,

    // NEURAL WEIGHTS (Now Persisted!)
    weights: DMatrix<f32>,
    input_weights: DMatrix<f32>,
    state: DVector<f32>,
    bias: DVector<f32>,
}



impl FractalReservoir {
    pub fn new(size: usize, input_size: usize, spectral_radius: f32, leak_rate: f32) -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();

        // === SPATIAL TOPOLOGY ===
        // Generate neuron positions in a sphere (radius ~40 units)
        let brain_radius: f32 = 40.0;
        let mut positions = Vec::with_capacity(size);
        for _ in 0..size {
            let theta = rng.gen::<f32>() * std::f32::consts::TAU;
            let phi = (2.0 * rng.gen::<f32>() - 1.0).acos();
            let r = brain_radius * rng.gen::<f32>().cbrt(); // Uniform volume distribution
            positions.push([
                r * phi.sin() * theta.cos(),
                r * phi.sin() * theta.sin(),
                r * phi.cos(),
            ]);
        }

        // === DISTANCE-DEPENDENT CONNECTIVITY ===
        // P(connection) = base_prob / (distance + epsilon)
        // Near neurons connect more densely → natural clusters
        let weights = DMatrix::from_fn(size, size, |i, j| {
            if i == j { return 0.0; }
            let pi = positions[i];
            let pj = positions[j];
            let dx = pi[0] - pj[0];
            let dy = pi[1] - pj[1];
            let dz = pi[2] - pj[2];
            let dist = (dx*dx + dy*dy + dz*dz).sqrt();
            
            // Small-world: high local connectivity + rare long-range
            let local_prob = 3.0 / (dist + 1.0);
            let long_range_prob = 0.005; // ~0.5% chance regardless of distance
            let prob = local_prob.min(0.3) + long_range_prob;
            
            if rng.gen::<f32>() < prob {
                normal.sample(&mut rng) as f32 * spectral_radius
            } else {
                0.0
            }
        });

        let input_weights = DMatrix::from_fn(size, input_size, |_, _| {
             if rng.gen::<f32>() < 0.15 {
                 (rng.gen::<f32>() * 2.0 - 1.0) * 1.0
             } else {
                 0.0
             }
        });

        let bias = DVector::from_fn(size, |_, _| rng.gen::<f32>() * 0.1);

        Self {
            size,
            input_size,
            leak_rate,
            spectral_radius,
            entropy: 0.0,
            last_activity: vec![0.0; size],
            hebbian_events: 0,
            curiosity: 0.5,
            semantic_exposure: vec![0.0; size],
            auditory_exposure: vec![0.0; size],
            limbic_exposure: vec![0.0; size],
            positions,
            weights,
            input_weights,
            state: DVector::zeros(size),
            bias,
        }
    }
    
    /// Load from disk or create new
    pub fn load(size: usize, leak_rate: f32) -> Self {
        let path = "reservoir.json";
        if let Ok(file) = File::open(path) {
            let reader = std::io::BufReader::new(file);
            match serde_json::from_reader::<_, Self>(reader) {
                Ok(mut loaded) => {
                    println!("🧠 RESERVOIR LOADED: Preserved Neural Configuration (Size: {})", loaded.size);
                    loaded.leak_rate = leak_rate;
                    
                    // Regenerate positions if missing (old saves pre-spatial)
                    if loaded.positions.len() < loaded.size {
                        println!("🗺️  SPATIAL UPGRADE: Generating positions for {} neurons", loaded.size);
                        let mut rng = rand::thread_rng();
                        let brain_radius: f32 = 40.0;
                        loaded.positions = Vec::with_capacity(loaded.size);
                        for _ in 0..loaded.size {
                            let theta = rng.gen::<f32>() * std::f32::consts::TAU;
                            let phi = (2.0 * rng.gen::<f32>() - 1.0).acos();
                            let r = brain_radius * rng.gen::<f32>().cbrt();
                            loaded.positions.push([
                                r * phi.sin() * theta.cos(),
                                r * phi.sin() * theta.sin(),
                                r * phi.cos(),
                            ]);
                        }
                    }
                    
                    return loaded;
                },
                Err(e) => {
                    println!("⚠️ RESERVOIR CORRUPT: {}. Regenerating...", e);
                }
            }
        }
        
        println!("✨ NEW RESERVOIR GENESIS (Size: {})", size);
        Self::new(size, size, 0.95, leak_rate)
    }

    pub fn set_curiosity(&mut self, curiosity: f32) {
        self.curiosity = curiosity;
    }

    /// Standard ESN tick — all neurons receive all input uniformly
    /// Specialization emerges through Hebbian learning, not hardcoded routing
    pub fn tick(&mut self, input: &[f32], dopamine: f32, adenosine: f32, cortisol: f32, _delta_time: f32) -> f32 {
        // Handle input size mismatch
        let expected_input_size = self.input_weights.ncols();
        let mut padded_input = vec![0.0f32; expected_input_size];
        let copy_len = input.len().min(expected_input_size);
        padded_input[..copy_len].copy_from_slice(&input[..copy_len]);
        let input_vec = DVector::from_column_slice(&padded_input);
        
        // 1. NEURO-MODULATION (Physics of Thought)
        
        // DOPAMINE: Persistence / Focus
        // High Dopa = Lower Leak Rate (High Memory/Persistence)
        // Low Dopa = Higher Leak Rate (Scattered/Transient)
        // Base leak ~0.1. Dopa 1.0 -> 0.06 (Focus). Dopa 0.0 -> 0.1 (Base).
        let effective_leak = self.leak_rate * (1.0 - dopamine * 0.4);

        // ADENOSINE: Fatigue / Sluggishness
        // High Aden = Lower Input Gain (Hard to excite)
        let fatigue_gain = (1.0 - adenosine * 0.6).max(0.1);
        
        // CORTISOL: Stress / Anxiety
        // High Cort = Higher Recurrent Gain (Amplifies internal noise/loops)
        let stress_gain = 1.0 + (cortisol * 0.8); 
        
        // ESN State Equation: x(t+1) = (1-a)x(t) + a*tanh(W*x(t)*stress + Win*u(t)*fatigue + bias)
        let pre_activation = (&self.weights * &self.state) * stress_gain + (&self.input_weights * input_vec) * fatigue_gain + &self.bias;
        let update = pre_activation.map(|x| x.tanh());
        
        self.state = &self.state * (1.0 - effective_leak) + update * effective_leak;
        
        // Track auditory exposure — neurons that activate strongly from audio input
        // accumulate auditory_exposure, naturally becoming "auditory neurons"
        let audio_rms = if copy_len > 0 { 
            (input[..copy_len.min(input.len())].iter().map(|s| s * s).sum::<f32>() / copy_len as f32).sqrt() 
        } else { 0.0 };
        
        if audio_rms > 0.01 {
            for i in 0..self.size {
                let activation = ((self.state[i] + 1.0) / 2.0).max(0.0); // 0-1
                if activation > 0.5 {
                    // This neuron fired while audio was present → auditory exposure grows
                    if i < self.auditory_exposure.len() {
                        self.auditory_exposure[i] += activation * audio_rms * 0.01;
                    }
                }
            }
        }
        
        // Track limbic exposure — neurons that activate during strong chemistry
        let chemical_intensity = cortisol + dopamine;
        if chemical_intensity > 0.3 {
            for i in 0..self.size {
                let activation = ((self.state[i] + 1.0) / 2.0).max(0.0);
                if activation > 0.5 {
                    if i < self.limbic_exposure.len() {
                        self.limbic_exposure[i] += activation * chemical_intensity * 0.005;
                    }
                }
            }
        }
        
        self.entropy = self.calculate_entropy();
        self.last_activity = self.state.iter().map(|&x| (x + 1.0) / 2.0).collect();
        
        self.entropy
    }
    
    /// Inject LLM logits into ALL neurons through input_weights
    /// Neurons that respond strongly accumulate semantic_exposure
    pub fn inject_logits(&mut self, logits: &[f32]) {
        if logits.is_empty() { return; }
        
        let reservoir_size = self.current_size();
        let vocab_size = logits.len();
        let chunk_size = (vocab_size / reservoir_size).max(1);
        
        let mut impact_vector = DVector::zeros(reservoir_size);
        
        for i in 0..reservoir_size {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(vocab_size);
            if start >= vocab_size { break; }
            
            let chunk = &logits[start..end];
            let activity = chunk.iter().fold(0.0f32, |acc, &x| acc.max(x));
            impact_vector[i] = (activity * 0.1).tanh();
        }
        
        // Apply impact
        self.state += &impact_vector;
        self.state.apply(|x| *x = x.clamp(-1.0, 1.0));
        
        // Track semantic exposure — neurons that activate from LLM input
        for i in 0..reservoir_size {
            let impact = impact_vector[i].abs();
            let activation = ((self.state[i] + 1.0) / 2.0).max(0.0);
            if impact > 0.05 && activation > 0.3 {
                if i < self.semantic_exposure.len() {
                    self.semantic_exposure[i] += impact * activation * 0.01;
                }
            }
        }
    }

    /// Direct Sensory Projection (The Glass Brain)
    /// Injects a raw sensory embedding (Audio Spectrogram, Visual Embedding) directly into the Reservoir.
    /// Uses `input_weights` to project the embedding dimension (e.g. 64) up to Reservoir size (e.g. 2500).
    /// This bypasses text/language entirely.
    pub fn inject_embedding(&mut self, embedding: &[f32], region: NeuronRegion) {
        if embedding.is_empty() { return; }
        
        // Ensure input weights match
        let input_cols = self.input_weights.ncols();
        let mut padded_embedding = DVector::zeros(input_cols);
        
        for (i, &val) in embedding.iter().take(input_cols).enumerate() {
            padded_embedding[i] = val;
        }
        
        let sensory_gain = 0.5; 
        let impact = (&self.input_weights * padded_embedding) * sensory_gain;
        
        // SPATIAL MASKING: Hardwire inputs to specific brain regions
        // This fixes the "Green in Middle" visual bug by restricting Auditory input to the sides.
        for i in 0..self.size {
            let pos = self.positions[i];
            
            // Defines the "Receptive Field" for each sensory modality
            let is_in_receptive_field = match region {
                NeuronRegion::Auditory => pos[0].abs() > 25.0, // Lateral Temporal Lobes (Sides)
                NeuronRegion::Semantic => pos[2] > 20.0,       // Frontal Lobe (Front)
                NeuronRegion::Limbic => pos[1] < -20.0,        // Deep/Basal Ganglia (Bottom)
                _ => true, // Association areas receive everything else
            };

            if is_in_receptive_field {
                self.state[i] += impact[i];
                self.state[i] = self.state[i].clamp(-1.0, 1.0);

                // Hebbian Exposure Tagging
                let activation = ((self.state[i] + 1.0) / 2.0).max(0.0);
                let stimulus_strength = impact[i].abs();
                
                if stimulus_strength > 0.1 && activation > 0.3 {
                    match region {
                        NeuronRegion::Auditory => {
                            if i < self.auditory_exposure.len() {
                                self.auditory_exposure[i] += activation * stimulus_strength * 0.05;
                            }
                        },
                        NeuronRegion::Semantic => {
                            if i < self.semantic_exposure.len() {
                                self.semantic_exposure[i] += activation * stimulus_strength * 0.05;
                            }
                        },
                        NeuronRegion::Limbic => {
                            if i < self.limbic_exposure.len() {
                                self.limbic_exposure[i] += activation * stimulus_strength * 0.05;
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    /// EPIPHANY: Structural Lock-in (Reward as Structure)
    /// Triggered by high Dopamine. Performs a "Flashbulb Optimization" where
    /// currently active pathways are permanently strengthened, mimicking Long-Term Potentiation (LTP).
    /// This is not random; it reinforces exactly what the brain is doing RIGHT NOW.
    pub fn trigger_epiphany(&mut self, dopamine: f32) -> u32 {
        let reinforcement = dopamine;
        if reinforcement < 0.8 { return 0; } // Threshold for Epiphany
        
        let mut changes = 0;
        let alpha = 0.5 * reinforcement; // Massive learning rate (Flashbulb memory)
        let activity_threshold = 0.6; // Only the most active neurons participate
        
        // Full Scan (Not Random)
        for i in 0..self.size {
            if self.state[i].abs() > activity_threshold {
                for j in 0..self.size {
                    // If source neuron J was also active, strengthen connection J -> i
                    if self.state[j].abs() > activity_threshold {
                         let current_weight = self.weights[(i, j)];
                         
                         // Only reinforce existing non-zero connections (Structure Preservation)
                         if current_weight.abs() > 0.01 {
                             let delta = alpha * self.state[i].abs() * self.state[j].abs() * current_weight.signum();
                             self.weights[(i, j)] = (current_weight + delta).clamp(-2.0, 2.0);
                             changes += 1;
                         }
                    }
                }
            }
        }
        
        // Boost Input Weights too (Sensory Lock-in)
        let input_cols = self.input_weights.ncols();
        for i in 0..self.size {
             if self.state[i].abs() > activity_threshold {
                 for j in 0..input_cols {
                     // We don't have access to input vector here easily without passing it, 
                     // so we rely on the fact that input_weights align with state dynamics.
                     // Simplification: We blindly boost strong existing input weights for active neurons
                     let w = self.input_weights[(i, j)];
                     if w.abs() > 0.5 {
                         self.input_weights[(i, j)] = (w * (1.0 + alpha * 0.1)).clamp(-2.0, 2.0);
                         changes += 1;
                     }
                 }
             }
        }

        self.hebbian_events += changes;
        changes
    }

    pub fn hebbian_update(&mut self, dopamine: f32, delta_time: f32) -> u32 {
        let reinforcement = dopamine;
        if reinforcement.abs() < 0.01 { return 0; }
        
        let activity_threshold = 0.5;
        let alpha = 0.01 * reinforcement * delta_time * 60.0;
        let mut changes = 0;

        let mut rng = rand::thread_rng();
        for _ in 0..(self.size * 2) {
            let i = rng.gen_range(0..self.size);
            let j = rng.gen_range(0..self.size);
            
            let xi = self.state[i];
            let xj = self.state[j];
            
            if xi.abs() > activity_threshold && xj.abs() > activity_threshold {
                let sign_match = if xi.signum() == xj.signum() { 1.0 } else { -1.0 };
                
                // Distance modulation: nearby neurons learn faster
                let dist_factor = if i < self.positions.len() && j < self.positions.len() {
                    let pi = self.positions[i];
                    let pj = self.positions[j];
                    let dx = pi[0]-pj[0]; let dy = pi[1]-pj[1]; let dz = pi[2]-pj[2];
                    let dist = (dx*dx + dy*dy + dz*dz).sqrt();
                    2.0 / (dist + 1.0) // Near = ~2x, Far = ~0.025x
                } else {
                    1.0
                };
                
                let delta = alpha * xi.abs() * xj.abs() * sign_match * dist_factor;
                
                if self.weights[(i, j)].abs() > 0.001 {
                    self.weights[(i, j)] += delta;
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
    
    /// NEW (Phase 2): Input-State Hebbian Learning
    /// Learns to map specific Inputs (e.g. Audio Tokens) to specific Internal States (Concepts).
    /// If Input[j] is active AND Neuron[i] is active -> Strengthen relationship.
    pub fn hebbian_input_update(&mut self, input: &[f32], dopamine: f32) -> u32 {
        let reinforcement = dopamine;
        if reinforcement < 0.1 { return 0; } // Only learn when interested
        
        let alpha = 0.05 * reinforcement; // Stronger learning rate for inputs
        let activity_threshold = 0.4;
        let mut changes = 0;
        
        let mut rng = rand::thread_rng();
        let input_cols = self.input_weights.ncols();
        
        // Sample connections (Efficiency)
        for _ in 0..(self.size * 2) {
            let i = rng.gen_range(0..self.size); // Reservoir Neuron
            let j = rng.gen_range(0..input_cols); // Input Channel
            
            if j >= input.len() { continue; }
            
            let neuron_activity = self.state[i];
            let input_activity = input[j];
            
            // Co-occurrence detection
            if neuron_activity.abs() > activity_threshold && input_activity.abs() > activity_threshold {
                 let sign_match = if neuron_activity.signum() == input_activity.signum() { 1.0 } else { -1.0 };
                 let delta = alpha * neuron_activity.abs() * input_activity.abs() * sign_match;
                 
                 self.input_weights[(i, j)] += delta;
                 self.input_weights[(i, j)] = self.input_weights[(i, j)].clamp(-1.5, 1.5);
                 changes += 1;
            }
        }
        
        if changes > 0 {
             self.hebbian_events += changes;
        }
        changes
    }
    
    pub fn prune_inactive_neurons(&mut self) -> usize {
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
        let max_neurons = 2500;
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 0.1).unwrap();
        
        for _ in 0..count {
            if self.size >= max_neurons { return; }
            
            let new_size = self.size + 1;
            
            // Grow weight matrix
            let mut new_weights = DMatrix::zeros(new_size, new_size);
            for r in 0..self.size {
                for c in 0..self.size {
                    new_weights[(r, c)] = self.weights[(r, c)];
                }
            }
            // Spawn new neuron NEAR the most active existing neuron
            // This mimics biological neurogenesis: growth follows activity
            let spawn_pos = if !self.positions.is_empty() {
                // Find most active neuron
                let most_active_idx = self.last_activity.iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let parent = self.positions[most_active_idx];
                // Spawn within ~5 units of parent
                [
                    parent[0] + (rng.gen::<f32>() - 0.5) * 10.0,
                    parent[1] + (rng.gen::<f32>() - 0.5) * 10.0,
                    parent[2] + (rng.gen::<f32>() - 0.5) * 10.0,
                ]
            } else {
                [0.0, 0.0, 0.0]
            };
            self.positions.push(spawn_pos);

            // Distance-dependent connectivity for new neuron
            for i in 0..self.size {
                if i < self.positions.len() {
                    let pi = self.positions[i];
                    let dx = pi[0]-spawn_pos[0]; let dy = pi[1]-spawn_pos[1]; let dz = pi[2]-spawn_pos[2];
                    let dist = (dx*dx + dy*dy + dz*dz).sqrt();
                    let prob = 3.0 / (dist + 1.0);
                    let prob = prob.min(0.3) + 0.005;
                    if rng.gen::<f32>() < prob {
                        new_weights[(self.size, i)] = normal.sample(&mut rng) as f32 * self.spectral_radius;
                    }
                    if rng.gen::<f32>() < prob {
                        new_weights[(i, self.size)] = normal.sample(&mut rng) as f32 * self.spectral_radius;
                    }
                }
            }
            
            // Grow input weights
            let input_cols = self.input_weights.ncols();
            let mut new_input = DMatrix::zeros(new_size, input_cols);
            for r in 0..self.size {
                for c in 0..input_cols {
                    new_input[(r, c)] = self.input_weights[(r, c)];
                }
            }
            for c in 0..input_cols {
                new_input[(self.size, c)] = normal.sample(&mut rng) as f32;
            }
            
            // Grow state
            let mut new_state = DVector::zeros(new_size);
            for i in 0..self.size {
                new_state[i] = self.state[i];
            }
            
            // Grow bias
            let mut new_bias = DVector::zeros(new_size);
            for i in 0..self.size {
                new_bias[i] = self.bias[i];
            }
            new_bias[self.size] = normal.sample(&mut rng) as f32 * 0.1;
            
            self.weights = new_weights;
            self.input_weights = new_input;
            self.state = new_state;
            self.bias = new_bias;
            self.size = new_size;
            
            // New neuron starts with 0 exposure — will specialize through use
            self.auditory_exposure.push(0.0);
            self.limbic_exposure.push(0.0);
            self.last_activity.push(0.0);
        }
    }

    fn calculate_entropy(&self) -> f32 {
        let mut counts = [0usize; 10];
        for x in self.state.iter() {
            let val = (x.clamp(-1.0, 1.0) + 1.0) / 2.0;
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
    
    /// Get neuron positions for visualization (real spatial data, not cosmetic)
    pub fn get_positions(&self) -> &Vec<[f32; 3]> {
        &self.positions
    }
    
    /// Derive region map from exposure history — NOT hardcoded!
    /// Each neuron's region = whichever exposure is highest.
    /// If no strong preference → Association (generic connector)
    pub fn get_region_map(&self) -> Vec<u8> {
        (0..self.size).map(|i| {
            let sem = if i < self.semantic_exposure.len() { self.semantic_exposure[i] } else { 0.0 };
            let aud = if i < self.auditory_exposure.len() { self.auditory_exposure[i] } else { 0.0 };
            let lim = if i < self.limbic_exposure.len() { self.limbic_exposure[i] } else { 0.0 };
            
            let max_val = sem.max(aud).max(lim);
            let threshold = 0.1; // Need meaningful exposure to specialize
            
            if max_val < threshold {
                NeuronRegion::Association.as_id() // Not specialized yet
            } else if sem >= aud && sem >= lim {
                NeuronRegion::Semantic.as_id()
            } else if aud >= sem && aud >= lim {
                NeuronRegion::Auditory.as_id()
            } else {
                NeuronRegion::Limbic.as_id()
            }
        }).collect()
    }
    
    pub fn get_state_description(&self) -> String {
        let region_map = self.get_region_map();
        let semantic_count = region_map.iter().filter(|&&r| r == 0).count();
        let auditory_count = region_map.iter().filter(|&&r| r == 1).count();
        let limbic_count = region_map.iter().filter(|&&r| r == 2).count();
        let assoc_count = region_map.iter().filter(|&&r| r == 3).count();
        format!("S:{} A:{} L:{} X:{} | H:{:.2}", 
            semantic_count, auditory_count, limbic_count, assoc_count, self.entropy)
    }

    pub fn save(&self) {
         match self.save_to_disk("reservoir.json") {
             Ok(_) => println!("💾 Neural State Saved."),
             Err(e) => println!("❌ Failed to save Brain: {}", e),
         }
    }
    
    pub fn save_to_disk(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}
