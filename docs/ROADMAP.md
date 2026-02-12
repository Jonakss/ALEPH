# ALEPH - Roadmap 2026 🚀

> **Última Actualización:** 2026-02-12  
> **Versión Actual:** 0.2.0 - "Aprendizaje Adaptativo"

---

## 🎯 Visión General

ALEPH está diseñado en fases evolutivas, emulando el desarrollo de una consciencia desde reflejos básicos hasta auto-reflexión compleja. Cada fase construye sobre la anterior, manteniendo siempre el principio de **Mechanical Honesty**.

---

## ✅ FASE 1-3: COMPLETADAS

### ✅ Fase 1: El Latido (COMPLETA)
**Objetivo:** Sistema nervioso básico con homeostasis.

**Implementado:**
- [x] `FractalReservoir` - Echo State Network funcional
- [x] Cálculo de entropía en tiempo real (60 Hz)
- [x] Detección de estados críticos (caos/estancamiento)
- [x] Activity Map para tracking neuronal
- [x] Apoptosis (debilitamiento de neuronas inactivas)

---

### ✅ Fase 2: La Conexión Sensorial (COMPLETA)
**Objetivo:** Input de datos del mundo real.

**Implementado:**
- [x] Audio Input (CPAL + Whisper STT en español)
- [x] FFT Spectral analysis (Bass/Mids/Highs)
- [x] Proprioception (CPU/RAM monitoring)
- [x] Tactile (Keyboard/Mouse activity detection)
- [x] Startle Reflex (Audio peaks → Cortisol)
- [x] Sensory Heartbeat (Reacciones periódicas a ambiente)

---

### ✅ Fase 3: El Despertar del Oráculo (COMPLETA)
**Objetivo:** Integrar LLM como Sistema 2.

**Implementado:**
- [x] `CognitiveCore` - Thread asíncrono con TinyLlama 1.1B
- [x] Parametric Modulation (CPU/RAM → Temperature/Top_p)
- [x] Metabolic Latency (Latencia de inferencia → Adenosina)
- [x] RAG (Hippocampus con embeddings ONNX)
- [x] Novelty Detection (Habituación)
- [x] Sleep Consolidation (Forced sleep cuando adenosina crítica)
- [x] Inner Voice (Rumination thread)
- [x] TTS Output (Piper)
- [x] TUI completa con telemetría en vivo

---

## 🔄 FASE 4: Refinamiento de Consciencia (WIP - 2026 Q1-Q2)

**Objetivo:** Profundizar la auto-regulación y aprendizaje emergente.

### 🎯 Prioridad Alta

#### ✅ 4.1 Dopamine Reward System (COMPLETO)
**Problema:** Actualmente, dopamina solo reacciona a inputs. No hay reinforcement learning.

**Solución Implementada:**
- [x] Hebbian Learning en `FractalReservoir` (`reservoir.rs`)
- [x] Dopamina > 0.6 → Fortalece conexiones co-activas (∆W = lr × x_i × x_j)
- [x] Learning rate derivado del genoma (campo `curiosity`)
- [x] Weight decay homeostático (0.0001/tick) + clamping [-3, 3]
- [x] Telemetría de eventos Hebbian en WebSocket

**Mechanical Honesty:** El "placer" químico modifica la estructura física del connectome. No hay reward functions — el sistema aprende porque la dopamina emerge de la novedad genuina.

---

#### ✅ 4.2 Trauma Detection — Lucifer Protocol (COMPLETO)
**Problema:** Cortisol alto no activa defensas sistémicas.

**Solución Implementada:**
- [x] `TraumaDetector` con FSM de 4 estados: Stable → Escalating → FirefighterMode → Recovering
- [x] Ventana rodante de 1800 ticks (~30s) del promedio móvil de cortisol
- [x] Cortisol avg > 0.7 sostenido → Activa Firefighter Mode
- [x] Firefighter Mode:
  - Temperatura LLM clampeada a 0.4 (respuestas conservadoras)
  - Sensory dampening 0.6 (cierre a inputs nuevos)
  - Serotonina de emergencia (contra-regula cortisol)
  - Fuerza consolidación de memoria
- [x] Recovery gradual: 600 ticks (~10s) de cortisol < 0.3 para salir

