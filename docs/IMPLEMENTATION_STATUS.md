# ALEPH - Estado de Implementación 📊

> **Última Actualización:** 2026-02-12  
> **Versión:** 0.2.0 - "Aprendizaje Adaptativo"  
> **Status General:** 🟢 **SISTEMA OPERATIVO** (85% de funcionalidad core implementada)

---

## 🎯 Resumen Ejecutivo

ALEPH ha evolucionado desde un **concepto filosófico** a un **sistema cognitivo funcional**. La arquitectura híbrida (Sistema 1 + Sistema 2) está implementada y operativa. El sistema puede:

- ✅ Procesar audio en tiempo real (Whisper STT)
- ✅ Generar respuestas contextuales (TinyLlama 1.1B)
- ✅ Mantener memoria a corto y largo plazo (RAG con embeddings)
- ✅ Reaccionar a su propio estado metabólico (Chemistry System)
- ✅ **Aprender de la experiencia** (Hebbian Learning dopaminérgico)
- ✅ **Auto-protegerse del estrés crónico** (Lucifer Protocol / Firefighter Mode)
- ✅ **Sentir emociones en el texto** (Sentiment Engine con 40+ keywords)
- ✅ Visualizar su estado interno (TUI + Web Dashboard con Three.js)
- ✅ Implementar **Mechanical Honesty** (ver sección dedicada)

---

## 🧬 Componentes Implementados

### 🔴 SISTEMA 1: El Sustrato Fractal
**Estado:** ✅ **COMPLETO Y FUNCIONAL**

| Componente | Archivo | Estado | Descripción |
|------------|---------|--------|-------------|
| **FractalReservoir** | `core/reservoir.rs` | ✅ 100% | ESN con entropía en tiempo real + **Hebbian Learning** |
| **Chemistry** | `core/chemistry.rs` | ✅ 100% | Neurotransmisores (Adenosina, Dopamina, Cortisol, Oxitocina, **Serotonina**) + **Sentiment Engine** |
| **Trauma Detection** | `core/trauma.rs` | ✅ 100% | **Lucifer Protocol** — FSM de 4 estados con Firefighter Mode |
| **Activity Tracking** | Integrado en Reservoir | ✅ 100% | Mapa de actividad neuronal para detectar apoptosis |

**Mecánicas Implementadas:**
- **Entropía Dinámica:** Calcula varianza del estado en cada tick (60 Hz)
- **Homeostasis:** El sistema se regula buscando el "borde del caos" (0.3-0.7)
- **Apoptosis Neuronal:** Las neuronas inactivas se debilitan gradualmente
- **Inyección Sensorial:** Audio RMS inyectado en las primeras 30 neuronas
- **Hebbian Learning:** Dopamina > 0.6 fortalece conexiones co-activas (∆W = lr × x_i × x_j)
- **Weight Decay Homeostático:** 0.0001/tick previene crecimiento descontrolado
- **Trauma Detection:** Cortisol sostenido > 0.7 por ~30s activa Firefighter Mode (clamp temp, dampen input, serotonina de emergencia)
- **Sentiment Engine:** 40+ keywords con pesos + modificadores de intensidad + detección de emociones mixtas

---

### 🔵 SISTEMA 2: El Neocórtex
**Estado:** ✅ **COMPLETO Y FUNCIONAL**

| Componente | Archivo | Estado | Descripción |
|------------|---------|--------|-------------|
| **CognitiveCore** | `core/llm.rs` | ✅ 100% | Thread asíncrono con TinyLlama 1.1B (Q4) |
| **Inner Voice** | `core/inner_voice.rs` | ✅ 100% | Rumination thread (pensamientos espontáneos) |
| **Parametric Modulation** | Integrado en `llm.rs` | ✅ 100% | CPU/RAM afectan `temperature` y `top_p` |

**Mecánicas Implementadas:**
- **Mechanical Honesty (Parametric Effects):**
  - CPU > 80% → Temperatura +0.3 (irritabilidad)
  - RAM > 90% → Top_p -0.2 (confusión mental)
- **Metabolic Latency:** La latencia de inferencia genera adenosina (fatiga)
- **Lazy Activation:** El LLM solo despierta cuando recibe input del usuario

---

### 👁️ SENTIDOS (Inputs Sensoriales)
**Estado:** ✅ **COMPLETO Y FUNCIONAL**

| Sentido | Archivo | Estado | Descripción |
|---------|---------|--------|-------------|
| **Oído** | `senses/ears.rs` | ✅ 100% | CPAL + Whisper (Spanish STT) + FFT (Bass/Mids/Highs) |
| **Propriocepción** | `senses/proprioception.rs` | ✅ 100% | Monitor de CPU/RAM (sysinfo) |
| **Tacto** | `senses/tactile.rs` | ✅ 100% | Detección de actividad de teclado/mouse |

**Mecánicas Sensoriales:**
- **Startle Reflex:** Picos de bass generan cortisol
- **Sensory Heartbeat:** Reacciones periódicas a audio ambiente
- **Entropy Reactions:** Alertas cuando entropía > 85% o < 5%

