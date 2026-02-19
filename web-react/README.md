# ALEPH Neural Dashboard (React)

This directory contains the React-based frontend for the ALEPH Observer. It replaces the legacy static HTML implementation.

## Features

- **Real-time Neurochemistry**: Visualization of Dopamine, Cortisol, Adenosine, Oxytocin, and Serotonin levels.
- **Sparklines**: Historical trend data for internal chemical states.
- **3D Neural Reservoir**: Interactive Three.js visualization of the active neural topology.
- **Consciousness Stream**: Live feed of the system's thought process and logs.
- **Bidirectional Control**: Send stimuli (text) or actions (Poke, Sleep, Dream) to the ALEPH daemon.

## Setup

1.  **Install dependencies**:
    ```bash
    npm install
    ```

2.  **Run Development Server**:
    ```bash
    npm run dev
    ```

3.  **Build for Production**:
    ```bash
    npm run build
    ```

## Architecture

- **`useAlephSocket`**: Custom hook managing the WebSocket connection to the Rust backend (daemon).
- **`components/`**: Functional React components for each dashboard panel.
- **`hooks/`**: Custom hooks for logic encapsulation.

## Connection

The dashboard connects to `ws://localhost:3030` by default. Ensure the ALEPH backend (`cargo run --release`) is running.