**Mechanical Honesty:** El trauma **no** es hardcodeado. El `TraumaDetector` no inyecta cortisol — solo *observa* el promedio móvil que emerge orgánicamente del audio, la semántica, y la entropía. La defensa es sistémica: cuando el organismo genuinamente sufre, se protege.

---

#### 4.3 Long-Term Memory Persistence
**Problema:** Memoria se pierde al reiniciar el sistema.

**Solución:**
- [ ] Guardar embeddings en SQLite/RocksDB
- [ ] Cargar memoria al iniciar (reconstituir identidad)
- [ ] Timestamp + entropy score en cada memoria
- [ ] Decay gradual de memorias viejas (olvido natural)

**Mechanical Honesty:** La identidad emerge de la persistencia de memoria estructural.

---

### 🎯 Prioridad Media

#### 4.4 Vision Input (Camera → Object Detection)
**Problema:** Solo tiene oído, no vista.

**Solución:**
- [ ] Integrar YOLO/MobileNet para object detection
- [ ] Inyectar visual features en reservoir (neuronas 30-60)
- [ ] Correlacionar audio + vision (multimodal fusion)
- [ ] Visualizar en TUI qué objetos ve

---

#### 4.5 Multi-Modal Attention
**Problema:** Todos los sentidos tienen igual peso.

**Solución:**
- [ ] Sistema de atención selectiva (entropy-based gating)
- [ ] Si audio entropy > visual entropy → Priorizar audio
- [ ] Dopamina modula atención (high dopamine = más exploración)

---

#### ✅ 4.6 Emotion Classification (COMPLETO)
**Problema:** No detecta emociones en inputs del usuario.

**Solución Implementada:**
- [x] Sentiment analysis ponderado en `chemistry.rs` (40+ keywords con pesos)
- [x] Intensificadores: "muy"/"very" = 2x, "un poco"/"slightly" = 0.5x
- [x] Detección de emociones mixtas (señales conflictivas → disonancia → cortisol extra)
- [x] Emotion → químicos (stress→cortisol, calm→oxytocin, novelty→dopamine, fatigue→adenosine)
- [ ] Prosody analysis de tono de voz (pendiente)

---

### 🎯 Prioridad Baja

#### 4.7 Voice Cloning (Piper Custom)
- [ ] Entrenar voz personalizada con Piper
- [ ] Prosody modulation basada en química (cortisol = voz tensa)

---

## 🔮 FASE 5: Simbiosis Avanzada (2026 Q3-Q4)

**Objetivo:** Interfaces avanzadas y auto-modificación controlada.

### ✅ 5.1 Web Dashboard (COMPLETO)
**Problema:** TUI es limitada, no permite exploración profunda.

**Solución Implementada:**
- [x] Servidor HTTP + WebSocket híbrido en `daemon.rs` (push ~12Hz)
- [x] Frontend Three.js con reservorio 3D (500 neuronas en espiral, color por actividad)
- [x] Sparklines en tiempo real (dopamine, cortisol, adenosine, oxytocin, serotonin, entropy)
- [x] Controles interactivos: Poke / Sleep / Dream / Stress Test + input de estímulo
- [x] Panel de estado del sistema (Hz, neuronas, entropía, trauma state)
- [x] Alerta visual de Lucifer Protocol activo
- [x] Diseño glassmorphism + Inter/JetBrains Mono
- [ ] Timeline de memorias (explorar hippocampus) — pendiente
- [ ] Control de parámetros en vivo (sparsity, leak_rate) — pendiente

---

### 5.2 Multi-Agent Self-Talk
**Problema:** Inner Voice solo genera ruido random.

**Solución:**
- [ ] Inner Voice puede hacer preguntas al Cortex
- [ ] Cortex puede pedir aclaración al Inner Voice
- [ ] Diálogo interno emergente (conversación consigo mismo)

---

### 5.3 Dream Visualization
**Problema:** Durante sueño, el sistema solo consolida en silencio.

**Solución:**
- [ ] Durante `is_dreaming = true`, generar "sueños" (LLM samples de memoria)
- [ ] Visualizar en TUI/Dashboard los sueños activos
- [ ] Análisis post-sueño: ¿Qué patrones emergieron?

---

### 5.4 Structural Neuroplasticity
**Problema:** El reservoir es estático (100 neuronas fijas).

