#!/bin/bash
# ALEPH Lightning AI Setup Script - CPU Only (avoids CUDA build issues)
# Run with: bash setup_lightning.sh

set -e
echo "🧠 ALEPH Lightning AI Setup (CPU Mode)"
echo "======================================="

# 1. Install libclang (required by whisper-rs bindgen)
echo "📦 Installing libclang..."
sudo apt-get update
sudo apt-get install -y libclang-dev clang build-essential pkg-config libssl-dev libasound2-dev curl

# 2. Set environment for libclang
export LIBCLANG_PATH=/usr/lib/llvm-14/lib/
echo "export LIBCLANG_PATH=/usr/lib/llvm-14/lib/" >> ~/.bashrc

# 3. Rust (if not installed)
if ! command -v cargo &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust already installed: $(rustc --version)"
fi

# 4. Modify Cargo.toml to disable CUDA (avoid kernel build errors)
echo "🔧 Disabling CUDA features..."
if [ -f "Cargo.toml" ]; then
    # Remove cuda feature from candle dependencies
    sed -i 's/, "cuda"//g' Cargo.toml
    sed -i 's/"cuda",//g' Cargo.toml
    sed -i 's/features = \["cuda"\]/features = []/g' Cargo.toml
fi

# 5. Download models
echo "📥 Downloading models..."
# TinyLlama (small, reliable, 638MB)
if [ ! -f "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf" ]; then
    echo "   Downloading TinyLlama..."
    wget -q --show-progress https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf
fi

# Tokenizer
if [ ! -f "tokenizer_tinyllama.json" ]; then
    echo "   Downloading tokenizer..."
    wget -q https://huggingface.co/TinyLlama/TinyLlama-1.1B-Chat-v1.0/resolve/main/tokenizer.json -O tokenizer_tinyllama.json
fi

# Whisper model (base, ~140MB)
if [ ! -f "ggml-base.bin" ]; then
    echo "   Downloading Whisper base model..."
    wget -q --show-progress https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
fi

# 6. Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean 2>/dev/null || true

# 7. Build (CPU only)
echo "🔨 Building ALEPH (CPU only, this may take a while)..."
CUDA_VISIBLE_DEVICES="" cargo build --release 2>&1 | tail -30

echo ""
echo "✅ Setup complete!"
echo ""
echo "📁 Models:"
ls -lh *.gguf *.bin 2>/dev/null || echo "   Check downloads"
echo ""
echo "🚀 To run: CUDA_VISIBLE_DEVICES='' cargo run --release"
echo ""
echo "⚠️  Note: Lightning Studio may not have audio input."
echo "    ALEPH will run but voice input won't work without a mic."
