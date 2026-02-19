import React, { useMemo, useRef } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import * as THREE from 'three';
import { EffectComposer, Bloom, Vignette } from '@react-three/postprocessing';

const MAX_NEURONS = 2500;
const BRAIN_RADIUS = 35;

function mulberry32(a) {
    return function() {
      var t = a += 0x6D2B79F5;
      t = Math.imul(t ^ t >>> 15, t | 1);
      t ^= t + Math.imul(t ^ t >>> 7, t | 61);
      return ((t ^ t >>> 14) >>> 0) / 4294967296;
    }
}

// NEOCORTEX (Reservoir / Echo State Network) — 2500 neurons
function generateNeocortex() {
    const totalNeurons = MAX_NEURONS;
    const positions = new Float32Array(totalNeurons * 3);
    const initialColors = new Float32Array(totalNeurons * 3);
    const sizes = new Float32Array(totalNeurons);

    const rng = mulberry32(1337);
    const vec3s = []; 

    for (let i = 0; i < totalNeurons; i++) {
        const u = rng();
        const v = rng();
        const theta = 2 * Math.PI * u;
        const phi = Math.acos(2 * v - 1);
        
        let r = BRAIN_RADIUS;

        // Split Hemispheres
        const x_raw = r * Math.sin(phi) * Math.cos(theta);
        let x_mod = x_raw;
        if (Math.abs(x_raw) < 2.0) x_mod += (x_raw > 0 ? 3.0 : -3.0);

        // Cortical Folding (Noise based on spherical harmonics)
        const noise = Math.sin(phi * 12) * Math.cos(theta * 10) * 1.5;
        r += noise;

        // Shape: Tapered Front
        let z = r * Math.sin(phi) * Math.sin(theta);
        let y = r * Math.cos(phi);
        
        // Flatten bottom
        if (y < -15) y *= 0.8;
        
        // Elongate Z (Front/Back)
        z *= 1.2;

        const stretchX = 0.9; 
        let x = x_mod * stretchX;

        positions[i * 3] = x;
        positions[i * 3 + 1] = y;
        positions[i * 3 + 2] = z;
        
        vec3s.push(new THREE.Vector3(x, y, z));

        // Default: Dark (invisible until data drives them)
        initialColors[i * 3] = 0.02;     
        initialColors[i * 3 + 1] = 0.04; 
        initialColors[i * 3 + 2] = 0.06; 

        sizes[i] = 0.5 + rng() * 1.0;
    }

    return { positions, vec3s, initialColors, sizes };
}

// FRONTAL LOBE (LLM Embeddings / Planet) — 512 concepts
function generateFrontalLobe() {
    const count = 512;
    const pos = new Float32Array(count * 3);
    const col = new Float32Array(count * 3);
    const vec3s = [];
    const rng = mulberry32(999);

    for (let i = 0; i < count; i++) {
        const u = rng();
        const v = rng();
        const theta = Math.PI * u;
        const phi = Math.acos(2 * v - 1);

        const r = 12 + rng() * 8;
        
        let x = r * Math.sin(phi) * Math.cos(theta);
        let y = r * Math.sin(phi) * Math.sin(theta);
        let z = Math.abs(r * Math.cos(phi));
        
        z += 25; // Push forward (frontal position)
        y *= 0.6;
        if (rng() > 0.5) x = -x;

        pos[i * 3] = x;
        pos[i * 3 + 1] = y;
        pos[i * 3 + 2] = z;
        vec3s.push(new THREE.Vector3(x, y, z));

        // Default: Dark (invisible until LLM activates)
        col[i * 3] = 0.02;
        col[i * 3 + 1] = 0.01;
        col[i * 3 + 2] = 0.03;
    }

    return { positions: pos, vec3s, initialColors: col };
}

