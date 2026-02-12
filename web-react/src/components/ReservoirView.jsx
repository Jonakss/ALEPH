import React, { useMemo, useRef } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import * as THREE from 'three';

const MAX_NEURONS = 500;

function NeuronCloud({ activity }) {
  const pointsRef = useRef();

  // Generate spiral positions once
  const { positions, initialColors, sizes } = useMemo(() => {
    const pos = new Float32Array(MAX_NEURONS * 3);
    const col = new Float32Array(MAX_NEURONS * 3);
    const siz = new Float32Array(MAX_NEURONS);

    for (let i = 0; i < MAX_NEURONS; i++) {
      const t = i / MAX_NEURONS;
      const r = 20 + t * 30;
      const theta = t * Math.PI * 12;
      const y = (t - 0.5) * 40 + Math.sin(theta * 2) * 5;

      pos[i*3] = Math.cos(theta) * r;
      pos[i*3+1] = y;
      pos[i*3+2] = Math.sin(theta) * r;

      // Base color: Cyan/Blue-ish
      col[i*3] = 0; 
      col[i*3+1] = 0.83; 
      col[i*3+2] = 1;

      siz[i] = 2.0;
    }
    return { positions: pos, initialColors: col, sizes: siz };
  }, []);

  useFrame(() => {
    if (!pointsRef.current) return;
    
    const geometry = pointsRef.current.geometry;
    const colors = geometry.attributes.color.array;

    // Reset to base dark state
    for (let i = 0; i < MAX_NEURONS; i++) {
        colors[i*3] = 0;      // R
        colors[i*3+1] = 0.05; // G (Saved roughly from original logic where 0.02 was used)
        colors[i*3+2] = 0.1;  // B
    }

    // Apply activity
    if (activity && activity.length > 0) {
        // activity is array of [index, value] pairs (sparse)
        for (let i = 0; i < activity.length; i++) {
            const [idx, val] = activity[i];
            if (idx < MAX_NEURONS) {
                // Hot color (Red/Orange) for high activity, Cyan for low
                colors[idx*3] = Math.min(val * 2.0, 1.0);     // R
                colors[idx*3+1] = Math.max(0.2, 1.0 - val);   // G
                colors[idx*3+2] = Math.max(0.4, 1.0 - val);   // B
            }
        }
    }
    
    geometry.attributes.color.needsUpdate = true;
  });

  return (
    <points ref={pointsRef}>
      <bufferGeometry>
        <bufferAttribute attach="attributes-position" count={MAX_NEURONS} array={positions} itemSize={3} />
        <bufferAttribute attach="attributes-color" count={MAX_NEURONS} array={initialColors} itemSize={3} />
        <bufferAttribute attach="attributes-size" count={MAX_NEURONS} array={sizes} itemSize={1} />
      </bufferGeometry>
      <pointsMaterial
        size={2}
        vertexColors
        transparent
        opacity={0.85}
        sizeAttenuation
        blending={THREE.AdditiveBlending}
      />
    </points>
  );
}

function Scene({ activity }) {
    const orbitRef = useRef();
    
    useFrame(({ clock, camera }) => {
        const t = clock.getElapsedTime() * 0.1; // Slow rotation
        camera.position.x = Math.cos(t) * 80;
        camera.position.z = Math.sin(t) * 80;
        camera.position.y = Math.sin(t * 0.5) * 15;
        camera.lookAt(0, 0, 0);
    });

    return (
        <NeuronCloud activity={activity} />
    );
}

export function ReservoirView({ telemetry }) {
  const activity = telemetry?.reservoir_activity || []; // Standard format might need checking

  return (
    <div className="panel reservoir-panel" style={{ gridColumn: 1, gridRow: 1, minHeight: '420px', position: 'relative' }}>
      <div className="panel-header">
        <div><span className="icon">🧠</span> Neural Reservoir</div>
        {/* <div id="reservoir-info" style={{ color: 'var(--accent-cyan)' }}>Live</div> */}
      </div>
      <div style={{ width: '100%', height: '380px', borderRadius: '0 0 16px 16px', overflow: 'hidden' }}>
        <Canvas camera={{ position: [0, 0, 80], fov: 60 }} gl={{ antialias: true, alpha: true }}>
            <color attach="background" args={['#06060c']} />
            <ambientLight intensity={0.5} color="#222244" />
            <fog attach="fog" args={['#06060c', 0.003]} />
            <Scene activity={activity} />
        </Canvas>
      </div>
    </div>
  );
}
