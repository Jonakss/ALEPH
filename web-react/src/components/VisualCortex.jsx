import React, { useRef, useEffect } from 'react';

export function VisualCortex({ telemetry }) {
    const canvasRef = useRef(null);

    useEffect(() => {
        if (!telemetry || !telemetry.visual_cortex) return;

        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');
        const width = 64;
        const height = 64;
        const pixelSize = 4; // Scale up 4x for 256x256 display

        // Clear
        ctx.fillStyle = '#000';
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        const data = telemetry.visual_cortex;

        // Render Grid
        for (let y = 0; y < height; y++) {
            for (let x = 0; x < width; x++) {
                const idx = y * width + x;
                const val = data[idx]; // 0.0 to 1.0

                if (val > 0.05) { // Threshold to ignore noise
                    // Heatmap: 
                    // Low val -> Dark Red
                    // High val -> Bright Orange/White
                    const r = Math.min(255, val * 255 * 2.0); 
                    const g = Math.min(255, Math.max(0, (val - 0.5) * 255 * 2.0));
                    const b = Math.min(255, Math.max(0, (val - 0.8) * 255 * 5.0));
                    
                    ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
                    ctx.fillRect(x * pixelSize, y * pixelSize, pixelSize, pixelSize);
                }
            }
        }
        
        // Scanlines / CRT Effect
        ctx.fillStyle = 'rgba(0, 0, 0, 0.1)';
        for (let y = 0; y < canvas.height; y += 2) {
            ctx.fillRect(0, y, canvas.width, 1);
        }

    }, [telemetry]);

    return (
        <div className="panel visual-panel" style={{ 
            background: '#050505', 
            border: '1px solid #333', 
            padding: '10px',
            borderRadius: '8px',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center'
        }}>
            <h3 style={{ margin: '0 0 10px 0', color: '#f50', fontSize: '12px', textTransform: 'uppercase', letterSpacing: '1px' }}>
                Occipital Feed (64x64)
            </h3>
            <canvas 
                ref={canvasRef} 
                width={256} 
                height={256} 
                style={{ 
                    border: '1px solid #222',
                    imageRendering: 'pixelated',
                    boxShadow: '0 0 10px rgba(255, 50, 0, 0.2)'
                }} 
            />
        </div>
    );
}
