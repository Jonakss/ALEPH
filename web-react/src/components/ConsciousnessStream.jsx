import React, { useEffect, useState, useRef } from 'react';

export function ConsciousnessStream({ telemetry }) {
  const [stream, setStream] = useState([]);
  const bottomRef = useRef(null);
  const lastThoughtRef = useRef('');

  useEffect(() => {
    if (telemetry?.current_state) {
      const thought = telemetry.current_state;
      if (thought !== lastThoughtRef.current) {
        lastThoughtRef.current = thought;
        setStream(prev => {
            const next = [...prev, thought];
            if (next.length > 200) next.shift(); // Limit history
            return next;
        });
      }
    }
  }, [telemetry?.current_state]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [stream]);

  const getStyle = (text) => {
    if (text.includes('🧠') || text.includes('🔥') || text.includes('🛡️') || text.includes('⚙️')) {
      return { color: 'var(--accent-cyan)' };
    } else if (text.includes('💬') || text.includes('🗣️')) {
      return { color: 'var(--accent-green)' };
    } else {
      return { color: 'var(--accent-magenta)' };
    }
  };

  return (
    <div className="panel stream-panel" style={{ gridColumn: 1, gridRow: 2, display: 'flex', flexDirection: 'column' }}>
      <div className="panel-header">
        <div><span className="icon">💭</span> Consciousness Stream</div>
        <div id="stream-count">{stream.length} thoughts</div>
      </div>
      <div className="panel-body" style={{ padding: 0 }}>
        <div className="stream-content" style={{
           flex: 1,
           minHeight: '200px',
           maxHeight: '260px',
           overflowY: 'auto',
           fontFamily: "'JetBrains Mono', monospace",
           fontSize: '12px',
           lineHeight: 1.8,
           padding: '16px',
           scrollbarWidth: 'thin',
           scrollbarColor: 'var(--border-glass) transparent'
        }}>
          {stream.map((text, i) => (
            <div key={i} className="stream-entry" style={{
                padding: '4px 0',
                borderBottom: '1px solid rgba(255,255,255,0.02)',
                animation: 'fadeIn 0.3s ease forwards',
                ...getStyle(text)
            }}>
              {text}
            </div>
          ))}
          <div ref={bottomRef} />
        </div>
      </div>
    </div>
  );
}
