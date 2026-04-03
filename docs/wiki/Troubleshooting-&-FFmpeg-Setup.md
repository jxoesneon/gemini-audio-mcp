# 🛠️ Troubleshooting & FFmpeg Setup

FFmpeg is the "heart" of our audio engine. It handles all the heavy lifting for transcoding raw PCM data into your preferred formats.

## 📥 Installing FFmpeg

### 🍎 macOS
The easiest way is via Homebrew:
```bash
brew install ffmpeg
```

### 🐧 Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install ffmpeg
```

### 🪟 Windows
1. Download the latest build from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/).
2. Extract the `bin` folder to a permanent location (e.g., `C:\ffmpeg`).
3. Add `C:\ffmpeg\bin` to your **System PATH**.
4. Restart your terminal/IDE.

---

## ❌ Common Errors

### "FFmpeg not found in PATH"
- **Cause**: The server cannot find the `ffmpeg` executable.
- **Fix**: Run `ffmpeg -version` in your terminal. If it fails there, your installation or PATH configuration is incorrect. If you are using Docker, ensure you are using our provided image which has FFmpeg pre-installed.

### "GEMINI_API_KEY not set"
- **Cause**: The environment variable is missing.
- **Fix**: Ensure your MCP client (Claude Desktop, etc.) is passing the environment variable.
- **Example config**:
  ```json
  "env": {
    "GEMINI_API_KEY": "your_key_here"
  }
  ```

### "Micro-crossfade click"
- **Cause**: Occurs if the generated audio is too short for the requested duration.
- **Fix**: Increase the `duration` parameter. The system needs at least 1-2 seconds of "tail" audio to perform a clean crossfade.
