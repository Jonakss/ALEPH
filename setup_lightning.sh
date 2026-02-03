#!/bin/bash
set -e

echo "⚡ BEGINNING ALEPH LIGHTNING SETUP ⚡"

# 1. System Dependencies (Assuming Ubuntu/Debian based Lightning container)
echo "📦 Installing System Dependencies (ALSA, SSL, Build Tools)..."
sudo apt-get update && sudo apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    libasound2-dev \
    alsa-utils \
    cmake \
    clang

# 2. Rust Setup
if ! command -v cargo &> /dev/null
then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust is already installed."
fi

# 3. Model Weights
MODEL_DIR="weights"
MODEL_FILE="tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
MODEL_URL="https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"

mkdir -p $MODEL_DIR
if [ ! -f "$MODEL_DIR/$MODEL_FILE" ]; then
    echo "🧠 Downloading Model Weights ($MODEL_FILE)..."
    # Use wget or curl
    if command -v wget &> /dev/null; then
        wget -q --show-progress -O "$MODEL_DIR/$MODEL_FILE" "$MODEL_URL"
    else
        curl -L -o "$MODEL_DIR/$MODEL_FILE" "$MODEL_URL"
    fi
else
    echo "✅ Model weights found."
fi

# 4. Build
echo "🔨 Building ALEPH (Release Log)..."
cargo build --release

echo "✨ SETUP COMPLETE! ✨"
echo "To run ALEPH:"
echo "  cargo run --release"
