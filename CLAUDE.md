# Project Instructions

## Tech Stack
- **Language**: Rust 2021, Node.js >= 18.0.0
- **Libraries**: Tokio (async), reqwest/tungstenite (HTTP/WS), Serde (JSON), Anyhow (error handling), FFmpeg (audio processing)
- **Protocol**: Model Context Protocol (MCP) via JSON-RPC over stdin/stdout

## Code Style
- **Naming**:
  - Files: `kebab-case`
  - Functions/Variables: `snake_case` (Rust)
  - Types/Structs: `PascalCase` (Rust)
- **Error Handling**: `anyhow::Result<T>` with `?` operator, `thiserror` for custom error types
- **Async**: `tokio` runtime for all async operations

## Testing
- **Run tests**: `cargo test`
- **Test pattern**: Unit tests in `mod tests` block at the bottom of each module

## Build & Run
- **Build**: `cargo build --release`
- **Run (binary)**: `./target/release/gemini-audio-mcp`
- **Run (Node.js wrapper)**: `node index.js`
- **Environment**: Set `GEMINI_API_KEY` for API access.
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## Project Structure
- `src/main.rs`: Entry point, MCP JSON-RPC loop, tool dispatch
- `src/audio.rs`: FFmpeg wrapper, PCM encoding, audio cleanup, playback logic
- `src/gemini.rs`: API client for Gemini Live (WebSocket) and Lyria (REST)
- `src/mixer.rs`: Audio processing (crossfade)
- `src/config.rs`: Persistent configuration management
- `index.js`: Node.js wrapper for platform-specific binary execution
- `docs/`: Architecture and design documentation

## Conventions
- **Commits**: Follow [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat:`, `fix:`, `docs:`, `perf:`)
- **PRs**: Squash merges preferred
- **Audio Output**: Files are saved to `~/Library/Application Support/gemini-audio-mcp/audio_outputs` (macOS) or platform-equivalent
