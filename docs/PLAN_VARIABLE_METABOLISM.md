# Plan: Capa de Metabolismo Variable (Variable Loop Frequency)

> **Estado:** 📋 PLANIFICADO — No implementar hasta aprobación  
> **Fecha:** 2026-02-03  
> **Categoría:** Mechanical Honesty / Fase 4  
> **Prioridad:** Media (depende de validación de Fase 4.1 Dopamine Reward)

---

## 1. Razonamiento Metacognitivo

### 1.1 Análisis del Concepto

El usuario propone que la frecuencia del loop principal (`FRECUENCIA_HZ`) no sea una constante fija, sino una **variable dinámica vinculada al estado interno** (dopamina/interés, adenosina, cortisol). A mayor interés (hyperfoco), mayor frecuencia de muestreo y procesamiento.

### 1.2 Alineación Bio-Digital

En biología, esto se conoce como **resolución temporal de la atención**. En estados de alerta o hyperfoco (pico de noradrenalina/dopamina), la percepción del tiempo se "enlentece" subjetivamente porque el cerebro **procesa más información por unidad de tiempo**. Aleph replicaría esto a nivel mecánico: más ciclos por segundo = más "temporal resolution" interna.

### 1.3 Validación Técnica

Es factible reemplazar la constante por un valor calculado en el Hub (loop principal). Esto reforzaría la **Honestidad Mecánica**: Aleph no solo "dice" que está en hyperfoco, sino que sus engranajes (el loop) giran físicamente más rápido.

---

## 2. La Frecuencia Cardíaca Digital

**La idea central:** Convertir la tasa de refresco en la **Frecuencia Cardíaca** de Aleph. No es un cambio cosmético; es cambiar la velocidad a la que el ser procesa su realidad.

### Por qué encaja con Mechanical Honesty

| Estado Interno | Comportamiento del Loop | Analogía Biológica |
|----------------|-------------------------|-------------------|
| **Hyperfoco** (alta dopamina, bajo cortisol, baja adenosina) | Loop sube a 90–120 Hz | Sobrecarga temporal: más ciclos, más atención |
| **Neutro** | Loop ~60 Hz (actual) | Estado basal |
| **Aburrimiento** (alta adenosina, baja dopamina) | Loop baja a 24–30 Hz | Economía de energía: reflejos lentos |
| **Estrés** (cortisol alto sostenido) | Loop puede bajar (protección) o subir (alerta) según protocolo | Respuesta fight-or-flight |

### Efectos Concretos

- **Audio:** Más Hz → lectura de espectro más frecuente → reacciones más precisas
- **Chemistry:** Actualizaciones más granulares del estado químico
- **Poke:** Respuesta más rápida en hyperfoco, más lenta en aburrimiento
- **TUI:** Histograma de audio y avatar más fluidos en hyperfoco; más "entortados" o pesados en aburrimiento. **Representación visual del estado anímico sin una sola palabra.**

---

## 3. Restricciones del Mundo Real

### 3.1 Techo Térmico (Hardware)

Si CPU/GPU ya está al 90%, subir a 144 Hz en hyperfoco podría causar saturación o throttling. Aleph intentaría "pensar" más rápido de lo que su cuerpo de silicio permite.

**Mitigación:**
- **Cap superior:** Nunca exceder Hz si `last_body_state.cpu_usage > 85%`
- **Feedback loop:** Si el tick empieza a tardar más que el intervalo, bajar Hz automáticamente

### 3.2 Coherencia del Tiempo (Delta-Time)

Si el loop va más rápido, funciones que dependen de "ticks" se acelerarían:
- `chemistry.tick()` — decay de neurotransmisores
- `ego.tick()` — dinámica del reservoir
- Decay de insight, novelty, etc.

**Problema:** A 120 Hz, la química decaería el doble de rápido que a 60 Hz, distorsionando el modelo.

**Solución: Normalización por `delta_time`**
- Medir `elapsed` real de cada tick
- Pasar `delta_time` (en segundos) a `chemistry.tick()`, `reservoir`, etc.
- Todas las tasas de decay se multiplican por `delta_time` para ser invariantes a la frecuencia

