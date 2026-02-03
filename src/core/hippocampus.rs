use crate::core::memory_vector::VectorStore;
use anyhow::Result;

pub struct Hippocampus {
    store: VectorStore,
}

impl Hippocampus {
    pub fn new() -> Result<Self> {
        Ok(Self {
            store: VectorStore::new()?,
        })
    }

    /// Guarda un evento en la memoria a largo plazo (Volatile RAM)
    pub fn remember(&mut self, text: &str, context: &str, entropy: f32) -> Result<()> {
        self.store.add(text.to_string(), vec![context.to_string()], entropy)
    }

    /// Retorna similitud máxima (0.0 - 1.0) para detectar Habituación
    pub fn check_novelty(&self, text: &str) -> Result<f32> {
        self.store.get_max_similarity(text)
    }

    /// Ciclo de Sueño: Consolida recuerdos importantes y limpia RAM
    pub fn consolidate_sleep(&mut self) -> Result<usize> {
        self.store.consolidate_memories()
    }

    /// Dada una situación actual, recupera recuerdos relevantes y su score máximo
    pub fn recall_relevant(&self, query: &str) -> Option<(String, f32)> {
        let results = self.store.search(query, 3).ok()?;
        
        if results.is_empty() {
            return None;
        }

        let max_score = results.first().map(|(_, s)| *s).unwrap_or(0.0);

        // Formatea los recuerdos en un bloque de texto para el Prompt
        let mut block = String::from("--- MEMORIA A LARGO PLAZO ---\n");
        for (text, score) in results {
            if score > 0.4 { // Umbral de relevancia
                block.push_str(&format!("- (Similitud {:.2}): {}\n", score, text));
            }
        }
        
        if block.len() > 30 {
            Some((block, max_score))
        } else {
            None
        }
    }
    /// Retorna la cantidad de recuerdos almacenados (Sinapsis Totales)
    pub fn memory_count(&self) -> usize {
        self.store.memory_count()
    }

    /// Persiste memoria actual a disco (sin consolidar). MECHANICAL HONESTY: identidad sobrevive al cierre.
    pub fn save(&self) -> Result<()> {
        self.store.save()
    }
}
