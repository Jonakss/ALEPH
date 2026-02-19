import { useState, useRef, useCallback } from 'react';

const WS_URL = 'ws://localhost:3030';

/**
 * AudioCapture — Browser Microphone → WebSocket Binary
 * 
 * Captures audio from the browser's mic using getUserMedia + ScriptProcessorNode,
 * sends raw f32 PCM as binary WebSocket frames to the backend.
 * Backend decodes them in daemon.rs (opcode 0x2) and feeds to ears.rs processor.
 */
export function AudioCapture() {
    const [isCapturing, setIsCapturing] = useState(false);
    const [error, setError] = useState(null);
    const wsRef = useRef(null);
    const streamRef = useRef(null);
    const audioCtxRef = useRef(null);
    const processorRef = useRef(null);

    const startCapture = useCallback(async () => {
        try {
            setError(null);

            // 1. Get browser microphone
            const stream = await navigator.mediaDevices.getUserMedia({ 
                audio: { 
                    sampleRate: 44100,
                    channelCount: 1,
                    echoCancellation: false,
                    noiseSuppression: false,
                    autoGainControl: false,
                } 
            });
            streamRef.current = stream;

            // 2. Create AudioContext
            const audioCtx = new AudioContext({ sampleRate: 44100 });
            audioCtxRef.current = audioCtx;

            const source = audioCtx.createMediaStreamSource(stream);

            // 3. Open dedicated WebSocket for binary audio
            const ws = new WebSocket(WS_URL);
            ws.binaryType = 'arraybuffer';
            wsRef.current = ws;

            ws.onopen = () => {
                console.log('🎤 Audio WebSocket Connected');

                // 4. ScriptProcessorNode to capture PCM
                // bufferSize=4096 gives ~93ms chunks at 44100Hz
                const processor = audioCtx.createScriptProcessor(4096, 1, 1);
                processorRef.current = processor;

                processor.onaudioprocess = (e) => {
                    if (ws.readyState !== WebSocket.OPEN) return;

                    const inputData = e.inputBuffer.getChannelData(0); // Float32Array
                    // Send raw f32 bytes (little-endian, which is browser default)
                    ws.send(inputData.buffer);
                };

                source.connect(processor);
                processor.connect(audioCtx.destination); // Required for processing to work

                setIsCapturing(true);
            };

            ws.onerror = (e) => {
                console.error('Audio WS Error:', e);
                setError('WebSocket connection failed');
            };

            ws.onclose = () => {
                console.log('🎤 Audio WebSocket Closed');
                setIsCapturing(false);
            };

        } catch (err) {
            console.error('Mic capture failed:', err);
            setError(err.message || 'Mic access denied');
        }
    }, []);

    const stopCapture = useCallback(() => {
        // Disconnect processor
        if (processorRef.current) {
            processorRef.current.disconnect();
            processorRef.current = null;
        }
        // Close AudioContext
        if (audioCtxRef.current) {
            audioCtxRef.current.close();
            audioCtxRef.current = null;
        }
        // Stop mic tracks
        if (streamRef.current) {
            streamRef.current.getTracks().forEach(t => t.stop());
            streamRef.current = null;
        }
        // Close WebSocket
        if (wsRef.current) {
            wsRef.current.close();
            wsRef.current = null;
        }
        setIsCapturing(false);
    }, []);

    return (
        <div style={{ marginTop: '8px' }}>
            <button
                onClick={isCapturing ? stopCapture : startCapture}
                style={{
                    width: '100%',
                    padding: '10px',
                    borderRadius: '10px',
                    border: `1px solid ${isCapturing ? 'var(--cortisol)' : 'var(--border)'}`,
                    background: isCapturing 
                        ? 'rgba(255, 50, 50, 0.15)' 
                        : 'rgba(255, 255, 255, 0.03)',
                    color: isCapturing ? 'var(--cortisol)' : 'var(--text-dim)',
                    cursor: 'pointer',
                    fontSize: '13px',
                    transition: 'all 0.3s',
                }}
            >
                {isCapturing ? '🔴 Stop Listening' : '🎤 Stream Mic to ALEPH'}
            </button>
            {error && (
                <div style={{ color: 'var(--cortisol)', fontSize: '10px', marginTop: '4px' }}>
                    ⚠️ {error}
                </div>
            )}
        </div>
    );
}
