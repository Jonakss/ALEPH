#!/bin/bash

# Configuration of paths for Lightning AI (T4)
export CUDA_HOME=/usr/local/cuda-12.2
export PATH=$CUDA_HOME/bin:$PATH
export LD_LIBRARY_PATH=$CUDA_HOME/lib64:/usr/local/lib/ollama/cuda_v12:$LD_LIBRARY_PATH

# Architecture for T4
export CANDLE_CUDA_ARCH=75

echo "--- ALEPH GPU BOOTSTRAP ---"
echo "CUDA_HOME: $CUDA_HOME"
echo "CANDLE_CUDA_ARCH: $CANDLE_CUDA_ARCH"

# Re-run with headless flag
cargo run -- --headless
