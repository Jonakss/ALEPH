use std::collections::VecDeque;

/// Legacy: Audio memory buffer for future FFT trend analysis
#[allow(dead_code)]
pub struct AudioMemory {
    buffer: VecDeque<f32>,
    capacity: usize,
}

#[allow(dead_code)]
impl AudioMemory {
    pub fn new(seconds: usize, sample_rate: usize) -> Self {
        // Asumiendo que guardamos RMS por tick (60Hz), no samples de audio raw (44100Hz)
        // para ahorrar memoria y facilitar "resumen".
        let capacity = seconds * sample_rate; 
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: f32) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(value);
    }

    pub fn get_statistics(&self) -> (f32, f32) {
        if self.buffer.is_empty() {
            return (0.0, 0.0);
        }
        let sum: f32 = self.buffer.iter().sum();
        let avg = sum / self.buffer.len() as f32;
        let max = *self.buffer.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);
        (avg, max)
    }

    pub fn get_recent_trend(&self) -> &'static str {
        let (avg, _max) = self.get_statistics();
        if avg < 0.1 { "SILENCE" }
        else if avg < 0.5 { "NORMAL ACTIVITY" }
        else { "HIGH NOISE / CHAOS" }
    }
}