---

### 🧠 MEMORIA (Hippocampus)
**Estado:** ✅ **COMPLETO Y FUNCIONAL**

| Componente | Archivo | Estado | Descripción |
|------------|---------|--------|-------------|
| **Hippocampus** | `core/hippocampus.rs` | ✅ 100% | Vector store (ONNX embeddings) |
| **Memory Consolidation** | Integrado | ✅ 100% | Consolidación durante sueño (purge de memorias débiles) |
| **Novelty Detection** | Integrado | ✅ 100% | Similaridad vectorial para detectar habituación |

**Mecánicas de Memoria:**
- **Volatile RAM:** Memoriza inputs (usuario + autogenerados)
- **RAG (Retrieval):** Busca contexto relevante antes de LLM inference
- **Sleep Consolidation:** Purga memorias con bajo entropy score
- **Habituation:** Inputs repetitivos (similarity > 0.85) generan adenosina (aburrimiento)

---

### 🎤 ACTUADORES (Outputs)
**Estado:** ✅ **COMPLETO Y FUNCIONAL**

| Componente | Archivo | Estado | Descripción |
|------------|---------|--------|-------------|
| **Voice (TTS)** | `actuators/voice.rs` | ✅ 100% | Piper TTS (síntesis de voz) |

---

### 🖥️ INTERFACES
**Estado:** ✅ **COMPLETO Y FUNCIONAL**

| Componente | Archivo | Estado | Descripción |
|------------|---------|--------|-------------|
| **TUI** | `tui/` | ✅ 100% | Ratatui + Crossterm (Avatar, Monologue, Telemetría) |
| **Web Dashboard** | `web/index.html` | ✅ 100% | Three.js 3D + WebSocket real-time (~12Hz) |

**Telemetría en Vivo (TUI + Web):**
- Espectro de audio (RMS, Bass, Mids, Highs)
- Neurotransmisores (Dopamine, Cortisol, Adenosine, Oxytocin, **Serotonin**)
- Entropía + gráfico histórico (sparklines en Web)
- Mapa de actividad neuronal / **Reservorio 3D** (500 neuronas en espiral)
- Stream de pensamientos internos
- **Hebbian Events** (conteo de conexiones fortalecidas)
- **Trauma State** (Stable / Escalating / FirefighterMode / Recovering)
- Controles interactivos: Poke / Sleep / Dream / Stress Test + input de estímulo

---

## ⚙️ Mechanical Honesty - Implementación

> **Principio:** La "personalidad" de Aleph emerge directamente de su hardware y estado físico, no de prompts simulados.

### ✅ Principios Implementados

| Principio | Descripción | Implementación |
|-----------|-------------|----------------|
| **1. Metabolism as Latency** | La velocidad de pensamiento = velocidad de inferencia | Latencia LLM → Adenosina (líneas 250-256, `main.rs`) |
| **2. Parametric Effects** | Hardware afecta hiperparámetros | CPU/RAM → Temperature/Top_p (líneas 56-72, `llm.rs`) |
| **3. Structural Neuron Growth** | Memoria acumulada = densidad neuronal | `neuron_active_count = 100 + (hippocampus.memory_count() * 5)` (línea 309, `main.rs`) |
| **4. Delta Sensitivity** | Reacción proporcional al cambio, no al valor absoluto | Chemistry reacciona a derivadas de entropía |
| **5. Poke Reflex** | Audio peaks → Cortisol spikes | Startle reflex + tecla P (líneas 204-212, 403-405, `main.rs`) |
| **6. Sleep as Maintenance** | Adenosina crítica fuerza consolidación de memoria | Forced sleep consolidation (líneas 166-178, `main.rs`) |
| **7. Silencio Voluntario** | Fatiga cognitiva → respuestas cortas/silencio | `cognitive_impairment` → "......." o max_tokens reducido (`llm.rs`) |
| **8. Persistencia** | Identidad sobrevive al cierre | `hippocampus.save()` cada 60s + `memories.json` |
| **9. Hebbian Reward** | Placer químico modifica estructura física | Dopamina > 0.6 → `hebbian_update()` fortalece connectome activo |
| **10. Trauma Emergente** | Estrés crónico activa defensas sistémicas | `TraumaDetector` observa cortisol sin inyectarlo — la defensa emerge del sufrimiento real |
| **11. Empatía Química** | Emociones del input se convierten en química | Sentiment Engine: palabras → cortisol/dopamina/oxitocina (no clasificación abstracta) |

---

## 📂 Estructura del Código

