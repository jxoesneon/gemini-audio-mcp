# Architecture Document

## Components

### 1. MCP Server (`src/main.rs`)
- Exposes tools:
  - `generate_soundscape(prompt, duration, format)`
  - `generate_music(prompt, model, format)`
  - `transition_soundscape(from_prompt, to_prompt, transition_duration, format)`
  - `configure(...)`, `play_audio(path)`, `cleanup_assets()`, `check_dependencies()`
- Handles the JSON-RPC lifecycle and dispatches to the internal modules.

### 2. Gemini & Lyria Client (`src/gemini.rs`)
- **WebSocket (Soundscapes)**: Manages sessions with `models/gemini-2.5-flash-native-audio-latest`. Handles binary JSON/BSON decoding and PCM aggregation.
- **REST (Music)**: Interfaces with Google's Lyria 3 models:
  - `lyria-3-pro-preview`: $0.08 per request (Full songs).
  - `lyria-3-clip-preview`: $0.04 per request (30s clips/loops).

### 3. Audio Processor (`src/audio.rs`)
- **Encoding**: Uses direct Stdin Piping to FFmpeg for high-performance encoding.
- **Transcoding**: High-quality resampling and format conversion via FFmpeg command-line tools.
- **Looping**: Optimized Rust logic to repeat audio samples with 100ms micro-crossfades to meet target durations.

### 4. Mixer Module (`src/mixer.rs`) - *NEW*
- Implements linear or equal-power crossfading between two PCM buffers.
- Leverages `ffmpeg` or manual sample manipulation for smooth transitions.

## Universal Formats Logic
The conversion tool maps the requested format string to the corresponding FFmpeg flags:
- `mp3`: `libmp3lame`
- `ogg`: `libvorbis`
- `opus`: `libopus`
- `flac`: `flac`
- `aac`: `aac`
- `m4a`: `aac` (default) or `alac`
- `aiff`: `pcm_s16be`
- `wma`: `wmav2`
- `ac3`: `ac3`
- `wav`: `pcm_s16le`

## Data Flow
1. Tool Call received via Stdio.
2. `gemini::generate_audio` fetches PCM data.
3. `audio::process` applies looping if `duration` is set.
4. (Optional) `mixer::crossfade` blends two generated buffers.
5. `audio::convert` invokes `ffmpeg` to produce the final file.
6. Returns absolute path.
