# 🎵 Gemini Audio MCP

**Gemini Audio MCP** is a high-performance Model Context Protocol (MCP) server that leverages the power of the **Gemini 2.0 Multimodal Live API** to generate high-fidelity, environmental soundscapes on-demand.

---

## 🚀 Mission Statement
Our mission is to provide an immersive, AI-powered audio generation layer for any MCP-compatible environment, enabling the creation of dynamic, seamless, and high-quality environmental audio through simple text prompts.

---

## ✨ Key Features

-   **🌊 Dynamic Soundscapes**: Generate complex environmental audio using the latest Gemini 2.5 Native Audio models.
-   **🎵 Professional Music**: High-fidelity music production via Google's **Lyria 3** models:
    -   **Lyria 3 Pro**: Full song generation with structural coherence ($0.08/req).
    -   **Lyria 3 Clip**: Low-latency clips and rhythmic loops ($0.04/req).
-   **🔁 Infinite Looping**: Seamless, click-free looping with 100ms micro-crossfades.
-   **🔀 Smooth Crossfades**: Transition between two different soundscapes with customizable crossfade durations.
-   **📂 Universal Formats**: Export audio to a variety of formats (WAV, MP3, OGG, FLAC) powered by FFmpeg.
-   **▶️ Auto-play Integration**: Instantly play generated audio through your system's default player upon completion.
-   **⚙️ Persistent Configuration**: Fine-tune default bitrates, sample rates, and durations once and reuse them across sessions.

---

## 🛠 Installation Guide

### Prerequisites
1.  **FFmpeg**: Required for audio conversion and processing.
    -   **macOS**: `brew install ffmpeg`
    -   **Ubuntu/Debian**: `sudo apt install ffmpeg`
    -   **Windows**: Download from [ffmpeg.org](https://ffmpeg.org/download.html).
2.  **Rust Toolchain**: Required for building the project (`cargo`).
3.  **Gemini API Key**: Obtain your key from the [Google AI Studio](https://aistudio.google.com/).

### Setup
1.  Clone the repository:
    ```bash
    git clone https://github.com/mcp-servers/gemini-audio-mcp.git
    cd gemini-audio-mcp
    ```
2.  Build the project:
    ```bash
    cargo build --release
    ```
3.  Configure your environment:
    Set the `GEMINI_API_KEY` environment variable in your MCP client or system.

---

## 🔧 Tool Usage Examples

### Generate a Soundscape
Create an immersive 30-second loop of a cyberpunk rainy city.
```json
{
  "name": "generate_soundscape",
  "arguments": {
    "prompt": "Heavy rain on neon-lit cyberpunk city streets, distant hover-car hums, muffled holographic advertisements.",
    "duration": 30,
    "format": "mp3",
    "auto_play": true
  }
}
```

### Transition Between Environments
Seamlessly shift from a peaceful forest to a roaring thunderstorm.
```json
{
  "name": "transition_soundscape",
  "arguments": {
    "from_prompt": "Quiet morning forest with chirping birds and rustling leaves.",
    "to_prompt": "Intense tropical thunderstorm with loud thunder claps and heavy downpour.",
    "transition_duration": 10,
    "auto_play": true
  }
}
```

### Update Server Defaults
Set the default output format to FLAC for higher quality.
```json
{
  "name": "configure",
  "arguments": {
    "default_format": "flac",
    "default_sample_rate": 48000
  }
}
```

---

## 🏛 Architecture Overview

The server is built with a modular Rust architecture designed for efficiency and reliability:

-   **`main.rs`**: The core MCP protocol engine handling tool registration and request dispatching.
-   **`gemini.rs`**: Manages low-level WebSocket communication with the Gemini 2.0 Multimodal Live API.
-   **`audio.rs`**: Handles PCM data manipulation, including seamless looping algorithms and FFmpeg integration for format transcoding.
-   **`mixer.rs`**: Implements audio processing logic for crossfading and blending multiple audio streams.
-   **`config.rs`**: Provides a persistent JSON-based configuration layer for user preferences.

---

## 📄 License
Distributed under the **MIT License**. See `LICENSE` for more information.
