#!/bin/bash
# ALEPH Lightning AI Setup Script
# Run with: bash setup_lightning.sh

set -e
echo "🧠 ALEPH Lightning AI Setup"
echo "=========================="

# 1. System Dependencies
echo "📦 Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev libasound2-dev curl git

# 2. Rust (if not installed)
if ! command -v cargo &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust already installed"
fi

# 3. Clone ALEPH (if not present)
if [ ! -d "ALEPH" ]; then
    echo "📥 Cloning ALEPH..."
    git clone https://github.com/YOUR_REPO/ALEPH.git
    cd ALEPH
else
    echo "✅ ALEPH directory exists"
    cd ALEPH
fi

# 4. Download Models
echo "📥 Downloading models..."
mkdir -p models

# TinyLlama (small, reliable)
if [ ! -f "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf" ]; then
    echo "   Downloading TinyLlama..."
    wget -q https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf
fi

# Tokenizer
if [ ! -f "tokenizer_tinyllama.json" ]; then
    echo "   Downloading tokenizer..."
    wget -q https://huggingface.co/TinyLlama/TinyLlama-1.1B-Chat-v1.0/resolve/main/tokenizer.json -O tokenizer_tinyllama.json
fi

# Whisper model
if [ ! -f "ggml-base.bin" ]; then
    echo "   Downloading Whisper..."
    wget -q https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
fi

# 5. Switch to TinyLlama (known working)
echo "🔧 Configuring for TinyLlama..."
sed -i 's|const MODEL_FILE:.*|const MODEL_FILE: \&str = "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf";|' src/core/llm.rs
sed -i 's|const TOKENIZER_FILE:.*|const TOKENIZER_FILE: \&str = "tokenizer_tinyllama.json";|' src/core/llm.rs

# Also switch back to Llama loader instead of custom Gemma
sed -i 's|use crate::core::quantized_gemma_raw::ModelWeights as Gemma;|use candle_transformers::models::quantized_llama::ModelWeights as Llama;|' src/core/llm.rs
sed -i 's|model: Gemma,|model: Llama,|' src/core/llm.rs
sed -i 's|fn load_model(device: \&Device) -> Result<Gemma>|fn load_model(device: \&Device) -> Result<Llama>|' src/core/llm.rs
sed -i 's|let model = Gemma::from_gguf|let model = Llama::from_gguf|' src/core/llm.rs

# 6. Verify GPU
echo "🖥️ Checking GPU..."
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-gpu=name,memory.total,memory.free --format=csv
else
    echo "⚠️ No NVIDIA GPU detected, will use CPU"
fi

# 7. Build
echo "🔨 Building ALEPH..."
cargo build --release 2>&1 | tail -20

# 8. Verify
echo ""
echo "✅ Setup complete!"
echo ""
echo "📁 Files:"
ls -lh *.gguf *.bin *.json 2>/dev/null || echo "   (check models/)"
echo ""
echo "🚀 To run: cargo run --release"
echo "📺 Note: TUI requires a terminal. For headless, you may need to disable TUI."
