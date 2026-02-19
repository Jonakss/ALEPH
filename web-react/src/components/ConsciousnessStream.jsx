import React, { useEffect, useRef, useMemo } from 'react';

export function ConsciousnessStream({ telemetry }) {
  const thoughts = useMemo(() => telemetry?.thoughts || [], [telemetry?.thoughts]);
  const bottomRef = useRef(null);
  const streamRef = useRef(null);

  useEffect(() => {
    // Only auto-scroll if we are already near the bottom
    const container = streamRef.current;
    if (container) {
        const isNearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 100;
        if (isNearBottom) {
            bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
        }
    } else {
        // Initial load
        bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [thoughts]);

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
        <div id="stream-count">{thoughts.length} thoughts</div>
      </div>
      <div className="panel-body" style={{ padding: 0 }}>
        <div 
           className="stream-content" 
           ref={streamRef}
           style={{
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
          {thoughts.map((text, i) => (
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