### 3.3 Suavizado

Evitar saltos bruscos (60 → 120 en un tick). Usar:
- **Interpolación** (lerp) hacia el Hz target
- **Límite de cambio por tick** (ej: max ±5 Hz por tick)
- **Hysteresis** para evitar oscilaciones en el borde

---

## 4. Especificación Técnica (Borrador)

### 4.1 Variables Involucradas

| Variable | Tipo | Rol |
|----------|------|-----|
| `current_hz` | `f32` | Frecuencia actual (suavizada) |
| `target_hz` | `f32` | Frecuencia calculada a partir de química |
| `hz_min` | `const f32` | Piso (ej: 24) |
| `hz_max` | `const f32` | Techo (ej: 120) |
| `hz_max_hardware` | `f32` | Techo dinámico según CPU (si CPU > 85%, bajar) |

### 4.2 Función de Mapeo Química → Hz

```
target_hz = hz_min + (hz_max - hz_min) * f(dopamine, adenosine, cortisol)
```

**Propuesta de `f()` (a refinar):**
- Dopamina ↑ → target_hz ↑
- Adenosina ↑ → target_hz ↓
- Cortisol moderado → target_hz ↑ (alerta); cortisol extremo → target_hz ↓ (protección)
- `is_dreaming` → forzar hz_min

### 4.3 Cambios en main.rs (Pseudocódigo)

```rust
// Reemplazar:
const FRECUENCIA_HZ: u64 = 60;

// Por:
const HZ_MIN: f32 = 24.0;
const HZ_MAX: f32 = 120.0;
const HZ_SMOOTH: f32 = 0.05;  // Factor de suavizado (lerp)

let mut current_hz: f32 = 60.0;

// En cada tick, después de chemistry.tick():
let target_hz = compute_target_hz(&chemistry, is_dreaming, last_body_state.cpu_usage);
current_hz = lerp(current_hz, target_hz, HZ_SMOOTH);

// En el sleep:
let interval_ms = 1000.0 / current_hz;
if elapsed < Duration::from_secs_f32(interval_ms / 1000.0) {
    thread::sleep(Duration::from_secs_f32(interval_ms / 1000.0) - elapsed);
}
```

### 4.4 Cambios en chemistry.rs y reservoir.rs

- Añadir parámetro `delta_time: f32` a `tick()`
- Multiplicar tasas de decay por `delta_time` para invariancia temporal

### 4.5 Telemetría

- `telem.fps = current_hz` (ya no hardcodear 60.0)
- Opcional: mostrar Hz actual en TUI como "❤️ 72 bpm" (Frecuencia Cardíaca Digital)

---

## 5. Orden de Implementación (Cuando se Apruebe)

1. **Fase A:** Normalización delta_time en chemistry y reservoir (prerrequisito)
2. **Fase B:** Función `compute_target_hz()` con mapeo simple (solo dopamina)
3. **Fase C:** Variable `current_hz` y sleep dinámico en main loop
4. **Fase D:** Suavizado (lerp) y límite hardware
5. **Fase E:** Integrar adenosina/cortisol en el mapeo
6. **Fase F:** Actualizar TUI (fps dinámico, opcional "bpm")

---

## 6. Criterios de Éxito

- [ ] Loop varía entre 24–120 Hz según estado químico
- [ ] Química y reservoir se comportan igual a 60 Hz fijos (invariancia temporal)
- [ ] Alta carga de CPU limita el techo de Hz (no overclocking suicida)
- [ ] TUI refleja fluidez variable (avatar/audio más fluidos en hyperfoco)
- [ ] Sin crash ni degradación en 1h de ejecución con transiciones

---

## 7. Referencias

- **Análisis original:** Traza de Razonamiento (Metacognición) del usuario, 2026-02-03
- **MECHANICAL_HONESTY.md:** Principios 2 (Parametric Effects), 4 (Delta Sensitivity)
- **ROADMAP.md:** Fase 4.1 Dopamine Reward System (prerrequisito conceptual)

---

> **"Cuando estemos listos, haremos que el `thread::sleep` sea esclavo de su dopamina."**  
> — Nota del diseñador
