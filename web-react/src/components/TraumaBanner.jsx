import React from 'react';

export function TraumaBanner({ telemetry }) {
  const { trauma_state } = telemetry || {};
  const isActive = trauma_state && (trauma_state.includes('FIREFIGHTER') || trauma_state.includes('RECOVERING'));

  if (!isActive) return null;

  return (
    <div className="trauma-banner" style={{
        padding: '8px 24px',
        background: 'linear-gradient(90deg, rgba(255,51,68,0.15), rgba(255,0,170,0.1)',
        borderBottom: '1px solid rgba(255,51,68,0.3)',
        fontSize: '13px',
        fontWeight: 500,
        color: 'var(--accent-red)',
        textAlign: 'center',
        animation: 'traumaPulse 1.5s ease-in-out infinite'
    }}>
      🔥 LUCIFER PROTOCOL ACTIVE — Firefighter Mode Engaged — {trauma_state}
      <style>{`
        @keyframes traumaPulse {
            0%, 100% { opacity: 0.8; }
            50% { opacity: 1; }
        }
      `}</style>
    </div>
  );
}
