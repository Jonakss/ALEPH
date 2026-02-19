import React, { useRef, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import { Icosahedron, Sphere, Torus, Sparkles } from '@react-three/drei';
import * as THREE from 'three';

export function Avatar({ telemetry }) {
    const groupRef = useRef();
    const coreRef = useRef();
    const ringRef = useRef();
    
    // Extract State
    const cortisol = telemetry?.cortisol || 0;
    const dopamine = telemetry?.dopamine || 0;
    const state = telemetry?.current_state || "IDLE";
    
    // Determine Color based on emotion
    const baseColor = useMemo(() => {
        if (state.includes("PANIC") || cortisol > 0.6) return new THREE.Color("#ff3333"); // Red/Stress
        if (state.includes("SLEEP")) return new THREE.Color("#3300aa"); // Deep Blue/Sleep
        if (dopamine > 0.6) return new THREE.Color("#ffaa00"); // Gold/Excited
        return new THREE.Color("#00ffff"); // Default Cyan
    }, [cortisol, dopamine, state]);

    useFrame(({ clock }) => {
        const t = clock.getElapsedTime();
        
        // Jitter (Cortisol)
        const jitter = cortisol * 0.1 * Math.sin(t * 50);
        
        if (groupRef.current) {
            // Floating
            groupRef.current.position.y = Math.sin(t) * 0.2;
            groupRef.current.position.x = jitter;
        }

        if (coreRef.current) {
            // Spin speed based on dopamine
            const speed = 0.5 + dopamine;
            coreRef.current.rotation.x = t * speed;
            coreRef.current.rotation.y = t * speed * 0.8;
            
            // Pulse size based on "Breathing" AND Audio
            const audioEnergy = (telemetry?.audio_spectrum?.rms || 0) * 10.0;
            const breath = Math.sin(t * 2) * 0.1 + 1 + Math.min(audioEnergy, 0.5);
            coreRef.current.scale.setScalar(breath);
        }
        
        if (ringRef.current) {
             ringRef.current.rotation.z = -t * 0.5;
             ringRef.current.rotation.x = Math.sin(t) * 0.5;
        }
    });

    return (
        <group ref={groupRef} position={[0, 0, 0]}>
            {/* Inner Light */}
            <pointLight position={[0, 0, 0]} intensity={2} color={baseColor} distance={10} decay={2} />
            
            {/* Core Entity - Hyper Glass */}
            <Icosahedron ref={coreRef} args={[1.5, 0]}>
                <meshPhysicalMaterial 
                    color={baseColor} 
                    emissive={baseColor}
                    emissiveIntensity={0.5}
                    roughness={0.1}
                    metalness={0.2}
                    transmission={0.6}
                    thickness={3}
                    ior={1.5}
                    clearcoat={1}
                />
            </Icosahedron>
            
            {/* Thinking Particles */}
            <Sparkles count={50} scale={6} size={4} speed={0.4} opacity={0.5} color={baseColor} />

            {/* Orbiting Ring (Consciousness) */}
            <Torus ref={ringRef} args={[3.0, 0.05, 16, 100]} rotation={[1.5, 0, 0]}>
                 <meshBasicMaterial color={baseColor} transparent opacity={0.3} />
            </Torus>
            
            {/* Second Ring */}
            <Torus args={[3.5, 0.02, 16, 100]} rotation={[2, 0, 0]}>
                 <meshBasicMaterial color="#ffffff" transparent opacity={0.1} />
            </Torus>
        </group>
    );
}
