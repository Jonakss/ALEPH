use nalgebra::DVector;
use std::collections::VecDeque;
use std::time::Instant;
use rand::prelude::*;

const MAX_MEMORIES: usize = 50;

#[derive(Clone, Debug)]
pub struct Memory {
    pub stimulus: f32,
    pub original_entropy: f32,
    pub timestamp: Instant,
}

pub struct Hippocampus {
    memories: VecDeque<Memory>,
}

impl Hippocampus {
    pub fn new() -> Self {
        Self {
            memories: VecDeque::new(),
        }
    }

    /// Guarda un recuerdo (Input Audio + Entropia que causó)
    pub fn remember(&mut self, stimulus: f32, entropy: f32) {
        // Política de Olvido: FIFO
        if self.memories.len() >= MAX_MEMORIES {
            self.memories.pop_front();
        }
        self.memories.push_back(Memory {
            stimulus,
            original_entropy: entropy,
            timestamp: Instant::now(),
        });
    }

    /// Retorna un recuerdo aleatorio para Rumiar
    pub fn replay_memory(&self) -> Option<Memory> {
        if self.memories.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.memories.len());
        self.memories.get(index).cloned()
    }


    pub fn memory_count(&self) -> usize {
        self.memories.len()
    }
}