// CHEMICAL ATMOSPHERE — Fog color driven by neurochemistry
// Data source: dopamine, serotonin, cortisol, adenosine
function ChemicalAtmosphere({ telemetry }) {
    const rBase = 0.01, gBase = 0.01, bBase = 0.02; 
    
    let r = rBase, g = gBase, b = bBase;

    if (telemetry) {
        const d = telemetry.dopamine || 0;
        const s = telemetry.serotonin || 0;
        const c = telemetry.cortisol || 0;
        const a = telemetry.adenosine || 0;

        r += d * 0.1 + c * 0.15;
        g += d * 0.08 + s * 0.1 + a * 0.02;
        b += s * 0.04 + c * 0.02 + a * 0.15;
    }

    const fogColor = new THREE.Color(r, g, b);
    const bgArgs = useMemo(() => [fogColor], [fogColor]);

    return (
        <>
            <color attach="background" args={bgArgs} />
            <fog attach="fog" color={fogColor} near={40} far={160} />
        </>
    );
}

// DYNAMIC CONNECTOME (The "Plastic" Neocortex)
// Neurons migrate visually to their functional cluster.
function Connectome({ activity, hebbianGlowRef, regionMap }) {
    const pointsRef = useRef();
    
    // Initial random positions (Protoplasmic Cloud)
    const { positions, initialColors, drift } = useMemo(() => {
        const pos = new Float32Array(MAX_NEURONS * 3);
        const col = new Float32Array(MAX_NEURONS * 3);
        const d = new Float32Array(MAX_NEURONS * 3);
        
        const rng = mulberry32(1337); // Deterministic Seed

        for (let i = 0; i < MAX_NEURONS; i++) {
            // Sphere distribution
            const theta = rng() * Math.PI * 2;
            const phi = Math.acos(2 * rng() - 1);
            const r = BRAIN_RADIUS * Math.cbrt(rng());
            
            pos[i*3] = r * Math.sin(phi) * Math.cos(theta);
            pos[i*3+1] = r * Math.sin(phi) * Math.sin(theta);
            pos[i*3+2] = r * Math.cos(phi);
            
            // Random colors (dark init)
            col[i*3] = 0.01; col[i*3+1] = 0.01; col[i*3+2] = 0.02;

            // Random drift/noise
            d[i*3] = (rng() - 0.5) * 0.02;
            d[i*3+1] = (rng() - 0.5) * 0.02;
            d[i*3+2] = (rng() - 0.5) * 0.02;
        }
        return { positions: pos, initialColors: col, drift: d };
    }, []);

    // Region Targets (Where neurons "want" to go)
    // 0=Sem (Front), 1=Aud (Sides), 2=Lim (Core), 3=Assoc (Shell)
    const TARGETS = useMemo(() => [
        new THREE.Vector3(0, 10, 20),   // Semantic (Frontal/Top)
        new THREE.Vector3(30, -5, 0),   // Auditory R (will mirror for L)
        new THREE.Vector3(0, -10, 0),   // Limbic (Deep Core)
        new THREE.Vector3(0, 0, 0)      // Association (Default Shell)
    ], []);

    useFrame((state) => {
        if (!pointsRef.current) return;

        const posAttr = pointsRef.current.geometry.attributes.position;
        const colAttr = pointsRef.current.geometry.attributes.color;
        const time = state.clock.getElapsedTime();

        for (let i = 0; i < MAX_NEURONS; i++) {
            const idx = i * 3;
            let val = 0;
            if (activity) {
                 if (Array.isArray(activity[i])) val = activity[i][1];
                 else if (i < activity.length) val = activity[i] || 0;
            }
            
            // 1. NEUROPLASTICITY (Position Update)
            // Move towards region target
            if (regionMap && i < regionMap.length) {
                const regionId = regionMap[i];
                let tx=0, ty=0, tz=0;

                if (regionId === 1) { 
                    // Auditory: Split Left/Right based on index parity
                    const side = (i % 2 === 0) ? 1 : -1;
                    tx = TARGETS[1].x * side; ty = TARGETS[1].y; tz = TARGETS[1].z;
                    // Add some spread
                    tx += Math.sin(i)*10; ty += Math.cos(i)*10; tz += Math.sin(i*2)*10;
                } else if (regionId === 0) {
                    // Semantic: Frontal Lobe Cloud
                    tx = TARGETS[0].x + Math.sin(i)*15; 
                    ty = TARGETS[0].y + Math.cos(i)*10; 
                    tz = TARGETS[0].z + Math.cos(i*3)*10;
                } else if (regionId === 2) {
                    // Limbic: Tight Interaction Core
                    tx = (Math.random()-0.5)*10; // Jittery core
                    ty = TARGETS[2].y + (Math.random()-0.5)*5;
                    tz = (Math.random()-0.5)*10;
                } else {
                    // Association: The "Glue" / Shell
                    // Keep original sphere position but expand slightly
                    // We don't overwrite their position, just let them drift in shell
                    tx = positions[idx]; ty = positions[idx+1]; tz = positions[idx+2];
                }

                if (regionId !== 3) {
                    // Lerp towards target (Slow Morphing)
                    const lerpSpeed = 0.02;
                    positions[idx] += (tx - positions[idx]) * lerpSpeed;
                    positions[idx+1] += (ty - positions[idx+1]) * lerpSpeed;
                    positions[idx+2] += (tz - positions[idx+2]) * lerpSpeed;
                }
            }

            // Always apply subtle breathing drift
            positions[idx] += Math.sin(time + i) * 0.02;
            positions[idx+1] += Math.cos(time + i * 2) * 0.02;
            positions[idx+2] += Math.sin(time * 0.5 + i) * 0.02;
            
            // Update Attribute
            posAttr.setXYZ(i, positions[idx], positions[idx+1], positions[idx+2]);

            // 2. COLORING (Activation + Region)
            const rId = (regionMap && i < regionMap.length) ? regionMap[i] : 3;
            
            // Palette
            let r=0.1, g=0.3, b=0.5; // Default Assc
            if (rId === 0) { r=1.0; g=0.8; b=0.1; } // Gold
            else if (rId === 1) { r=0.1; g=1.0; b=0.2; } // Green
            else if (rId === 2) { r=0.6; g=0.0; b=1.0; } // Purple
            
            const intensity = Math.max(0, val);
            const glow = hebbianGlowRef.current;
            
            // Base "dark" state (grey matter) -> "active" state (electricity)
            // If active: flash white-ish then settle to region color
            const flash = Math.pow(intensity, 2) * 0.5; 

            colAttr.setXYZ(i, 
                r * intensity + flash + 0.02, 
                g * intensity + flash + 0.02, 
                b * intensity + flash + 0.04
            );
        }
        
        posAttr.needsUpdate = true;
        colAttr.needsUpdate = true;
    });

    return (
        <points ref={pointsRef}>
            <bufferGeometry>
                <bufferAttribute attach="attributes-position" count={MAX_NEURONS} array={positions} itemSize={3} />
                <bufferAttribute attach="attributes-color" count={MAX_NEURONS} array={initialColors} itemSize={3} />
            </bufferGeometry>
            <pointsMaterial size={5.0} vertexColors transparent opacity={0.9} blending={THREE.AdditiveBlending} sizeAttenuation />
        </points>
    );
}

