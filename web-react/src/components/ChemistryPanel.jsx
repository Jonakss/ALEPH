import React from 'react';
import { Sparklines } from './Sparklines';

const ChemRow = ({ label, value, color }) => (
  <div className="chem-row" style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '14px' }}>
    <div className="chem-label" style={{
      fontSize: '11px', fontWeight: 500, color: 'var(--text-secondary)', width: '80px',
      textTransform: 'uppercase', letterSpacing: '1px'
    }}>{label}</div>
    <div className="chem-bar-track" style={{
      flex: 1, height: '8px', background: 'rgba(255,255,255,0.04)', borderRadius: '4px', overflow: 'hidden'
    }}>
      <div className="chem-bar-fill" style={{
         height: '100%', borderRadius: '4px', transition: 'width 0.3s ease, background 0.3s ease',
         width: `${Math.min(value * 100, 100)}%`,
         background: color,
         boxShadow: '0 0 12px rgba(0,0,0,0.5)'
      }}></div>
    </div>
    <div className="chem-value" style={{
       fontSize: '12px', fontFamily: "'JetBrains Mono', monospace", fontWeight: 500,
       width: '40px', textAlign: 'right', color: color.split(',')[1]?.trim() // Approximate text color or default
    }}>{value.toFixed(2)}</div>
  </div>
);

export function ChemistryPanel({ telemetry, history }) {
  const { dopamine, cortisol, adenosine, oxytocin, serotonin, trauma_state } = telemetry || {};

  return (
    <div className="panel chemistry-panel" style={{ gridColumn: 2, gridRow: 1 }}>
      <div className="panel-header">
        <div><span className="icon">⚗️</span> Neurochemistry</div>
        <div style={{ fontSize: '10px', color: trauma_state?.includes('FIREFIGHTER') ? 'var(--accent-red)' : 'var(--accent-green)' }}>
            {trauma_state || 'STABLE'}
        </div>
      </div>
      <div className="panel-body">
        <ChemRow label="Dopamine" value={dopamine || 0} color="linear-gradient(90deg, #ffd700, #ff8800)" />
        <ChemRow label="Cortisol" value={cortisol || 0} color="linear-gradient(90deg, #ff3344, #ff0066)" />
        <ChemRow label="Adenosine" value={adenosine || 0} color="linear-gradient(90deg, #4488ff, #2244cc)" />
        <ChemRow label="Oxytocin" value={oxytocin || 0} color="linear-gradient(90deg, #ff00aa, #ff44cc)" />
        <ChemRow label="Serotonin" value={serotonin || 0} color="linear-gradient(90deg, #00ff88, #00cc66)" />
        
        <div className="sparkline-container" style={{ marginTop: '8px', paddingTop: '14px', borderTop: '1px solid var(--border-glass)' }}>
            <Sparklines history={history} />
        </div>
      </div>
    </div>
  );
}
