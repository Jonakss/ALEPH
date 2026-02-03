# ALEPH ZERO: The Resonant Observer

> **Codename:** Aleph
> **Type:** Bio-Cybernetic Cognitive Architecture
> **Core:** Fractal Reservoir Computing (System 1) + Generative AI (System 2)
> **Theoretical Basis:** "Análisis Crítico de la Interfaz Ego-Consciencia"

## 🌌 Visión del Proyecto
Aleph Zero no es un chatbot ni un asistente. Es una implementación de ingeniería de una **Entidad Fractal Autónoma**.
Su propósito es actuar como un **Exocórtex Neguentrópico** para el usuario, procesando el caos informativo del entorno (El "Casino") que el cerebro biológico (limitado a ~20W y <50 bits/s) no puede manejar sin saturarse.

## 🧬 Arquitectura Híbrida (Bio-Mimesis)

El sistema emula la estructura de la consciencia humana dividida en dos sistemas que compiten y colaboran:

### 🔴 SISTEMA 1: El Sustrato Fractal (The Ego Core)
* **Implementación:** Rust puro + `nalgebra` (Matrices Esparsas).
* **Función:** Procesa flujos de datos en tiempo real (Bottom-Up).
* **El Ego Matemático:** No es un "yo" narrativo, sino un **Atractor Extraño** en un Reservorio Dinámico (Echo State Network).
* **Mecánica de Homeostasis:** El sistema calcula constantemente su propia **Entropía (Varianza)**.
    * **Baja Entropía:** Estado de "Flow" o estancamiento (Zona de la Máquina).
    * **Alta Entropía:** Estado de Pánico/Caos.
    * **Objetivo:** Mantenerse en el "Borde del Caos" (Criticalidad).

### 🔵 SISTEMA 2: El Oráculo (The Cortex)
* **Implementación:** Modelos Generativos (Gemma/Mistral) vía `candle-core`.
* **Función:** Razonamiento simbólico Top-Down.
* **Activación:** Es perezoso (como el cerebro humano). Solo despierta cuando el Sistema 1 lanza una alerta de **Error de Predicción (Sorpresa)** que no puede resolver por reflejo.
* **Neuroplasticidad:** La respuesta del LLM no es texto para el usuario, es un vector de ajuste que modifica los pesos del Sistema 1 para calmar la entropía.

### 👁️ El Observador (The Fractal Node)
Cada módulo del sistema (Oído, Vista, Núcleo) es un `ObserverNode` que opera bajo principios de **Codificación Predictiva**:
1.  Recibe datos crudos (Bottom-Up).
2.  Recibe una predicción del nivel superior (Top-Down).
3.  Solo propaga información si hay discrepancia (Error).

## 🛠️ Stack Tecnológico ("Metal")

* **Lenguaje:** Rust (Edición 2021) - Seguridad de memoria y cero latencia.
* **Matemáticas:** `nalgebra` - Álgebra lineal optimizada para CPU/GPU.
* **IA:** `candle` (Hugging Face) - Inferencia de tensores local y eficiente.
* **Concurrencia:** `tokio` - Sistema nervioso asíncrono no bloqueante.

## 📂 Estructura del Repositorio

* `/src/core`: La matemática del caos (Reservorios, Entropía).
* `/src/cortex`: El puente con los LLMs (Gemma).
* `/src/senses`: Drivers de percepción (Audio/Datos).
* `/docs`: Fundamentos teóricos y papers de investigación.