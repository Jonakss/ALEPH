import React from 'react';

export function Header({ isConnected, telemetry }) {
  const { loop_frequency, reservoir_size, entropy, hebbian_events } = telemetry || {};

  return (
    <header className="header" style={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      padding: '16px 24px',
      background: 'linear-gradient(180deg, rgba(10,10,20,0.95) 0%, rgba(10,10,20,0) 100%)',
      position: 'sticky',
      top: 0,
      zIndex: 100,
      backdropFilter: 'blur(12px)',
    }}>
      <div className="header-left" style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
        <div className="logo" style={{
          fontSize: '24px',
          fontWeight: 700,
          background: 'linear-gradient(135deg, var(--accent-cyan), var(--accent-magenta))',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          letterSpacing: '4px',
        }}>ALEPH</div>
        <div className={`connection-badge ${isConnected ? 'connected' : ''}`} style={{
           display: 'flex',
           alignItems: 'center',
           gap: '6px',
           padding: '4px 12px',
           borderRadius: '20px',
           fontSize: '11px',
           fontWeight: 500,
           background: isConnected ? 'rgba(0,255,136,0.1)' : 'rgba(255,51,68,0.15)',
           color: isConnected ? 'var(--accent-green)' : 'var(--accent-red)',
           border: `1px solid ${isConnected ? 'rgba(0,255,136,0.3)' : 'rgba(255,51,68,0.3)'}`,
           transition: 'all 0.5s ease',
        }}>
          <div className="connection-dot" style={{
             width: '6px', height: '6px', borderRadius: '50%', background: 'currentColor',
             animation: 'pulse 2s infinite'
          }}></div>
          <span>{isConnected ? 'CONNECTED' : 'DISCONNECTED'}</span>
        </div>
      </div>
      <div className="header-stats" style={{
         display: 'flex', gap: '20px', fontSize: '12px', 
         color: 'var(--text-secondary)', fontFamily: "'JetBrains Mono', monospace"
      }}>
        <div>Hz: <span style={{color: 'var(--accent-cyan)', fontWeight: 500}}>{(loop_frequency || 0).toFixed(1)}</span></div>
        <div>Neurons: <span style={{color: 'var(--accent-cyan)', fontWeight: 500}}>{reservoir_size || 0}</span></div>
        <div>Entropy: <span style={{color: 'var(--accent-cyan)', fontWeight: 500}}>{(entropy || 0).toFixed(3)}</span></div>
        <div>Hebbian: <span style={{color: 'var(--accent-cyan)', fontWeight: 500}}>{hebbian_events || 0}</span></div>
      </div>
    </header>
  );
}