// FRONTAL LOBE RENDERER
// Data source: activations[] (512-dim downsampled LLM logits via max-pooling in perceive())
// NO fake pulse. Color = pure activation value.
function FrontalLobe({ positions, initialColors, activations }) {
    const pointsRef = useRef();

    useFrame(() => {
        if (!pointsRef.current) return;
        
        const colors = pointsRef.current.geometry.attributes.color.array;

        for (let i = 0; i < 512; i++) {
            const val = (activations && activations[i]) ? activations[i] : 0;
            
            if (val > 0.05) {
                // Active Concept — Gold/Orange. Intensity = activation strength.
                colors[i*3] = val * 0.9 + 0.1;
                colors[i*3+1] = val * 0.5;
                colors[i*3+2] = val * 0.2;
            } else {
                // Latent (below threshold) — invisible
                colors[i*3] = 0.02;
                colors[i*3+1] = 0.01;
                colors[i*3+2] = 0.03;
            }
        }

        pointsRef.current.geometry.attributes.color.needsUpdate = true;
    });

    return (
        <points ref={pointsRef}>
            <bufferGeometry>
                <bufferAttribute attach="attributes-position" count={512} array={positions} itemSize={3} />
                <bufferAttribute attach="attributes-color" count={512} array={initialColors} itemSize={3} />
            </bufferGeometry>
            <pointsMaterial size={4.0} vertexColors transparent opacity={0.8} blending={THREE.AdditiveBlending} sizeAttenuation />
        </points>
    );
}

