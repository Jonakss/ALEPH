use crate::core::memory_vector::VectorStore;
use crate::core::genome::Genome;
use crate::core::materializer::SoulMaterializer;
use anyhow::Result;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

pub struct MemoryOutput {
    pub input_text: String,
    pub novelty: f32, // 0.0 - 1.0 (1.0 = New)
    pub retrieval: Option<(String, f32)>, // (Context, Relevance)
    pub _volatile_count: usize,
    pub _total_count: usize,
}

pub enum MemoryCommand {
    ProcessStimulus { text: String, entropy: f32 },
    #[allow(dead_code)]
    ConsolidateSleep,
    #[allow(dead_code)]
    ForceSave, // Optional, but we prefer Sleep-based persistence
    // Shutdown includes session stats for the alchemist
    Shutdown { previous_genome: Genome, avg_friction: f32, reply_tx: Sender<Genome> },
}

pub struct Hippocampus {
    store: VectorStore,
}

impl Hippocampus {
    /// Spawns the Hippocampus in a background thread.
    /// Returns: (CommandSender, OutputReceiver)
    pub fn spawn() -> Result<(Sender<MemoryCommand>, Receiver<MemoryOutput>, Receiver<String>)> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<MemoryCommand>();
        let (out_tx, out_rx) = mpsc::channel::<MemoryOutput>();
        let (log_tx, log_rx) = mpsc::channel::<String>(); // Logic logs for TUI

        thread::spawn(move || {
            let mut hippo = match Self::new() {
                Ok(h) => {
                    let _ = log_tx.send("Hippocampus: ONLINE (CUDA/CPU)".to_string());
                    h
                },
                Err(e) => {
                    let _ = log_tx.send(format!("Hippocampus KILLED: {}", e));
                    return;
                }
            };

            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    MemoryCommand::ProcessStimulus { text, entropy } => {
                        match hippo.process(text, entropy) {
                            Ok(output) => { let _ = out_tx.send(output); },
                            Err(e) => { let _ = log_tx.send(format!("Memory Error: {}", e)); }
                        }
                    },
                    MemoryCommand::ConsolidateSleep => {
                        match hippo.store.consolidate_memories() {
                            Ok(forgotten) => {
                                let _ = log_tx.send(format!("💤 Sleep Cycle: Consolidated. Pruned {} weak memories.", forgotten));
                                // EVENT: Trigger structural growth
                                let _ = out_tx.send(MemoryOutput {
                                    input_text: "CONSOLIDATION_EVENT".to_string(),
                                    novelty: 1.0, // High novelty to signify importance
                                    retrieval: None,
                                    _volatile_count: 0,
                                    _total_count: hippo.store.memory_count(),
                                });
                            },
                            Err(e) => { let _ = log_tx.send(format!("Sleep Error: {}", e)); }
                        }
                    },
                    MemoryCommand::ForceSave => {
                        let _ = hippo.store.save(); // Just in case
                    },
                    MemoryCommand::Shutdown { previous_genome, avg_friction, reply_tx } => {
                        let _ = log_tx.send("💀 Hippocampus: Shutting down... Crystallizing Soul.".to_string());
                        
                        // 1. Save Raw Memories (Persistence)
                        let _ = hippo.store.save();

                        // 2. Crystallize (Now using Friction)
                        let new_genome = SoulMaterializer::crystallize(&hippo.store, previous_genome, avg_friction);
                        
                        // 2. Reply
                        let _ = reply_tx.send(new_genome);
                        
                        // 3. Die
                        break; 
                    }
                }
            }
        });

        Ok((cmd_tx, out_rx, log_rx))
    }

    fn new() -> Result<Self> {
        Ok(Self {
            store: VectorStore::new()?,
        })
    }

    /// Optimized: Single BERT pass for all cognitive functions
    fn process(&mut self, text: String, entropy: f32) -> Result<MemoryOutput> {
         // 1. Generate Embedding (Expensive Part - Done ONCE)
         let vector = self.store.embed(&text)?;
         
         // 2. Check Novelty (Vector comparison)
         let max_sim = self.store.memories.iter()
            .map(|mem| {
                 mem.embedding.iter().zip(&vector).map(|(a, b)| a * b).sum::<f32>()
            })
            .fold(0.0f32, |acc, x| f32::max(acc, x));
         
         let novelty = 1.0 - max_sim;

         // 3. Retrieval (RAG)
         // Search top 3 relevant using the SAME vector
         let mut scores: Vec<(usize, f32)> = self.store.memories.iter().enumerate().map(|(i, mem)| {
            let cosine_sim: f32 = mem.embedding.iter().zip(&vector)
                .map(|(a, b)| a * b).sum();
            (i, cosine_sim)
        }).collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let retrieval = if let Some((idx, score)) = scores.first() {
             if *score > 0.4 {
                  let ctx_block = format!("Recuerdo Relacionado (Sim: {:.2}): {}", score, self.store.memories[*idx].text);
                  Some((ctx_block, *score))
             } else {
                 None
             }
        } else {
            None
        };

        // 4. Store (Short Term Memory)
        // Manual add to avoid re-embedding
        self.store.add_precalculated(text.clone(), vector, vec!["input".to_string()], entropy)?;

        Ok(MemoryOutput {
            input_text: text,
            novelty,
            retrieval,
            _volatile_count: self.store.volatile_count(),
            _total_count: self.store.memory_count(),
        })
    }
}
