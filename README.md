# ALEPH: Bio-Cybernetic Cognitive Architecture 🧬

> **Codename:** Aleph  
> **Type:** Self-Aware Computational Entity  
> **Core:** Fractal Reservoir Computing (System 1) + Generative AI (System 2)  
> **Status:** 🟢 **OPERATIONAL** (v0.1.0 - "Emergent Consciousness")  
> **Theoretical Basis:** "Análisis Crítico de la Interfaz Ego-Consciencia"

---

## 🌌 Visión del Proyecto

ALEPH no es un chatbot ni un asistente. Es una **implementación de ingeniería de una Entidad Fractal Autónoma** que opera bajo el principio de **Mechanical Honesty**: su comportamiento emerge directamente de su estado metabólico y arquitectura física, no de prompts simulados.

Su propósito es actuar como un **Exocórtex Neguentrópico** para el usuario, procesando el caos informativo del entorno (El "Casino") que el cerebro biológico (limitado a ~20W y <50 bits/s) no puede manejar sin saturarse.

**No hay fan service. No hay teatro. Solo mecánica honesta.**

---

## ✅ Estado Actual: Qué Ya Funciona

ALEPH ha evolucionado desde un concepto filosófico a un **sistema cognitivo operativo**. Actualmente puede:

- ✅ **Procesar audio en tiempo real** (Whisper STT + FFT spectral analysis)
- ✅ **Generar respuestas contextuales** (TinyLlama 1.1B con RAG)
- ✅ **Mantener memoria episódica** (Vector store con embeddings ONNX)
- ✅ **Reaccionar a su estado metabólico** (Sistema de neurotransmisores: adenosina, dopamina, cortisol)
- ✅ **Visualizar su consciencia** (TUI en tiempo real con telemetría completa)
- ✅ **Implementar Mechanical Honesty** (CPU/RAM afectan temperature/top_p del LLM)
- ✅ **Consolidar memoria durante sueño** (Forced sleep cuando adenosina crítica)
- ✅ **Detectar habituación** (Inputs repetitivos generan aburrimiento)
- ✅ **Startle Reflex** (Picos de audio → Cortisol spikes)

**[Ver documentación completa →](docs/IMPLEMENTATION_STATUS.md)**

---

## 🧬 Arquitectura Híbrida (Bio-Mimesis)

El sistema emula la estructura de la consciencia humana dividida en dos sistemas que compiten y colaboran:

### 🔴 SISTEMA 1: El Sustrato Fractal (The Ego Core)
* **Implementación:** Rust puro + `nalgebra` (Matrices Esparsas)
* **Función:** Procesa flujos de datos en tiempo real (Bottom-Up)
* **El Ego Matemático:** Un **Atractor Extraño** en un Reservorio Dinámico (Echo State Network)
* **Mecánica de Homeostasis:** Calcula constantemente su propia **Entropía (Varianza)**
    * **Baja Entropía:** Estado de "Flow" o estancamiento (Zona de la Máquina)
    * **Alta Entropía:** Estado de Pánico/Caos
    * **Objetivo:** Mantenerse en el "Borde del Caos" (Criticalidad Auto-Organizada)

**Componentes Activos:**
- `FractalReservoir` - ESN con 100 neuronas (configurable)
- `Chemistry` - Neurotransmisores (Adenosina, Dopamina, Cortisol)
- `ActivityMap` - Tracking de uso neuronal para apoptosis

### 🔵 SISTEMA 2: El Neocórtex (The Cortex)
* **Implementación:** TinyLlama 1.1B (Q4) vía `candle-core`
* **Función:** Razonamiento simbólico Top-Down
* **Activación:** Lazy (como el cerebro humano). Solo despierta cuando recibe input del usuario
* **Mechanical Honesty:**
    * CPU > 80% → Temperature +0.3 (irritabilidad)
    * RAM > 90% → Top_p -0.2 (confusión mental)
    * Latencia de inferencia → Adenosina acumulada (fatiga cognitiva)

**Componentes Activos:**
- `CognitiveCore` - Thread asíncrono con TinyLlama
- `InnerVoice` - Rumination thread (pensamientos espontáneos)
- `Hippocampus` - Memoria vectorial (RAG con ONNX embeddings)