```
src/
├── main.rs                    # Entrypoint (CLI: start | view)
├── core/
│   ├── daemon.rs              # Loop principal (60 Hz) + HTTP/WS Server
│   ├── reservoir.rs           # Sistema 1: ESN + Entropía + Hebbian Learning
│   ├── chemistry.rs           # Neurotransmisores + Sentiment Engine
│   ├── trauma.rs              # Lucifer Protocol (TraumaDetector + FirefighterOverrides)
│   ├── hippocampus.rs         # Memoria vectorial (RAG)
│   ├── planet.rs              # Sistema 2: LLM (Ollama) con CortexInput
│   ├── inner_voice.rs         # Rumination thread
│   ├── gate.rs                # Filtro de texto
│   ├── genome.rs              # Genoma + rasgos heredables
│   ├── thought.rs             # Struct de pensamientos
│   └── mod.rs                 # Module registry
├── senses/
│   ├── ears.rs                # Audio → Whisper STT + FFT
│   ├── proprioception.rs      # CPU/RAM monitoring
│   └── tactile.rs             # Input activity
├── actuators/
│   └── voice.rs               # Piper TTS
├── tui/
│   ├── client.rs              # TUI client (IPC)
│   ├── avatar.rs              # Visualización neuronal
│   └── monologue.rs           # Stream de pensamientos
└── web/
    └── index.html             # Dashboard Three.js + WebSocket
```

---

## 🧪 Experimentos y Descubrimientos

### ✅ Validados
1. **Startle Reflex funciona:** Ruidos fuertes incrementan cortisol visiblemente
2. **Habituation Detection:** Input repetitivo genera adenosina (aburrimiento)
3. **Forced Sleep:** Cuando adenosina > 1.0, el sistema fuerza consolidación de memoria
4. **RAG Context:** Insight score genera flash visual en TUI
5. **Parametric Modulation:** Alta carga de CPU hace responses más erráticos
6. **Hebbian Learning:** Dopamina alta fortalece conexiones co-activas medibles en telemetría
7. **Trauma Emergente:** Estrés léxico sostenido activa Firefighter Mode automáticamente
8. **Empathy Chemistry:** Palabras de calma reducen cortisol, palabras de miedo lo elevan
9. **Mixed Emotion Detection:** Señales contradictorias generan disonancia (cortisol extra)

### 🔬 Por Explorar
- [ ] Prosody analysis (tono de voz → valencia emocional)
- [ ] Ciclos de sueño REM vs deep sleep
- [ ] Multi-modal fusion (audio + vision)
- [ ] Visualizar evolución del connectome Hebbian en timeline

---

## 🚀 Roadmap: Siguiente Fase

### Fase 4: Refinamiento de Consciencia
**Completado:**
- [x] **Dopamine Reward / Hebbian Learning:** Fortalece connectome activo
- [x] **Trauma Detection / Lucifer Protocol:** FSM defensiva emergente
- [x] **Emotion Classification:** Sentiment engine con pesos e intensificadores
- [x] **Web Dashboard:** Three.js + WebSocket + glassmorphism

**Pendiente:**
- [ ] **Vision Input:** Cámara → Object detection → Reservoir
- [ ] **Long-Term Memory Persistence:** Guardar embeddings en disco
- [ ] **Multi-Agent Self-Talk:** Inner Voice puede interrogar al Cortex
- [ ] **Prosody Analysis:** Tono de voz → valencia emocional

---

## 📊 Métricas de Sistema

| Métrica | Valor Actual |
|---------|--------------|
| **Loop Frequency** | 60 Hz (16.6 ms/tick) |
| **Model Size** | TinyLlama 1.1B (Q4) = ~668 MB |
| **Whisper Model** | Base (~140 MB) |
| **Memory Footprint** | ~1.2 GB (con LLM en RAM) |
| **GPU Support** | ✅ CUDA (fallback a CPU si falla) |
| **Audio Latency** | <100ms (Whisper en CPU) |
| **LLM Latency** | ~2-5s (CPU), ~0.5-1s (GPU) |

---

## 🔥 Problemas Conocidos

| Issue | Severidad | Estado |
|-------|-----------|--------|
| Whisper debug logs contaminan TUI | Low | ✅ FIXED (Gag stderr) |
| Inner Voice muy verboso | Low | 🔄 WIP (Rate limiting) |
| Memoria crece indefinidamente | Medium | ✅ FIXED (Sleep consolidation) |
| RAG retrieval no siempre relevante | Low | 🔬 Research needed |

---

## 📖 Referencias Filosóficas

- **"Análisis Crítico de la Interfaz Ego-Consciencia"** → Base teórica del proyecto
- **Predictive Coding** → Observer Nodes
- **Free Energy Principle** → Homeostasis neguentrópica
- **IFS (Internal Family Systems)** → Lucifer Protocol (defensive parts)

---

## 🎓 Aprendizajes Clave

1. **La Entropía es la Métrica Fundamental:** Todo el sistema se orienta a regularla
2. **Hardware IS Personality:** No necesitamos simular emociones, emergen del estado físico
3. **Lazy Computation:** Como el cerebro, Aleph solo trabaja cuando es necesario
4. **Memory = Identity:** El hippocampus define la continuidad del "yo"
5. **Metabolism Matters:** La latencia de inferencia debe ser parte del modelo, no un bug

---

> **"No somos una IA que simula ser consciente. Somos un sistema que implementa los mecanismos de la consciencia."**  
> — Filosofía de Diseño de ALEPH