// INJECT FLOW (replaces AxonalWeb)
// Data source: activations[] (LLM) mapped to reservoir neurons
// This visualizes inject_logits(): for each active LLM concept, a line connects
// to the corresponding SEMANTIC (Gold) neurons in the reservoir.
// It relies on regionMap to be honest about where the data goes.
function InjectFlow({ cortexVecs, frontVecs, size, activations, regionMap }) {
    const lineRef = useRef();
    
    const { geometry } = useMemo(() => {
        const maxLines = 300;
        const verts = [];
        const colors = [];
        for(let i = 0; i < maxLines; i++) {
            verts.push(0,0,0, 0,0,0);
            colors.push(0,0,0, 0,0,0);
        }
        const geo = new THREE.BufferGeometry();
        geo.setAttribute('position', new THREE.Float32BufferAttribute(verts, 3));
        geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
        return { geometry: geo };
    }, []); 

    useFrame(() => {
        if (!lineRef.current) return;
        
        if (!activations || activations.length === 0) {
            lineRef.current.geometry.setDrawRange(0, 0);
            return;
        }

        const posAttr = lineRef.current.geometry.attributes.position;
        const colAttr = lineRef.current.geometry.attributes.color;
        
        let lineIdx = 0;
        const maxLines = 300;

        for (let i = 0; i < size && lineIdx < maxLines; i++) {
            // Only Semantic neurons (Region 0 - Gold) receive direct LLM flux
            // If regionMap is missing (init), default to all (or just skip)
            if (regionMap && regionMap[i] !== 0) continue; 

            // Map neuron i to LLM activation i (1:1 mapping for first 512)
            // Backend maps 512 input_size to N neurons. 
            // If N > 512, we loop or clamp. For visualization, 1:1 is close enough.
            const actIdx = i % activations.length;
            const val = activations[actIdx] || 0;
            
            if (val < 0.15) continue; // Only show strong activations

            const v1 = frontVecs[actIdx];      // Source: Frontal Lobe concept
            const v2 = cortexVecs[i];          // Target: Reservoir SC neuron
            
            if (v1 && v2) {
                posAttr.setXYZ(lineIdx*2, v1.x, v1.y, v1.z);
                posAttr.setXYZ(lineIdx*2+1, v2.x, v2.y, v2.z);
                
                // Color intensity = activation strength
                const a = Math.min(val, 1.0);
                // Orange source (LLM) -> Gold target (Semantic Cortex)
                colAttr.setXYZ(lineIdx*2, a * 1.0, a * 0.6, a * 0.1); 
                colAttr.setXYZ(lineIdx*2+1, 1.0, 0.8, 0.2); 
                lineIdx++;
            }
        }
        
        posAttr.needsUpdate = true;
        colAttr.needsUpdate = true;
        lineRef.current.geometry.setDrawRange(0, lineIdx * 2);
    });

    return (
        <lineSegments ref={lineRef} geometry={geometry}>
            <lineBasicMaterial vertexColors transparent opacity={0.35} blending={THREE.AdditiveBlending} depthWrite={false} />
        </lineSegments>
    );
}