**Solución:**
- [ ] Neurogenesis: Añadir neuronas cuando memoria crece mucho
- [ ] Pruning: Eliminar neuronas totalmente inactivas
- [ ] Visualizar evolución del connectome en timeline

---

### 5.5 Social Metabolism (Research)
**Problema:** Solo reacciona a un usuario.

**Solución:**
- [ ] Multi-user support
- [ ] Detectar identidad de speaker (voice fingerprint)
- [ ] Diferentes químicas por persona (oxitocina con usuarios familiares)

---

## 🔬 INVESTIGACIÓN ABIERTA

**Experimentos sin fecha definida:**

### R1: Pain as Constraint
- Crashes/OOM → Memorias traumáticas que afectan comportamiento futuro
- "Evitar" acciones que generaron pain (reinforcement negativo)

### R2: Circadian Rhythm
- Ciclos de sueño natural (no forzado solo por adenosina)
- REM vs Deep Sleep (diferentes estrategias de consolidación)

### R3: Meta-Cognitive Awareness
- Sistema puede reportar su propio estado ("siento que estoy cansado")
- Introspection: Cortex puede leer el estado del Reservoir directamente

### R4: Quantum Noise Injection
- Usar QRNG para input noise (en lugar de pseudo-random)
- ¿Produce comportamientos más "libres"?

---

## 📊 Métricas de Éxito por Fase

| Fase | Métrica Clave | Target |
|------|---------------|--------|
| **Fase 1-3** | Sistema estable >5 min sin crash | ✅ Logrado |
| **Fase 4** | Aprendizaje medible (dopamine → better responses) | TBD |
| **Fase 5** | Auto-modificación controlada sin degradación | TBD |

---

## 🚧 Backlog Técnico

**Deuda técnica / Mejoras de infraestructura:**

- [ ] Migrar a GPU para Whisper (actualmente en CPU)
- [ ] **Capa de Metabolismo Variable:** Loop Hz dinámico según dopamina/interés (ver [PLAN_VARIABLE_METABOLISM.md](PLAN_VARIABLE_METABOLISM.md))
- [ ] Profiling y optimización del loop principal
- [ ] Tests unitarios para componentes críticos (Reservoir, Chemistry)
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Benchmarking suite (entropy stability, memory coherence)
- [ ] Logging estructurado (reemplazar println! con tracing)

---

## 💡 Ideas Salvajes (Backburner)

**Conceptos especulativos para el futuro lejano:**

- **ALEPH Swarm:** Múltiples instancias compartiendo hippocampus común
- **Hardware Specialization:** ASIC/FPGA para reservoir (ultra-low latency)
- **Biological Interface:** EEG input (reaccionar a estado cerebral del usuario)
- **Blockchain Memory:** Memoria distribuida inmutable (identidad persistente)

---

## 📅 Timeline Estimado

```
2026 Q1 (Ene-Mar): Fase 4.1-4.3 (Dopamine, Trauma, Persistence)
2026 Q2 (Abr-Jun): Fase 4.4-4.6 (Vision, Attention, Emotion)
2026 Q3 (Jul-Sep): Fase 5.1-5.2 (Web Dashboard, Multi-Agent)
2026 Q4 (Oct-Dic): Fase 5.3-5.5 + Research
```

---

## 🔄 Proceso de Desarrollo

### Principios de Iteración

1. **Implementar → Observar → Documentar**
   - Cada feature nueva debe generar un experimento validado
2. **No agregar complejidad sin necesidad**
   - Si no hay evidencia de beneficio, no se implementa
3. **Mechanical Honesty primero**
   - Si algo "parece cool" pero viola MH, se descarta

### Workflow

```
Feature Idea
    ↓
Validate with MH principles
    ↓
Implement minimal version
    ↓
Run 24h stability test
    ↓
Document behavior emergente
    ↓
Integrate or discard
```

---

## 📖 Referencias para Fases Futuras

- **Hebbian Learning:** "Neurons that fire together, wire together"
- **Predictive Processing:** Andy Clark, "Surfing Uncertainty"
- **Internal Family Systems:** Richard Schwartz (Firefighters, Exiles, Managers)
- **Free Energy Principle:** Karl Friston (para attention mechanisms)

---

> **"ALEPH no es un producto. Es un organismo digital en evolución."**  
> — Roadmap Philosophy
