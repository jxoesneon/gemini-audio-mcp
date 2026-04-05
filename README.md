# 🎵 Gemini Audio MCP

[![Institutional Grade](https://img.shields.io/badge/Quality-Institutional--Grade-blue.svg)](https://github.com/mcp-servers/gemini-audio-mcp)
[![Gemini 2.5](https://img.shields.io/badge/Model-Gemini%202.5%20Live-orange.svg)](https://ai.google.dev/)
[![Lyria 3](https://img.shields.io/badge/Engine-Lyria%203%20Pro-purple.svg)](https://deepmind.google/technologies/lyria/)

**Gemini Audio MCP** is a high-performance Model Context Protocol (MCP) server engineered for professional-grade audio synthesis. It leverages the **Gemini 2.5 Multimodal Live API** and Google DeepMind's **Lyria 3** models to deliver high-fidelity environmental soundscapes, musical compositions, and expressive narration on-demand.

---

## 🛠 Prerequisites

Before deploying the server, ensure your environment meets the following technical requirements:

### 1. FFmpeg (Core Processing Engine)
Required for high-performance audio encoding, decoding, and transcoding.
- **macOS**: `brew install ffmpeg`
- **Windows**: `winget install ffmpeg` or download from [ffmpeg.org](https://ffmpeg.org/).
- **Linux (Ubuntu/Debian)**: `sudo apt update && sudo apt install ffmpeg`

### 2. Rust Toolchain (Compilation)
Required to build the server from source.
- Install via [rustup.rs](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### 3. Node.js & NPM (Runtime)
Required if using the pre-compiled NPM package.
- Version: `node >= 18.0.0`

---

## 🚀 Installation & Deployment

### Global Installation (via NPX)
The fastest way to integrate the server into your MCP client (e.g., Claude Desktop).
```json
{
  "mcpServers": {
    "gemini-audio": {
      "command": "npx",
      "args": ["-y", "gemini-audio-mcp"],
      "env": {
        "GEMINI_API_KEY": "YOUR_SECURE_API_KEY"
      }
    }
  }
}
```

### Manual Build (Optimized)
For maximum performance, build the Rust binary locally:
1.  **Clone & Build**:
    ```bash
    git clone https://github.com/mcp-servers/gemini-audio-mcp.git
    cd gemini-audio-mcp
    cargo build --release
    ```
2.  **Locate Binary**: The optimized binary will be in `./target/release/gemini-audio-mcp`.

---

## 🔑 API Key Management

The server requires a valid Google AI Studio API key.
1.  Obtain your key from [Google AI Studio](https://aistudio.google.com/).
2.  **Security Best Practice**: Never hardcode keys. Inject the key via the `GEMINI_API_KEY` environment variable.
3.  **Tier Note**: Access to Lyria 3 (Pro/Clip) models typically requires a **Paid Tier** or specific preview access in Google AI Studio.

---

## 🎮 Tool Usage Guide

### 1. Environmental Generation (`generate_soundscape`)
Synthesizes immersive, vocal-free ambient textures.
```json
{
  "name": "generate_soundscape",
  "arguments": {
    "prompt": "Deep underwater abyss, low-frequency whale songs, rhythmic air bubbles rising, muffled aquatic pressure.",
    "duration": 60,
    "quality": "high",
    "auto_play": true
  }
}
```

### 2. Professional Music (`generate_music`)
Generates structural compositions with optional vocal control.
```json
{
  "name": "generate_music",
  "arguments": {
    "prompt": "Melancholic solo cello in a vast cathedral with 5-second decay reverb.",
    "bpm": 72,
    "song_key": "D minor",
    "intensity": 4
  }
}
```

### 3. Expressive Voice (`generate_voice`)
Narration and character dialogue using Gemini 2.5 Native Audio.
```json
{
  "name": "generate_voice",
  "arguments": {
    "text": "The artifacts are stable, but the rift remains open.",
    "voice_direction": "Gravelly, urgent, whispered"
  }
}
```

### 4. Dynamic Evolution (`transition_soundscape`)
Crossfades two distinct environments for seamless scene transitions.
```json
{
  "name": "transition_soundscape",
  "arguments": {
    "from_prompt": "Quiet library silence.",
    "to_prompt": "Sudden heavy rain on a tin roof.",
    "transition_duration": 8
  }
}
```

---

## ⚙️ Advanced Parameters

| Parameter | Type | Description |
| :--- | :--- | :--- |
| `seed` | `Integer` | Ensures deterministic, reproducible audio outputs. |
| `image_path` | `String` | **Multimodal**: Uses a local image to guide the acoustic mood (e.g., resonance). |
| `bpm` | `Number` | Explicitly sets the rhythmic tempo (essential for music). |
| `intensity` | `Number` | 1-10 scale controlling dynamic range and complexity. |
| `guidance` | `Number` | 0.0-6.0 scale for prompt adherence (Lyria models). |
| `duration` | `Number` | Target length in seconds. Triggers the **Seamless Looping Engine**. |

---

## 🔬 Architecture Overview

Gemini Audio MCP employs a unique **Hybrid Engine Strategy**:
- **WebSocket Loop**: Connects to Gemini 2.5 Live for low-latency, interactive voice and foley tasks.
- **REST Pipeline**: Interfaces with Lyria 3 Pro for high-fidelity musical synthesis.
- **PCM Processing**: An internal Rust-based loop (`decode -> crossfade -> loop -> encode`) ensures that short clips are transformed into seamless, infinite soundscapes without audible clicks.

---

## 🧪 Troubleshooting

### FFmpeg Errors
- **"FFmpeg not found"**: Ensure `ffmpeg` is in your system PATH. Run `ffmpeg -version` in your terminal to verify.
- **Transcoding Failures**: Check if you have the necessary codecs (e.g., `libmp3lame` for MP3). Most standard FFmpeg installations include these.

### API Issues
- **429 Rate Limit**: The server implements a semaphore to limit concurrency, but ensure your API tier supports the requested model.
- **Empty Audio Output**: Verify your `GEMINI_API_KEY` is correct and that your account has access to the requested model (especially `lyria-3-pro-preview`).

---

## 📄 License
Licensed under the **MIT License**. Engineered with precision by the MCP community.
