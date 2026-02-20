import React from 'react';
import { useAlephSocket } from './hooks/useAlephSocket';
import { Header } from './components/Header';
import { TraumaBanner } from './components/TraumaBanner';
import { ReservoirView } from './components/ReservoirView';
import { VisualCortex } from './components/VisualCortex';
import { ChemistryPanel } from './components/ChemistryPanel';
import { ConsciousnessStream } from './components/ConsciousnessStream';
import { Controls } from './components/Controls';
import './index.css';

function App() {
  const { telemetry, history, isConnected, sendStimulus, sendAction, debugInfo } = useAlephSocket();

  return (
    <div className="app-container">
      <Header isConnected={isConnected} telemetry={telemetry} />
      <TraumaBanner telemetry={telemetry} />
      
      {/* DEBUG OVERLAY */}
      <div style={{ 
          position: 'fixed', bottom: 10, right: 10, 
          background: 'rgba(0,0,0,0.8)', color: '#0f0', 
          padding: '10px', fontSize: '10px', zIndex: 9999,
          pointerEvents: 'none', maxWidth: '300px', wordWrap: 'break-word', fontFamily: 'monospace'
      }}>
          STATUS: {isConnected ? 'CONN' : 'DISC'}<br/>
          DEBUG: {debugInfo}<br/>
          KEYS: {telemetry ? Object.keys(telemetry).join(', ') : 'NO DATA'}
      </div>
      
      <div className="dashboard" style={{
        display: 'grid',
        gridTemplateColumns: 'minmax(0, 1fr) 380px', // Prevent overflow with minmax
        gridTemplateRows: 'auto auto',
        gap: '16px',
        padding: '16px 24px 24px',
        maxWidth: '1600px',
        margin: '0 auto',
      }}>
        {/* ROW 1 */}
        <ReservoirView telemetry={telemetry} />
        <VisualCortex telemetry={telemetry} />
        <ChemistryPanel telemetry={telemetry} history={history} />
        
        {/* ROW 2 */}
        <ConsciousnessStream telemetry={telemetry} />
        <Controls 
            sendAction={sendAction} 
            sendStimulus={sendStimulus} 
            telemetry={telemetry} 
        />
      </div>

      <style>{`
        @media (max-width: 900px) {
            .dashboard {
                grid-template-columns: 1fr !important;
            }
            .reservoir-panel { grid-column: 1 !important; grid-row: 1 !important; }
            .chemistry-panel { grid-column: 1 !important; grid-row: 2 !important; }
            .stream-panel { grid-column: 1 !important; grid-row: 3 !important; }
            .controls-panel { grid-column: 1 !important; grid-row: 4 !important; }
        }
      `}</style>
    </div>
  );
}

export default App;