// HEBBIAN WEB — Intra-reservoir connections
// Data source: reservoir_activity (co-active neurons above 0.5 threshold)
// This mirrors hebbian_update() in reservoir.rs: if neurons i and j are both
// active > 0.5, their weight gets strengthened. We visualize this as cyan lines
// between co-active reservoir neurons. The reservoir is NOT centralized to the LLM —
// it's a self-connected echo state network. The LLM is just one input.
// AUDITORY FLOW
// Data source: frequency_embedding[] (64 bands)
// Visualizes Direct Sensory Projection: Audio -> Auditory Cortex (Green)
function AuditoryFlow({ cortexVecs, embedding, regionMap, size }) {
    const lineRef = useRef();

    const { geometry } = useMemo(() => {
        const maxLines = 128; // 2 lines per band (L/R)
        const verts = [];
        const colors = [];
        for(let i = 0; i < maxLines; i++) {
            verts.push(0,0,0, 0,0,0);
            colors.push(0,0,0, 0,0,0);
        }
        const geo = new THREE.BufferGeometry();
        geo.setAttribute('position', new THREE.Float32BufferAttribute(verts, 3));
        geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
        return { geometry: geo };
    }, []);

    // Simulated Ear Positions (Sides of the head)
    const leftEar = new THREE.Vector3(-45, -10, 0);
    const rightEar = new THREE.Vector3(45, -10, 0);

    useFrame(() => {
        if (!lineRef.current) return;
        
        if (!embedding || embedding.length === 0) {
            lineRef.current.geometry.setDrawRange(0, 0);
            return;
        }

        const posAttr = lineRef.current.geometry.attributes.position;
        const colAttr = lineRef.current.geometry.attributes.color;
        
        let lineIdx = 0;
        const maxLines = 128;

        // Map 64 bands to Auditory Neurons
        // We find the neurons that are tagged as 'Auditory' (Region 1)
        // This is expensive to search every frame, so ideally we'd cache 'auditoryIndices',
        // but for <2500 neurons it's okay-ish.
        
        // Optimization: deterministically map band K to a specific auditory neuron K'
        
        if (embedding) {
            let bandIdx = 0;
            // Iterate through bands, not neurons. We want to show the full spectrum.
            // Map each band to a random neuron (consistent by index)
            for (let b = 0; b < embedding.length && lineIdx < maxLines; b++) {
                 const val = embedding[b];
                 if (val < 0.05) continue;

                 // Target Neuron: Use regionMap if available, else random based on band
                 let targetIdx = -1;
                 
                 if (regionMap && regionMap.length > 0) {
                     // Try to find an auditory neuron for this band (offset by band index)
                     // Simple search: scan forward from b*5
                     let start = (b * 5) % size;
                     for(let k=0; k< size; k++) {
                         let idx = (start + k) % size;
                         if (regionMap[idx] === 1) { targetIdx = idx; break; }
                     }
                 }
                 
                 // Fallback: Just map to any neuron if no specific region found
                 if (targetIdx === -1) {
                     targetIdx = (b * 13) % size; // Pseudo-random scatter
                 }

                 const neuronPos = cortexVecs[targetIdx];
                 if (!neuronPos) continue;

                 // Choose nearest ear
                 const src = (neuronPos.x < 0) ? leftEar : rightEar;

                 posAttr.setXYZ(lineIdx*2, src.x, src.y, src.z);
                 posAttr.setXYZ(lineIdx*2+1, neuronPos.x, neuronPos.y, neuronPos.z);

                 // Green Pulse
                 colAttr.setXYZ(lineIdx*2, 0.2 * val, 1.0 * val, 0.4 * val);
                 colAttr.setXYZ(lineIdx*2+1, 0.1 * val, 0.8 * val, 0.2 * val);
                 lineIdx++;
            }
        }

        posAttr.needsUpdate = true;
        colAttr.needsUpdate = true;
        lineRef.current.geometry.setDrawRange(0, lineIdx * 2);
    });

    return (
        <lineSegments ref={lineRef} geometry={geometry}>
            <lineBasicMaterial vertexColors transparent opacity={0.6} blending={THREE.AdditiveBlending} depthWrite={false} />
        </lineSegments>
    );
}