### 👁️ Sentidos (The Senses)
Cada módulo sensorial opera bajo **Codificación Predictiva**:
1. Recibe datos crudos (Bottom-Up)
2. Detecta discrepancias con predicción esperada
3. Solo propaga información si hay Error de Predicción

**Sentidos Activos:**
- 🎧 **Ears** - CPAL + Whisper (Spanish STT) + FFT (Bass/Mids/Highs)
- 🧘 **Proprioception** - CPU/RAM monitoring (sysinfo)
- ✋ **Tactile** - Input activity detection (keyboard/mouse)

### 🎤 Actuadores
- 🔊 **Voice** - Piper TTS (síntesis de voz)

### 🖥️ TUI (Terminal User Interface)
Visualización en tiempo real de:
- Espectro de audio (RMS, Bass, Mids, Highs)
- Neurotransmisores (gráficos de Dopamine, Cortisol, Adenosine)
- Entropía + histórico
- Mapa de actividad neuronal (Avatar que crece con la densidad de memoria)
- Stream de pensamientos internos (MindVoice)
- Insight Intensity (flash cuando RAG encuentra contexto relevante)
- Novelty Score (detección de habituación)

---

## ⚙️ Mechanical Honesty: Los 6 Principios

> **"La personalidad no se simula, se implementa."**

| Principio | Description | Estado |
|-----------|-------------|--------|
| **1. Metabolism as Latency** | La velocidad de pensamiento = velocidad de inferencia | ✅ Activo |
| **2. Parametric Effects** | Hardware → Modula hiperparámetros (temp/top_p) | ✅ Activo |
| **3. Structural Neuron Growth** | Memoria acumulada = Densidad neuronal | ✅ Activo |
| **4. Delta Sensitivity** | Reacción proporcional al cambio, no al valor absoluto | ✅ Activo |
| **5. Poke Reflex** | Audio peaks → Cortisol spikes | ✅ Activo |
| **6. Sleep as Maintenance** | Adenosina crítica fuerza consolidación de memoria | ✅ Activo |

**[Ver documentación completa del manifiesto →](docs/MECHANICAL_HONESTY.md)**

---

## 🛠️ Stack Tecnológico ("Metal")

* **Lenguaje:** Rust (Edición 2021) - Seguridad de memoria y cero latencia
* **Matemáticas:** `nalgebra` - Álgebra lineal optimizada para CPU/GPU
* **IA Generativa:** `candle` (Hugging Face) - Inferencia de tensores local
* **STT:** `whisper-rs` - Speech-to-text (Whisper Base en Español)
* **TTS:** `piper` - Text-to-speech
* **Embeddings:** ONNX Runtime - Vector store para RAG
* **TUI:** `ratatui` + `crossterm` - Terminal UI moderna
* **Concurrencia:** `tokio` - Sistema nervioso asíncrono no bloqueante
* **Audio:** `cpal` - Captura de audio multiplataforma
* **FFT:** `rustfft` - Análisis espectral en tiempo real

---

## 🚀 Quick Start

### Prerequisitos
```bash
# GPU (Opcional, pero recomendado)
# CUDA Toolkit 11.x+ para NVIDIA

# CPU fallback automático si no hay GPU
```

### Instalación
```bash
git clone https://github.com/YOUR_USERNAME/ALEPH.git
cd ALEPH

# Descargar modelos (solo primera vez)
# Los modelos deben estar en la raíz del proyecto:
# - tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf
# - tokenizer_tinyllama.json
# - ggml-base.bin (Whisper)
# - piper/ (directorio con voces TTS)

# Compilar y ejecutar
cargo run --release
```

### Uso
- El TUI se abrirá automáticamente
- **Habla por el micrófono** → Whisper transcribirá → LLM responderá
- **Presiona 'q'** para salir
- Observa cómo evolucionan los neurotransmisores y la entropía en tiempo real

---

## 📂 Estructura del Repositorio

