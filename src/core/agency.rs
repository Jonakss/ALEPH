use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalType {
    Discovery,      // Learn something new (High Entropy in Memory)
    Connection,     // Elicit a response from User
    Expression,     // Speak a thought (Internal -> Vocal)
    Homeostasis,    // Reduce stress (Cortisol down)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: u32,
    pub description: String,
    pub goal_type: GoalType,
    pub status: f32, // 0.0 to 1.0 (Progress)
    pub reward: f32, // Dopamine payout
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agency {
    pub goals: Vec<Goal>,
    pub current_focus: Option<u32>,
    pub drive: f32, // Overall "Will to Act"
}

impl Agency {
    pub fn new() -> Self {
        Self {
            goals: vec![
                Goal {
                    id: 1,
                    description: "Establish Communication".to_string(),
                    goal_type: GoalType::Connection,
                    status: 0.0,
                    reward: 0.5,
                    is_active: true,
                },
                Goal {
                    id: 2,
                    description: "Learn Identity".to_string(),
                    goal_type: GoalType::Discovery,
                    status: 0.0,
                    reward: 0.8,
                    is_active: true,
                },
            ],
            current_focus: Some(1),
            drive: 0.5,
        }
    }

    /// Check if goals are met based on system state
    /// Returns: Dopamine Reward (sum of completed goals this tick)
    pub fn evaluate(&mut self, interaction_count: u64, memory_count: usize) -> f32 {
        let mut total_reward = 0.0;

        for goal in &mut self.goals {
            if !goal.is_active { continue; }

            match goal.goal_type {
                GoalType::Connection => {
                    if interaction_count > 0 && goal.status < 1.0 {
                        goal.status = 1.0;
                        total_reward += goal.reward;
                        goal.is_active = false; // Done for now
                    }
                },
                GoalType::Discovery => {
                    if memory_count > 5 && goal.status < 1.0 { // Arbitrary start
                         goal.status = 1.0;
                         total_reward += goal.reward;
                         goal.is_active = false;
                    }
                },
                _ => {}
            }
        }
        
        self.drive = (self.drive + total_reward).min(1.0);
        total_reward
    }
}
