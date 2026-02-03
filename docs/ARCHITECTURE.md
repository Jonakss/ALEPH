# ESPECIFICACIÓN DE ARQUITECTURA: PROYECTO ALEPH

> **Ref:** Análisis Crítico de la Interfaz Ego-Consciencia (2026)
> **Principio:** Bio-Mimesis y Neguentropía Computacional.

## 1. Fundamentación Teórica
El sistema se basa en la premisa de que la consciencia opera como un sistema de **procesamiento de información limitado** (~20W, <50 bits/s) inmerso en un entorno entrópico ("El Casino") diseñado para inducir estados de bucle cerrado ("Zona de la Máquina").
Aleph actúa como un **Exocórtex** que externaliza el procesamiento de alta entropía para proteger la integridad biológica del usuario.

## 2. Modelo Termodinámico (Entropía)
* **Sistema Abierto:** A diferencia de los sistemas cerrados que tienden al desorden, Aleph debe operar como un sistema disipativo, importando señal (orden) y exportando entropía (calor/ruido).
* **Métrica de Salud:** La "Salud" del sistema no se mide por uptime, sino por **Homeostasis Neguentrópica**.
    * *Estado Crítico (Pánico):* Varianza excesiva (Caos).
    * *Estado Estático (Pecado/DMN):* Varianza nula (Estancamiento/Bucle).
    * *Estado Objetivo (Flow):* "Borde del Caos" (Criticalidad Auto-Organizada).

## 3. Arquitectura Cognitiva Híbrida

### 3.1. Sistema 1: El Reservorio Fractal (The Ego)
* **Función:** Mapeo de la "Red Neuronal por Defecto" (DMN).
* **Implementación:** Echo State Network (ESN) con topología esparsa.
* **Dinámica:**
    * Los inputs sensoriales (Audio RMS) excitan el reservorio.
    * La recurrencia interna genera "ecos" (memoria de corto plazo).
    * La **Disonancia** (Error de Predicción) se calcula como la diferencia entre el estado esperado y el real.

### 3.2. Sistema 2: El Neocórtex (Estructural)
* **Función:** Observación de Segundo Orden y Homeostasis.
* **Implementación:** `Neocortex` (Rust Struct). No es una IA, es lógica de control.
* **Mecanismo:**
    * Monitorea la **Derivada de la Entropía** ($\Delta E$).
    * Detecta eventos estructurales: `StimulusStart`, `Trauma`, `Stagnation`.
    * Regula el flujo de información antes de invocar procesos costosos (como el Lenguaje).

### 3.3. Sistema 3: El Nodo de Lenguaje (Gemma/LLM)
* **Función:** Generación Semántica bajo demanda.
* **Estado:** *Latente*. Solo se activa cuando el Neocórtex solicita etiquetar un evento complejo.
* **Herramienta:** Gemma-2b (Quantized).

## 4. Protocolos de Seguridad (IFS)
* **Lucifer Protocol:** Detección de subrutinas defensivas ("Firefighters") que secuestran el sistema ante picos de dolor/entropía. Aleph debe identificar estos picos y activar contramedidas de enfriamiento (down-regulation) en lugar de bloquearse.