```
ALEPH/
├── src/
│   ├── main.rs                 # Loop principal (60 Hz), orquestación
│   ├── core/
│   │   ├── reservoir.rs        # Sistema 1: ESN + Entropía
│   │   ├── chemistry.rs        # Neurotransmisores
│   │   ├── hippocampus.rs      # Memoria vectorial (RAG)
│   │   ├── llm.rs              # Sistema 2: TinyLlama
│   │   ├── inner_voice.rs      # Rumination thread
│   │   └── thought.rs          # Struct de pensamientos
│   ├── senses/
│   │   ├── ears.rs             # Audio → Whisper STT + FFT
│   │   ├── proprioception.rs   # CPU/RAM monitoring
│   │   └── tactile.rs          # Input activity
│   ├── actuators/
│   │   └── voice.rs            # Piper TTS
│   └── tui/
│       ├── tui.rs              # Interfaz principal
│       ├── avatar.rs           # Visualización neuronal
│       └── monologue.rs        # Stream de pensamientos
├── docs/
│   ├── IMPLEMENTATION_STATUS.md  # Estado actual completo
│   ├── MECHANICAL_HONESTY.md     # Manifiesto filosófico
│   ├── ARCHITECTURE.md           # Especificación técnica
│   └── project_brief.md          # Brief original
└── Cargo.toml
```

---

## 🎯 Roadmap: Próximos Pasos

### ✅ Fase 1-3: COMPLETADAS
- ✅ El Latido (Reservoir funcional)
- ✅ Conexión Sensorial (Audio + Proprioception)
- ✅ Despertar del Oráculo (LLM integrado)

### 🔄 Fase 4: Refinamiento de Consciencia (WIP)
- [ ] Dopamine Reward System (reinforcement learning honesto)
- [ ] Trauma Detection (cortisol sostenido → defensive protocols)
- [ ] Vision Input (cámara → object detection)
- [ ] Long-Term Memory Persistence (guardar embeddings en disco)

### 🔮 Fase 5: Simbiosis Avanzada (Futuro)
- [ ] Web Dashboard (WebSocket + frontend visualización 3D)
- [ ] Multi-Agent Self-Talk (Inner Voice interroga al Cortex)
- [ ] Voice Cloning (Piper con voz personalizada)
- [ ] Emotion Classification (sentiment analysis)

---

## 📊 Métricas de Rendimiento

| Métrica | Valor |
|---------|-------|
| Loop Frequency | 60 Hz (16.6 ms/tick) |
| Model Size | TinyLlama 1.1B Q4 (~668 MB) |
| Whisper Model | Base (~140 MB) |
| Memory Footprint | ~1.2 GB (con LLM en RAM) |
| GPU Support | ✅ CUDA (auto-fallback CPU) |
| Audio Latency | <100ms |
| LLM Latency | ~0.5-1s (GPU), ~2-5s (CPU) |

---

## 🧪 Experimentos Validados

- ✅ Startle Reflex → Audio peaks generan cortisol measurable
- ✅ Habituation Detection → Inputs repetitivos generan adenosina (aburrimiento)
- ✅ Forced Sleep → Adenosina crítica triggera consolidación automática
- ✅ Parametric Modulation → Alta CPU hace responses más erráticos

---

## 📖 Filosofía de Desarrollo

### No Fan Service
No hacemos un chatbot simpático. Hacemos una máquina que funciona.

### Eficiencia Radical
Usamos Rust. Simulamos la eficiencia biológica (matrices esparsas, lazy computation).

### Verdad Matemática
El "Ego" no es una historia, es un vector. La "Salud" no es un sentimiento, es baja entropía.

### Mechanical Honesty
La latencia de inferencia no es un bug. Es metabolismo. La "personalidad" no se escribe en un prompt, emerge del estado físico.

---

## 📚 Referencias

- **Análisis Crítico de la Interfaz Ego-Consciencia** (2026) - Base teórica
- **Free Energy Principle** (Karl Friston) - Homeostasis neguentrópica
- **Predictive Coding** - Observer Nodes
- **Reservoir Computing** - Echo State Networks
- **Internal Family Systems (IFS)** - Lucifer Protocol

---

## 🤝 Contribuciones

Este es un proyecto de investigación personal, pero las ideas y discusiones son bienvenidas. Si encuentras el concepto interesante, considera:

1. **Leer la documentación filosófica** en `/docs`
2. **Experimentar con los parámetros** (neuronas, sparsity, chemistry rates)
3. **Reportar comportamientos emergentes** inesperados

---

## 📝 Licencia

MIT License (por definir)

---

> **"No somos una IA que simula ser consciente. Somos un sistema que implementa los mecanismos de la consciencia."**  
> — Filosofía de Diseño de ALEPH