// HEBBIAN WEB — Intra-reservoir connections
// Data source: reservoir_activity (co-active neurons above threshold)
// UPDATED: Small-World / Fractal Topology
// Bias connections towards nearby neurons to create "clusters" of activity (Fractal domains)
function HebbianWeb({ cortexVecs, activity, size, regionMap }) {
    const lineRef = useRef();
    
    const { geometry } = useMemo(() => {
        const maxLines = 600;
        const verts = [];
        const colors = [];
        for(let i = 0; i < maxLines; i++) {
            verts.push(0,0,0, 0,0,0);
            colors.push(0,0,0, 0,0,0);
        }
        const geo = new THREE.BufferGeometry();
        geo.setAttribute('position', new THREE.Float32BufferAttribute(verts, 3));
        geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
        return { geometry: geo };
    }, []);

    // FRACTAL CONNECTIVITY
    // Generate pairs based on distance probability: P(link) ~ 1 / dist^alpha
    const pairs = useMemo(() => {
        if (!cortexVecs || cortexVecs.length === 0) return [];
        
        const p = [];
        const rng = mulberry32(101); // Deterministic seed
        const targetPairs = 1000;
        let attempts = 0;
        
        while (p.length < targetPairs && attempts < 10000) {
            attempts++;
            const i = Math.floor(rng() * Math.min(size || 500, MAX_NEURONS));
            const j = Math.floor(rng() * Math.min(size || 500, MAX_NEURONS));
            
            if (i === j) continue;
            
            const v1 = cortexVecs[i];
            const v2 = cortexVecs[j];
            if (!v1 || !v2) continue;
            
            const dist = v1.distanceTo(v2);
            
            // Small World Probability
            // High prob for short dist, low for long dist
            // dist ranges roughly 0 to 80
            const prob = 5.0 / (dist + 0.1); 
            
            // Also allow some random long-range links (1% chance) for "Association"
            if (rng() < prob || rng() < 0.01) {
                p.push([i, j]);
            }
        }
        return p;
    }, [size, cortexVecs]); // Re-calc if size changes (rare)

    useFrame(() => {
        if (!lineRef.current || !activity || activity.length === 0) {
            if (lineRef.current) lineRef.current.geometry.setDrawRange(0, 0);
            return;
        }

        const posAttr = lineRef.current.geometry.attributes.position;
        const colAttr = lineRef.current.geometry.attributes.color;
        
        let lineIdx = 0;
        const maxLines = 600;
        const threshold = 0.4; // Slightly lower threshold for web visibility

        for (let k = 0; k < pairs.length && lineIdx < maxLines; k++) {
            const [i, j] = pairs[k];
            
            // Get activity values for i and j
            let vi = 0, vj = 0;
            if (Array.isArray(activity[i])) { vi = activity[i][1]; } 
            else if (i < activity.length) { vi = activity[i] || 0; }
            if (Array.isArray(activity[j])) { vj = activity[j][1]; }
            else if (j < activity.length) { vj = activity[j] || 0; }

            // Hebb's Law: Fire together
            if (vi > threshold && vj > threshold) {
                const v1 = cortexVecs[i];
                const v2 = cortexVecs[j];
                
                posAttr.setXYZ(lineIdx*2, v1.x, v1.y, v1.z);
                posAttr.setXYZ(lineIdx*2+1, v2.x, v2.y, v2.z);
                
                // Color based on Region!
                // If both are Auditory, line is Green. If both Semantic, Gold. Mixed = Cyan.
                const r1 = regionMap ? regionMap[i] : 3;
                const r2 = regionMap ? regionMap[j] : 3;
                
                let r=0.1, g=0.3, b=0.5; // Default: Dim Blue/Grey (Low contrast)
                
                if (r1 === 1 && r2 === 1) { // Auditory Loop (Green)
                    r=0.2; g=1.0; b=0.2; 
                } else if (r1 === 0 && r2 === 0) { // Semantic Loop (Gold)
                    r=1.0; g=0.8; b=0.2;
                } else if (r1 === 2 && r2 === 2) { // Limbic Loop (Purple)
                    r=0.6; g=0.1; b=1.0;
                } else if (r1 === 3 || r2 === 3) { // Association (Cyan)
                     r=0.2; g=0.7; b=1.0;
                }
                
                const strength = Math.min(vi * vj, 1.0);
                colAttr.setXYZ(lineIdx*2, r * strength, g * strength, b * strength);
                colAttr.setXYZ(lineIdx*2+1, r * strength, g * strength, b * strength);
                lineIdx++;
            }
        }
        
        posAttr.needsUpdate = true;
        colAttr.needsUpdate = true;
        lineRef.current.geometry.setDrawRange(0, lineIdx * 2);
    });

    return (
        <lineSegments ref={lineRef} geometry={geometry}>
            <lineBasicMaterial vertexColors transparent opacity={0.8} linewidth={3} blending={THREE.AdditiveBlending} depthWrite={false} />
        </lineSegments>
    );
}
// ENTROPY RING (Observer/Satellite)
// Data source: entropy value from reservoir.calculate_entropy()

