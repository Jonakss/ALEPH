import React, { useState } from 'react';
import { AudioCapture } from './AudioCapture';

export function Controls({ sendAction, sendStimulus, telemetry }) {
  const [inputObj, setInputObj] = useState('');
  
  const handleSend = () => {
      if (!inputObj.trim()) return;
      sendStimulus(inputObj);
      setInputObj('');
  };

  const { loop_frequency, reservoir_size, entropy, trauma_state } = telemetry || {};

  return (
    <div className="panel controls-panel" style={{ gridColumn: 2, gridRow: 2 }}>
      <div className="panel-header">
        <div><span className="icon">🎮</span> Interface</div>
      </div>
      <div className="panel-body">
        <div className="control-buttons" style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', marginBottom: '16px' }}>
          <button className="btn" onClick={() => sendAction('poke')}>👆 Poke</button>
          <button className="btn" onClick={() => sendAction('sleep')}>😴 Sleep</button>
          <button className="btn" onClick={() => sendAction('dream')}>🌙 Dream</button>
          <button className="btn btn-danger" onClick={() => window.alert('Stress Test initiated on backend via low-level signal (Simulated for safety)')}>⚡ Stress Test</button>
        </div>
        
        <div className="stimulus-input" style={{ display: 'flex', gap: '8px' }}>
          <input 
            type="text" 
            placeholder="Speak to ALEPH..." 
            value={inputObj}
            onChange={(e) => setInputObj(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSend()}
            style={{ flex: 1 }}
          />
          <button className="btn" onClick={handleSend}>Send</button>
        </div>

        {/* Browser Mic → WebSocket Audio */}
        <AudioCapture />

        <div className="sys-stats" style={{
             display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', marginTop: '16px',
             paddingTop: '14px', borderTop: '1px solid var(--border-glass)' 
        }}>
             <StatBox label="Loop Hz" value={(loop_frequency || 0).toFixed(1)} />
             <StatBox label="Neurons" value={reservoir_size || 0} />
             <StatBox label="Entropy" value={(entropy || 0).toFixed(3)} />
             <StatBox label="Trauma State" value={trauma_state || 'STABLE'} size="12px" />
        </div>
      </div>
    </div>
  );
}

const StatBox = ({ label, value, size = '16px' }) => (
    <div className="sys-stat" style={{ textAlign: 'center', padding: '8px', borderRadius: '8px', background: 'rgba(0,0,0,0.2)' }}>
        <div className="sys-stat-value" style={{ 
            fontFamily: "'JetBrains Mono', monospace", fontSize: size, fontWeight: 600, color: 'var(--accent-cyan)' 
        }}>{value}</div>
        <div className="sys-stat-label" style={{
            fontSize: '9px', color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: '1px', marginTop: '2px'
        }}>{label}</div>
    </div>
);
