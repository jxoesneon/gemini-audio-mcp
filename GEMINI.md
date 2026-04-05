# 🎵 Gemini Audio MCP - Project Context

## Project Overview
**Gemini Audio MCP** is a high-performance Model Context Protocol (MCP) server that integrates the **Gemini 2.0/3.0 Multimodal Live API** and **Lyria 3** music models into any MCP-compatible environment. It enables high-fidelity, on-demand generation of environmental soundscapes, expressive voice narration, and full musical compositions through simple text prompts.

- **Purpose**: Provide an immersive, AI-powered audio generation layer for game developers, creators, and automated sound design workflows.
- **Main Technologies**:
  - **Rust (2021)**: Core engine handling MCP protocol, WebSocket/REST API communication, and audio processing.
  - **Node.js (>=18)**: Distribution wrapper for platform-specific binary execution.
  - **FFmpeg**: Essential dependency for audio transcoding, PCM encoding, and processing.
  - **Gemini Live API**: Powers low-latency, high-quality environmental soundscapes and native audio generation via WebSockets.
  - **Lyria 3**: High-fidelity music and soundscape production (Lyria 3 Pro/Clip) via REST API. This ensures high-quality audio output without conversational filler.

## Building and Running
### Prerequisites
- **Rust Toolchain**: Required to build the binary.
- **FFmpeg**: Must be installed and available in the system path.
- **Gemini API Key**: Set as `GEMINI_API_KEY` environment variable.

### Commands
- **Build (Release)**: `cargo build --release`
- **Run (NPM Wrapper)**: `node index.js`
- **Run (Direct Binary)**: `./target/release/gemini-audio-mcp`
- **Test**: `cargo test`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## Development Conventions
- **Language**: Rust for the backend logic; Node.js for packaging/CLI wrapper.
- **Error Handling**: Uses `anyhow::Result<T>` with the `?` operator for streamlined error management.
- **Async Runtime**: Powered by `tokio` for high-performance, non-blocking operations.
- **Naming Conventions**:
  - **Rust**: `snake_case` for functions/variables, `PascalCase` for types/structs.
  - **Files**: Single-word or `snake_case` in `src/`.
- **Commits**: Adheres to [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat:`, `fix:`, `docs:`, `perf:`).
- **Persistent Storage**: Configuration and audio assets are stored in platform-specific application support directories (e.g., `~/Library/Application Support/gemini-audio-mcp` on macOS).

## MCP Tools
The server exposes the following tools to MCP clients:
1.  **`generate_soundscape`**: Creates complex environmental audio (uses Gemini 2.0 Live).
2.  **`generate_voice`**: Generates expressive speech/narration (uses Gemini 2.5 Native Audio).
3.  **`generate_music`**: Produces full songs or loops (uses Lyria 3 Pro/Clip).
4.  **`generate_sfx`**: Generates isolated sound effects (uses Lyria 3 Clip).
5.  **`transition_soundscape`**: Creates a crossfaded transition between two prompts.
6.  **`configure`**: View or update persistent settings (format, duration, sample rate, etc.).
7.  **`play_audio`**: Plays a local audio file using the system's default media player.
8.  **`cleanup_assets`**: Manually purge old audio files to save disk space.
9.  **`check_dependencies`**: Verifies if FFmpeg is installed and working.
10. **`generate_custom`**: Full control over engine, model, and raw prompt (no internal prompting).
11. **`list_models`**: Returns a list of available models for each engine.

## Directory Structure
- `src/main.rs`: Entry point, MCP JSON-RPC loop, and tool dispatching.
- `src/audio.rs`: FFmpeg integration, PCM encoding, and file management.
- `src/gemini.rs`: API client for Gemini Live (WebSocket) and Lyria (REST).
- `src/mixer.rs`: Crossfading and blending logic.
- `src/config.rs`: Persistent JSON-based configuration management.
- `index.js`: Cross-platform Node.js wrapper for binary execution.
