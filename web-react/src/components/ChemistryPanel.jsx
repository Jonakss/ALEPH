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

function getChemState(t) {
    if (!t) return "OFFLINE";
    if (t.cortisol > 0.7) return "⚠️ HIGH STRESS (Defense Mode)";
    if (t.adenosine > 0.8) return "💤 FATIGUED (Pruning needed)";
    if (t.dopamine > 0.7 && t.serotonin > 0.5) return "🚀 HYPER-LEARNING (Flow)";
    if (t.dopamine > 0.6) return "✨ PLASTICITY ACTIVE";
    if (t.serotonin < 0.3 && t.dopamine < 0.3) return "📉 DEPRESSED (Low Activity)";
    return "⚖️ HOMEOSTASIS (Balanced)";
}

export function ChemistryPanel({ telemetry, history }) {
  const { dopamine, cortisol, adenosine, oxytocin, serotonin, trauma_state } = telemetry || {};

  return (
    <div className="panel chemistry-panel" style={{ gridColumn: 2, gridRow: 1 }}>
      <div className="panel-header">
        <div><span className="icon">⚗️</span> Neurochemistry</div>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end' }}>
            <div style={{ fontSize: '10px', color: trauma_state?.includes('FIREFIGHTER') ? 'var(--accent-red)' : 'var(--accent-green)' }}>
                {trauma_state || 'STABLE'}
            </div>
            <div style={{ fontSize: '9px', color: 'var(--text-dim)', marginTop: '2px' }}>
                {getChemState(telemetry)}
            </div>
        </div>
      </div>
      
      {/* GENOME TRAITS */}
      <div style={{
          display: 'flex', justifyContent: 'space-between', 
          marginBottom: '12px', fontSize: '10px', color: 'var(--text-secondary)',
          background: 'rgba(255,255,255,0.03)', padding: '6px 8px', borderRadius: '4px',
          border: '1px solid rgba(255,255,255,0.05)'
      }}>
          <span style={{color: 'var(--accent-purple)'}}>🧬 GENOME v{telemetry?.generation ?? '?'}</span>
          <span>
            <span title="Curiosity" style={{marginRight:'8px'}}>🧐 {telemetry?.curiosity?.toFixed(2) ?? '-'}</span>
            <span title="Stress Tolerance">🛡️ {telemetry?.stress_tolerance?.toFixed(2) ?? '-'}</span>
          </span>
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
