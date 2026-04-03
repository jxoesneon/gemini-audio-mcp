# Architecture Document

## Components

### 1. MCP Server (`src/main.rs`)
- Exposes tools:
  - `generate_soundscape(prompt, duration, format)`
  - `transition_soundscape(from_prompt, to_prompt, transition_duration, final_duration, format)`
- Handles the JSON-RPC lifecycle and dispatches to the internal modules.

### 2. Gemini Live Client (`src/gemini.rs`)
- Manages WebSocket sessions with `models/gemini-2.5-flash-native-audio-latest`.
- Handles binary JSON/BSON decoding.
- Aggregates PCM chunks.

### 3. Audio Processor (`src/audio.rs`)
- **Capturing**: Initial capture to WAV via `hound`.
- **Conversion**: Uses `std::process::Command` to invoke `ffmpeg` for fast, high-quality encoding into 10 formats.
- **Looping**: Logic to repeat audio samples to meet a target duration.

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
