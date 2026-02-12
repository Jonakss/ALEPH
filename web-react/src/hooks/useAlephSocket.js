import { useState, useEffect, useRef, useCallback } from 'react';
import useWebSocket, { ReadyState } from 'react-use-websocket';

const WS_URL = 'ws://localhost:3030';

export function useAlephSocket() {
  const [telemetry, setTelemetry] = useState(null);
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
    filter: () => false, // Process all messages
  });

  useEffect(() => {
    if (lastMessage !== null) {
      try {
        const data = JSON.parse(lastMessage.data);
        setTelemetry(data);
        updateHistory(data);
      } catch (e) {
        console.error("Failed to parse telemetry", e);
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
    sendAction
  };
}
