import { useState, useEffect, useCallback } from 'react';
import useWebSocket, { ReadyState } from 'react-use-websocket';

const WS_URL = 'ws://localhost:3030';

export function useAlephSocket() {
  const [telemetry, setTelemetry] = useState(null);
  const [debugInfo, setDebugInfo] = useState("Initializing...");
  const [history, setHistory] = useState({
    dopamine: [],
    cortisol: [],
    adenosine: [],
    oxytocin: [],
    serotonin: [],
    entropy: [],
  });
  
  // Keep last 120 points for sparklines
  const SPARK_LEN = 120;

  const updateHistory = useCallback((newData) => {
    setHistory(prev => {
      const next = { ...prev };
      ['dopamine', 'cortisol', 'adenosine', 'oxytocin', 'serotonin', 'entropy'].forEach(key => {
        const val = newData[key] || 0;
        next[key] = [...(prev[key] || []), val].slice(-SPARK_LEN);
      });
      return next;
    });
  }, []);

  const { sendMessage, lastMessage, readyState } = useWebSocket(WS_URL, {
    onOpen: () => console.log('Connected to ALEPH'),
    shouldReconnect: () => true,
    reconnectInterval: 3000,
  });

  useEffect(() => {
    if (lastMessage !== null) {
      try {
        let raw = lastMessage.data;
        
        // Handle Blob (binary) data if received
        if (raw instanceof Blob) {
            setDebugInfo(`WARN: Received Blob (${raw.size}b). Expecting Text.`);
            // You might need to text() it if it's actually text data sent as binary
            // raw.text().then(...) 
            return; 
        }

        const parsed = JSON.parse(raw);
        // Handle Rust Enum serialization: {"Telemetry": {...}}
        const data = parsed.Telemetry || parsed; 
        
        if (data && Object.keys(data).length > 0) {
            setTelemetry(data);
            updateHistory(data);
            setDebugInfo(`OK: ${Object.keys(data).length} keys. Size: ${raw.length}`);
        } else {
            console.warn("Parsed object is empty or invalid:", parsed);
            setDebugInfo(`WARN: Parsed JSON has no keys or data is null. Raw: ${raw.substring(0,20)}...`);
        }
      } catch (e) {
        console.error("Failed to parse telemetry", e);
        const snippet = typeof lastMessage.data === 'string' ? lastMessage.data.substring(0, 50) : 'Binary/Unknown';
        setDebugInfo(`ERR: ${e.message} | MSG: ${snippet}...`);
      }
    }
  }, [lastMessage, updateHistory]);

  const sendStimulus = (text) => {
    sendMessage(JSON.stringify({ stimulus: text }));
  };

  const sendAction = (action) => {
    sendMessage(JSON.stringify({ action }));
  };

  return {
    telemetry,
    history,
    readyState,
    isConnected: readyState === ReadyState.OPEN,
    sendStimulus,
    sendAction,
    debugInfo
  };
}
