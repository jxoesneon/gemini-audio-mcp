# 🎛️ Audio Formats Reference

Gemini Audio MCP supports a wide variety of formats by leveraging FFmpeg's codec library. Below is a guide on when to use each.

## 📊 Comparison Table

| Format | Extension | Recommended Use Case | Quality |
| :--- | :--- | :--- | :--- |
| **MP3** | `.mp3` | General use, web compatibility | High (Compressed) |
| **OGG** | `.ogg` | Game development (Godot/UE5) | Excellent |
| **FLAC** | `.flac` | Archiving, high-fidelity production | Lossless |
| **OPUS** | `.opus` | Low-latency streaming, VoIP | Best Compression |
| **WAV** | `.wav` | Raw editing, zero compression | Uncompressed |
| **AAC** | `.m4a` | Apple ecosystem, mobile apps | High |

---

## ⚙️ Quality Settings

You can override defaults using the `audio_options` in tool calls:

### 1. Bitrate
Higher bitrates result in better quality but larger files.
- `128k`: Standard quality (good for voice).
- `192k`: High quality (recommended for soundscapes).
- `320k`: Premium quality (recommended for music).

### 2. Sample Rate
The default is **24,000 Hz** (native to Gemini 2.0). 
- If you request `44100` or `48000`, the server will use high-quality resampling filters via FFmpeg to up-mix the audio.

### 3. Channels
- **Mono (1)**: Native Gemini output.
- **Stereo (2)**: The server will duplicate the signal or apply a slight "pseudo-stereo" width if supported by the chosen codec.
