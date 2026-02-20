#!/bin/bash

# ALEPH Startup Script
# Usage: ./run.sh

echo "🌌 ALEPH: System Initialization..."

# 1. Kill any existing processes on ports 3030 (WS) or 5173 (Vite)
echo "🧹 Cleaning up ports..."
fuser -k 3030/tcp 2>/dev/null
fuser -k 5173/tcp 2>/dev/null

# 2. Trap Exit specific signals to kill children
trap 'kill $(jobs -p); echo "🛑 ALEPH Shutdown."; exit' SIGINT SIGTERM

# 3. Start Backend
echo "🧠 Starting Cortex (Backend)..."
cd "$(dirname "$0")"
cargo run --release &
BACKEND_PID=$!

# 4. Wait for Backend (Optional sleep or check)
sleep 2

# 5. Start Frontend
echo "👁️  Starting Web-React (Frontend)..."
cd "web-react"
npm run dev -- --host &
FRONTEND_PID=$!

# 6. Wait for both
echo "✅ System Online. Press Ctrl+C to stop."
wait $BACKEND_PID $FRONTEND_PID
