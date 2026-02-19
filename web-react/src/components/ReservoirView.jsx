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

    const fogColor = useMemo(() => new THREE.Color(r, g, b), [r, g, b]);
    const bgArgs = useMemo(() => [fogColor], [fogColor]);

    return (
        <>
            <color attach="background" args={bgArgs} />
            <fog attach="fog" color={fogColor} near={40} far={160} />
        </>
    );
}

// DYNAMIC CONNECTOME — Renders REAL backend positions
// Data source: telemetry.neuron_positions (from FractalReservoir)
// No fake clustering — what you see IS the reservoir topology.
function Connectome({ activity, regionMap, neuronPositions }) {
    const pointsRef = useRef();
    
    // Pre-allocate buffers (filled from backend data each frame)
    const buffers = useMemo(() => {
        const pos = new Float32Array(MAX_NEURONS * 3);
        const col = new Float32Array(MAX_NEURONS * 3);
        // Dark init
        for (let i = 0; i < MAX_NEURONS; i++) {
            col[i*3] = 0.01; col[i*3+1] = 0.01; col[i*3+2] = 0.02;
        }
        return { positions: pos, colors: col };
    }, []);

    useFrame((state) => {
        if (!pointsRef.current) return;

        const posAttr = pointsRef.current.geometry.attributes.position;
        const colAttr = pointsRef.current.geometry.attributes.color;
        const time = state.clock.getElapsedTime();
        
        // How many neurons does the backend report?
        const backendCount = neuronPositions ? neuronPositions.length : 0;
        const count = Math.min(backendCount, MAX_NEURONS);

        for (let i = 0; i < MAX_NEURONS; i++) {
            if (i < count) {
                // === REAL POSITION from Backend ===
                const bp = neuronPositions[i]; // [x, y, z]
                // Add subtle breathing drift (organic feel, doesn't change topology)
                const bx = bp[0] + Math.sin(time * 0.3 + i) * 0.15;
                const by = bp[1] + Math.cos(time * 0.4 + i * 2) * 0.15;
                const bz = bp[2] + Math.sin(time * 0.2 + i * 0.5) * 0.15;
                
                posAttr.setXYZ(i, bx, by, bz);

                // === COLORING (Activation + Region) ===
                let val = 0;
                if (activity) {
                    if (Array.isArray(activity[i])) val = activity[i][1];
                    else if (i < activity.length) val = activity[i] || 0;
                }
                
                const rId = (regionMap && i < regionMap.length) ? regionMap[i] : 3;
                
                // Region Palette
                let r=0.1, g=0.3, b=0.5; // Default: Association (dim blue)
                if (rId === 0) { r=1.0; g=0.8; b=0.1; } // Semantic (Gold)
                else if (rId === 1) { r=0.1; g=1.0; b=0.2; } // Auditory (Green)
                else if (rId === 2) { r=0.6; g=0.0; b=1.0; } // Limbic (Purple)
                
                const intensity = Math.max(0, val);
                const flash = intensity * intensity * 0.5;

                colAttr.setXYZ(i, 
                    r * intensity + flash + 0.02, 
                    g * intensity + flash + 0.02, 
                    b * intensity + flash + 0.04
                );
            } else {
                // Unused neurons: invisible
                posAttr.setXYZ(i, 0, 0, 0);
                colAttr.setXYZ(i, 0, 0, 0);
            }
        }
        
        posAttr.needsUpdate = true;
        colAttr.needsUpdate = true;
    });

    return (
        <points ref={pointsRef}>
            <bufferGeometry>
                <bufferAttribute attach="attributes-position" count={MAX_NEURONS} array={buffers.positions} itemSize={3} />
                <bufferAttribute attach="attributes-color" count={MAX_NEURONS} array={buffers.colors} itemSize={3} />
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

// INJECT FLOW — LLM → Reservoir connections
// Uses REAL backend positions for neuron endpoints
function InjectFlow({ neuronPositions, frontVecs, size, activations, regionMap }) {
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
        
        if (!activations || activations.length === 0 || !neuronPositions || neuronPositions.length === 0) {
            lineRef.current.geometry.setDrawRange(0, 0);
            return;
        }

        const posAttr = lineRef.current.geometry.attributes.position;
        const colAttr = lineRef.current.geometry.attributes.color;
        
        let lineIdx = 0;
        const maxLines = 300;

        for (let i = 0; i < size && i < neuronPositions.length && lineIdx < maxLines; i++) {
            // Only Semantic neurons (Region 0 - Gold) receive direct LLM flux
            if (regionMap && regionMap[i] !== 0) continue; 

            const actIdx = i % activations.length;
            const val = activations[actIdx] || 0;
            
            if (val < 0.15) continue;

            const v1 = frontVecs[actIdx];       // Source: Frontal Lobe concept
            const np = neuronPositions[i];      // Target: Real reservoir position
            
            if (v1 && np) {
                posAttr.setXYZ(lineIdx*2, v1.x, v1.y, v1.z);
                posAttr.setXYZ(lineIdx*2+1, np[0], np[1], np[2]);
                
                const a = Math.min(val, 1.0);
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

// AUDITORY FLOW — Audio Spectrogram → Auditory Cortex
// Uses REAL backend positions for neuron endpoints
function AuditoryFlow({ neuronPositions, embedding, regionMap, size }) {
    const lineRef = useRef();

    const { geometry } = useMemo(() => {
        const maxLines = 128;
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
    const leftEar = useMemo(() => [-45, -10, 0], []);
    const rightEar = useMemo(() => [45, -10, 0], []);

    useFrame(() => {
        if (!lineRef.current) return;
        
        if (!embedding || embedding.length === 0 || !neuronPositions || neuronPositions.length === 0) {
            lineRef.current.geometry.setDrawRange(0, 0);
            return;
        }

        const posAttr = lineRef.current.geometry.attributes.position;
        const colAttr = lineRef.current.geometry.attributes.color;
        
        let lineIdx = 0;
        const maxLines = 128;

        for (let b = 0; b < embedding.length && lineIdx < maxLines; b++) {
            const val = embedding[b];
            if (val < 0.05) continue;

            // Find target auditory neuron for this band
            let targetIdx = -1;
            if (regionMap && regionMap.length > 0) {
                let start = (b * 5) % size;
                for(let k=0; k<size; k++) {
                    let idx = (start + k) % size;
                    if (regionMap[idx] === 1) { targetIdx = idx; break; }
                }
            }
            if (targetIdx === -1) {
                targetIdx = (b * 13) % size;
            }

            if (targetIdx >= neuronPositions.length) continue;
            const np = neuronPositions[targetIdx];
            if (!np) continue;

            // Choose nearest ear based on neuron x position
            const src = (np[0] < 0) ? leftEar : rightEar;

            posAttr.setXYZ(lineIdx*2, src[0], src[1], src[2]);
            posAttr.setXYZ(lineIdx*2+1, np[0], np[1], np[2]);

            // Green Pulse
            colAttr.setXYZ(lineIdx*2, 0.2 * val, 1.0 * val, 0.4 * val);
            colAttr.setXYZ(lineIdx*2+1, 0.1 * val, 0.8 * val, 0.2 * val);
            lineIdx++;
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
// Uses REAL backend positions for both endpoints
// Pairs are generated based on spatial proximity (small-world topology)
function HebbianWeb({ neuronPositions, activity, size, regionMap }) {
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

    // Generate candidate pairs based on spatial proximity
    const pairs = useMemo(() => {
        if (!neuronPositions || neuronPositions.length === 0) return [];
        
        const p = [];
        const rng = mulberry32(101);
        const targetPairs = 1000;
        const nCount = Math.min(neuronPositions.length, MAX_NEURONS);
        let attempts = 0;
        
        while (p.length < targetPairs && attempts < 10000) {
            attempts++;
            const i = Math.floor(rng() * nCount);
            const j = Math.floor(rng() * nCount);
            if (i === j) continue;
            
            const pi = neuronPositions[i];
            const pj = neuronPositions[j];
            if (!pi || !pj) continue;
            
            const dx = pi[0]-pj[0], dy = pi[1]-pj[1], dz = pi[2]-pj[2];
            const dist = Math.sqrt(dx*dx + dy*dy + dz*dz);
            
            // Small World Probability
            const prob = 5.0 / (dist + 0.1); 
            if (rng() < prob || rng() < 0.01) {
                p.push([i, j]);
            }
        }
        return p;
    }, [neuronPositions]);

    useFrame(() => {
        if (!lineRef.current || !activity || activity.length === 0 || !neuronPositions || neuronPositions.length === 0) {
            if (lineRef.current) lineRef.current.geometry.setDrawRange(0, 0);
            return;
        }

        const posAttr = lineRef.current.geometry.attributes.position;
        const colAttr = lineRef.current.geometry.attributes.color;
        
        let lineIdx = 0;
        const maxLines = 600;
        const threshold = 0.4;

        for (let k = 0; k < pairs.length && lineIdx < maxLines; k++) {
            const [i, j] = pairs[k];
            
            let vi = 0, vj = 0;
            if (Array.isArray(activity[i])) { vi = activity[i][1]; } 
            else if (i < activity.length) { vi = activity[i] || 0; }
            if (Array.isArray(activity[j])) { vj = activity[j][1]; }
            else if (j < activity.length) { vj = activity[j] || 0; }

            if (vi > threshold && vj > threshold) {
                if (i >= neuronPositions.length || j >= neuronPositions.length) continue;
                const p1 = neuronPositions[i];
                const p2 = neuronPositions[j];
                
                posAttr.setXYZ(lineIdx*2, p1[0], p1[1], p1[2]);
                posAttr.setXYZ(lineIdx*2+1, p2[0], p2[1], p2[2]);
                
                // Color based on Region
                const r1 = regionMap ? regionMap[i] : 3;
                const r2 = regionMap ? regionMap[j] : 3;
                
                let r=0.1, g=0.3, b=0.5;
                if (r1 === 1 && r2 === 1) { r=0.2; g=1.0; b=0.2; }
                else if (r1 === 0 && r2 === 0) { r=1.0; g=0.8; b=0.2; }
                else if (r1 === 2 && r2 === 2) { r=0.6; g=0.1; b=1.0; }
                else if (r1 === 3 || r2 === 3) { r=0.2; g=0.7; b=1.0; }
                
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
    const frontal = useMemo(() => generateFrontalLobe(), []);

    const hebbianEvents = telemetry?.hebbian_events || 0;
    const entropy = telemetry?.entropy || 0;
    const traumaState = telemetry?.trauma_state || '';
    const regionMap = useMemo(() => telemetry?.region_map || [], [telemetry?.region_map]);
    const neuronPositions = telemetry?.neuron_positions;
    const lastLogRef = useRef(0);
    
    // Global Hebbian Glow State (Shared)
    const hebbianGlowRef = useRef(0.0);
    
    React.useEffect(() => {
        if (hebbianEvents > 0) hebbianGlowRef.current = Math.min(1.0, hebbianGlowRef.current + hebbianEvents * 0.05);
    }, [hebbianEvents]);

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
            <Connectome 
                activity={reservoirActivity} 
                regionMap={regionMap}
                neuronPositions={neuronPositions}
            />

            {/* 3. INJECT FLOW — LLM→Reservoir connections */}
            <InjectFlow 
                neuronPositions={neuronPositions}
                frontVecs={frontal.vec3s} 
                size={size} 
                activations={activations}
                regionMap={regionMap}
            />

            {/* 3c. AUDITORY FLOW — Audio → Auditory Cortex */}
            <AuditoryFlow
                neuronPositions={neuronPositions}
                embedding={telemetry?.audio_spectrum?.frequency_embedding}
                regionMap={regionMap}
                size={size}
            />

            {/* 3b. HEBBIAN WEB — Intra-reservoir connections */}
            <HebbianWeb
                neuronPositions={neuronPositions}
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