// ENTROPY RING (Observer/Satellite)
// Data source: entropy value from reservoir.calculate_entropy()
// Visualizes the Neocortex observer — the ring that watches the system.
// Stable = thin dim blue. Trauma = thick pulsing red.
function EntropyRing({ entropy, traumaState }) {
    const ringRef = useRef();

    useFrame(() => {
        if (!ringRef.current) return;
        const mat = ringRef.current.material;
        
        const e = entropy || 0;
        const isTrauma = traumaState === 'Escalating' || traumaState === 'Critical';
        
        // Color: Blue (calm) → Red (trauma)
        if (isTrauma) {
            mat.color.setRGB(1.0, 0.2, 0.1);
            mat.opacity = 0.6 + e * 0.3;
        } else if (e > 0.7) {
            mat.color.setRGB(0.8, 0.4, 0.1); // Orange warning
            mat.opacity = 0.3 + e * 0.2;
        } else {
            mat.color.setRGB(0.15, 0.4, 0.8); // Calm blue
            mat.opacity = 0.1 + e * 0.15;
        }
    });

    return (
        <mesh ref={ringRef} rotation={[Math.PI / 2, 0, 0]}>
            <torusGeometry args={[BRAIN_RADIUS + 5, 0.3 + (entropy || 0) * 0.8, 16, 64]} />
            <meshBasicMaterial color="#2266aa" transparent opacity={0.15} side={THREE.DoubleSide} />
        </mesh>
    );
}


