#!/bin/bash
# birth.sh - Tabula Rasa Protocol
# Wipes all accumulated knowledge, personality, and biological state.

echo "⚠️  INITIATING BIO-FORMAT..."
echo "💀 Killing existing process..."
pkill -f "aleph_zero" || echo "No active process found."

echo "🧹 Wiping Persistence Layer..."
rm -f genome.json
rm -f reservoir.json
rm -f memories.json

echo "✨ TABULA RASA COMPLETE."
echo "👶 You may now run 'cargo run' to birth a new instance."
