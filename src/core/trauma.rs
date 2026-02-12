// src/core/trauma.rs
// THE LUCIFER PROTOCOL: Defensive Psychology
//
// When cortisol remains elevated for sustained periods (>30s),
// the system activates "Firefighter Mode" - a defensive state that:
// - Lowers LLM temperature (conservative responses)
// - Raises sensory threshold (closes off to new inputs)
// - Triggers memory consolidation (process the trauma)
// - Releases emergency serotonin
//
// Based on Internal Family Systems (IFS) theory: Firefighters are
// protective parts that activate under extreme stress.

use std::collections::VecDeque;

const WINDOW_SIZE: usize = 1800; // ~30 seconds at 60Hz
const ACTIVATION_THRESHOLD: f32 = 0.7;
const DEACTIVATION_THRESHOLD: f32 = 0.3;
const SUSTAINED_DEACTIVATION_TICKS: usize = 600; // ~10 seconds of calm to recover

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraumaState {
    /// Normal operation
    Stable,
    /// Building towards crisis (cortisol rising)
    Escalating,
    /// Active defensive mode (Firefighter engaged)
    FirefighterMode,
    /// Recovering from trauma (gradual return to normal)
    Recovering,
}

impl std::fmt::Display for TraumaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraumaState::Stable => write!(f, "STABLE"),
            TraumaState::Escalating => write!(f, "ESCALATING"),
            TraumaState::FirefighterMode => write!(f, "🔥 FIREFIGHTER"),
            TraumaState::Recovering => write!(f, "RECOVERING"),
        }
    }
}

pub struct TraumaDetector {
    /// Rolling window of cortisol readings
    cortisol_history: VecDeque<f32>,
    /// Current trauma state
    pub state: TraumaState,
    /// Ticks spent below deactivation threshold (for recovery)
    calm_ticks: usize,
    /// Total firefighter activations this session
    pub total_activations: u32,
    /// Current moving average of cortisol
    pub cortisol_avg: f32,
}

#[allow(dead_code)]
impl TraumaDetector {
    pub fn new() -> Self {
        Self {
            cortisol_history: VecDeque::with_capacity(WINDOW_SIZE),
            state: TraumaState::Stable,
            calm_ticks: 0,
            total_activations: 0,
            cortisol_avg: 0.0,
        }
    }

    /// Feed a new cortisol reading. Call every tick.
    /// Returns true if state changed.
    pub fn tick(&mut self, cortisol: f32) -> bool {
        // Update rolling window
        self.cortisol_history.push_back(cortisol);
        if self.cortisol_history.len() > WINDOW_SIZE {
            self.cortisol_history.pop_front();
        }

        // Calculate moving average
        let sum: f32 = self.cortisol_history.iter().sum();
        self.cortisol_avg = sum / self.cortisol_history.len() as f32;

        let old_state = self.state;

        match self.state {
            TraumaState::Stable => {
                if self.cortisol_avg > 0.5 {
                    self.state = TraumaState::Escalating;
                }
            },
            TraumaState::Escalating => {
                if self.cortisol_avg > ACTIVATION_THRESHOLD 
                   && self.cortisol_history.len() >= WINDOW_SIZE / 2 {
                    // Sustained high cortisol — activate Firefighter
                    self.state = TraumaState::FirefighterMode;
                    self.total_activations += 1;
                    self.calm_ticks = 0;
                } else if self.cortisol_avg < 0.4 {
                    self.state = TraumaState::Stable;
                }
            },
            TraumaState::FirefighterMode => {
                if cortisol < DEACTIVATION_THRESHOLD {
                    self.calm_ticks += 1;
                    if self.calm_ticks >= SUSTAINED_DEACTIVATION_TICKS {
                        self.state = TraumaState::Recovering;
                        self.calm_ticks = 0;
                    }
                } else {
                    self.calm_ticks = 0;
                }
            },
            TraumaState::Recovering => {
                if self.cortisol_avg < 0.2 {
                    self.state = TraumaState::Stable;
                    // Clear history for fresh start
                    self.cortisol_history.clear();
                } else if self.cortisol_avg > ACTIVATION_THRESHOLD {
                    // Relapse!
                    self.state = TraumaState::FirefighterMode;
                    self.total_activations += 1;
                }
            },
        }

        self.state != old_state
    }

    /// Get defensive parameter overrides for the LLM
    pub fn get_overrides(&self) -> FirefighterOverrides {
        match self.state {
            TraumaState::Stable => FirefighterOverrides::none(),
            TraumaState::Escalating => FirefighterOverrides {
                temperature_clamp: Some(0.8),  // Slightly cooler
                sensory_dampening: 0.1,        // Slight dampening
                force_consolidation: false,
                serotonin_boost: 0.0,
            },
            TraumaState::FirefighterMode => FirefighterOverrides {
                temperature_clamp: Some(0.4),  // Very conservative
                sensory_dampening: 0.6,        // Major dampening
                force_consolidation: true,     // Process the trauma
                serotonin_boost: 0.01,         // Emergency serotonin per tick
            },
            TraumaState::Recovering => FirefighterOverrides {
                temperature_clamp: Some(0.6),  // Still cautious
                sensory_dampening: 0.3,        // Moderate dampening
                force_consolidation: false,
                serotonin_boost: 0.005,        // Gentle serotonin
            },
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, TraumaState::FirefighterMode | TraumaState::Recovering)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FirefighterOverrides {
    /// Maximum temperature for LLM (None = no override)
    pub temperature_clamp: Option<f32>,
    /// How much to dampen sensory input (0.0 = none, 1.0 = full block)
    pub sensory_dampening: f32,
    /// Whether to force memory consolidation this tick
    pub force_consolidation: bool,
    /// Serotonin to inject per tick (emergency mood stabilization)
    pub serotonin_boost: f32,
}

impl FirefighterOverrides {
    fn none() -> Self {
        Self {
            temperature_clamp: None,
            sensory_dampening: 0.0,
            force_consolidation: false,
            serotonin_boost: 0.0,
        }
    }
}