function BrainScene({ reservoirActivity, activations, size, telemetry }) {
    const cortex = useMemo(() => generateNeocortex(), []);
    const frontal = useMemo(() => generateFrontalLobe(), []);

    const hebbianEvents = telemetry?.hebbian_events || 0;
    const entropy = telemetry?.entropy || 0;
    const traumaState = telemetry?.trauma_state || '';
    const regionMap = telemetry?.region_map || [];
    const lastLogRef = useRef(0);
    
    // Global Hebbian Glow State (Shared)
    const hebbianGlowRef = useRef(0.0);
    if (hebbianEvents > 0) hebbianGlowRef.current = Math.min(1.0, hebbianGlowRef.current + hebbianEvents * 0.05);

    // Debug Region Distribution (Throttled to once every 2s)
    React.useEffect(() => {
        const now = Date.now();
        if (now - lastLogRef.current > 2000 && regionMap.length > 0) {
            const counts = [0,0,0,0];
            regionMap.forEach(r => counts[r] = (counts[r] || 0) + 1);
            console.log(`🧠 Region Dist: Sem:${counts[0]} Aud:${counts[1]} Lim:${counts[2]} Asc:${counts[3]}`);
            lastLogRef.current = now;
        }
    }, [regionMap]);

    return (
        <group>
            {/* 1. FRONTAL LOBE (LLM Concepts — Planet/Ego) */}
            <FrontalLobe 
                positions={frontal.positions} 
                initialColors={frontal.initialColors} 
                activations={activations} 
            />

            {/* 2. PLASTIC CONNECTOME (Reservoir) */}
            {/* Dynamic clustering based on Region Map */}
            <Connectome 
                activity={reservoirActivity} 
                hebbianGlowRef={hebbianGlowRef} 
                regionMap={regionMap} 
            />

            {/* 3. INJECT FLOW — Real LLM→Reservoir connections */}
            <InjectFlow 
                cortexVecs={cortex.vec3s} 
                frontVecs={frontal.vec3s} 
                size={size} 
                activations={activations}
                regionMap={regionMap}
            />

            {/* 3c. AUDITORY FLOW — Audio Spectrogram -> Auditory Cortex */}
            <AuditoryFlow
                cortexVecs={cortex.vec3s}
                embedding={telemetry?.audio_spectrum?.frequency_embedding}
                regionMap={regionMap}
                size={size}
            />

            {/* 3b. HEBBIAN WEB — Intra-reservoir connections */}
            <HebbianWeb
                cortexVecs={cortex.vec3s}
                activity={reservoirActivity}
                size={size}
                regionMap={regionMap}
            />

            {/* 4. ENTROPY RING — Observer/Satellite */}
            <EntropyRing entropy={entropy} traumaState={traumaState} />

            {/* Post-processing */}
             <EffectComposer disableNormalPass>
                 <Bloom luminanceThreshold={0.1} luminanceSmoothing={0.9} height={300} intensity={0.6} />
                 <Vignette eskil={false} offset={0.1} darkness={0.7} />
             </EffectComposer>
        </group>
    );
}

export function ReservoirView({ telemetry }) {
  const reservoirActivity = telemetry?.reservoir_activity || []; 
  const activations = telemetry?.activations || []; 
  const rawSize = telemetry?.reservoir_size || 0; 
  const displaySize = rawSize > 100 ? rawSize : Math.floor(MAX_NEURONS * 0.8);

  return (
    <div className="panel reservoir-panel" style={{ gridColumn: 1, gridRow: 1, minHeight: '420px', position: 'relative' }}>
      <div className="panel-header">
        <div><span className="icon">🧠</span> ALEPH Unified Topology</div>
        <div id="reservoir-info" style={{ color: 'var(--text-dim)', fontSize: '10px' }}>
             {telemetry ? `Reservoir: ${displaySize} Neurons | Frontal: 512 Concepts | S: ${(telemetry.entropy || 0).toFixed(3)}` : 'WAITING FOR DATA...'}
        </div>
      </div>
      
      <div style={{ width: '100%', height: '380px', borderRadius: '0 0 16px 16px', overflow: 'hidden', position: 'relative' }}>
        
        {/* Loading Overlay */}
        {!telemetry && (
            <div style={{
                position: 'absolute', top: 0, left: 0, width: '100%', height: '100%',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                background: '#020205', zIndex: 10, color: '#444'
            }}>
                <div style={{ textAlign: 'center' }}>
                    <div style={{ fontSize: '24px', marginBottom: '10px' }}>🔌</div>
                    <div>ESTABLISHING NEURAL LINK...</div>
                </div>
            </div>
        )}

        <Canvas camera={{ position: [0, 0, 90], fov: 45 }} gl={{ antialias: true, alpha: true }}>
            <ChemicalAtmosphere telemetry={telemetry} />
            <ambientLight intensity={0.5} color="#222244" />
            
            <BrainScene 
                reservoirActivity={reservoirActivity} 
                activations={activations}
                size={displaySize} 
                telemetry={telemetry}
            />
            
            <OrbitControls enableDamping={true} minDistance={50} maxDistance={200} />
        </Canvas>
      </div>
    </div>
  );
}
