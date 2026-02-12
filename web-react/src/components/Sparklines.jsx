import React, { useEffect, useRef } from 'react';

const SPARK_LEN = 120;
const sparkColors = {
  dopamine: '#ffd700', cortisol: '#ff3344', adenosine: '#4488ff',
  oxytocin: '#ff00aa', serotonin: '#00ff88', entropy: '#00d4ff'
};

const SparklineRow = ({ label, data, color }) => {
  const canvasRef = useRef(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !data || data.length < 2) return;

    const ctx = canvas.getContext('2d');
    const rect = canvas.getBoundingClientRect();
    
    // Handle High DPI
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const w = rect.width;
    const h = rect.height;

    ctx.clearRect(0, 0, w, h);

    // Gradient fill
    const grad = ctx.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, color + '40');
    grad.addColorStop(1, color + '00');

    ctx.beginPath();
    ctx.moveTo(0, h);
    
    // Draw area
    for (let i = 0; i < data.length; i++) {
        const x = (i / (SPARK_LEN - 1)) * w;
        const val = data[i] || 0;
        const y = h - (val * h * 0.9) - h * 0.05;
        ctx.lineTo(x, y);
    }
    // Finish area shape
    ctx.lineTo(((data.length - 1) / (SPARK_LEN - 1)) * w, h);
    ctx.closePath();
    ctx.fillStyle = grad;
    ctx.fill();

    // Draw Line
    ctx.beginPath();
    for (let i = 0; i < data.length; i++) {
      const x = (i / (SPARK_LEN - 1)) * w;
      const val = data[i] || 0;
      const y = h - (val * h * 0.9) - h * 0.05;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.stroke();

  }, [data, color]);

  return (
    <div className="sparkline-row" style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
      <div className="sparkline-label" style={{
         fontSize: '10px', color: 'var(--text-dim)', width: '65px', 
         textTransform: 'uppercase', letterSpacing: '0.8px'
      }}>{label}</div>
      <canvas ref={canvasRef} className="sparkline-canvas" style={{
         height: '24px', flex: 1, borderRadius: '4px', background: 'rgba(0,0,0,0.2)', width: '100%'
      }}></canvas>
    </div>
  );
};

export function Sparklines({ history }) {
  if (!history) return null;

  return (
    <>
      <SparklineRow label="Dopamine" data={history.dopamine} color={sparkColors.dopamine} />
      <SparklineRow label="Cortisol" data={history.cortisol} color={sparkColors.cortisol} />
      <SparklineRow label="Adenosine" data={history.adenosine} color={sparkColors.adenosine} />
      <SparklineRow label="Oxytocin" data={history.oxytocin} color={sparkColors.oxytocin} />
      <SparklineRow label="Serotonin" data={history.serotonin} color={sparkColors.serotonin} />
      <SparklineRow label="Entropy" data={history.entropy} color={sparkColors.entropy} />
    </>
  );
}
