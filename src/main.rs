mod audio;
mod config;
mod gemini;
mod mixer;

use config::Config;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Load persistent configuration
    let mut config = Config::load();

    // Concurrency control: limit to 1 concurrent session to prevent 429 errors
    let semaphore = Arc::new(Semaphore::new(1));

    // Startup check for FFmpeg
    if let Err(e) = audio::ensure_ffmpeg() {
        tracing::warn!("{}", e);
    }

    // Automatic asset cleanup on startup
    if let Ok(count) = audio::cleanup_assets(Duration::from_secs(config.auto_cleanup_hours * 3600))
    {
        if count > 0 {
            tracing::info!("Cleaned up {} old audio assets.", count);
        }
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let req: Value = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let response = match method {
            "initialize" => {
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "gemini-audio-mcp",
                        "version": "0.1.0"
                    }
                })
            }
            "notifications/initialized" => {
                continue; // No response for notifications
            }
            "tools/list" => {
                json!({
                    "tools": [
                        {
                            "name": "generate_soundscape",
                            "description": "Generates immersive, high-quality environmental soundscapes (e.g., 'A rainy forest with distant thunder'). Best for background ambience and complex layered textures. Uses Gemini 2.0 Live.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "prompt": {
                                        "type": "string",
                                        "description": "The prompt describing the soundscape."
                                    },
                                    "duration": {
                                        "type": "number",
                                        "description": "Optional duration in seconds. Overrides default if provided."
                                    },
                                    "format": {
                                        "type": "string",
                                        "description": "Optional output format (wav, mp3, ogg, flac, etc.)."
                                    },
                                    "bitrate": { "type": "string" },
                                    "sample_rate": { "type": "number" },
                                    "channels": { "type": "number" },
                                    "auto_play": { "type": "boolean", "description": "If true, automatically plays the generated audio." }
                                },
                                "required": ["prompt"]
                            }
                        },
                        {
                            "name": "generate_voice",
                            "description": "Generates expressive speech and narration from text. Best for scripts, character dialogue, and narration. Uses Gemini 2.5 Native Audio.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "text": {
                                        "type": "string",
                                        "description": "The script or text to be read by the voice."
                                    },
                                    "voice_direction": {
                                        "type": "string",
                                        "description": "Optional instructions for the tone and style (e.g., 'Speak like a fast-talking auctioneer' or 'Use a whispery tone')."
                                    },
                                    "format": {
                                        "type": "string",
                                        "description": "Optional output format (wav, mp3, ogg, flac, etc.)."
                                    },
                                    "auto_play": { "type": "boolean", "description": "If true, automatically plays the generated audio." }
                                },
                                "required": ["text"]
                            }
                        },
                        {
                            "name": "generate_music",
                            "description": "Generates full songs, loops, or musical segments. Best for melodic content, rhythm, and structured compositions. (PAID MODELS - Pro: $0.08, Clip: $0.04).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "prompt": {
                                        "type": "string",
                                        "description": "Detailed description of the music (e.g., 'An upbeat jazz track with a fast tempo')."
                                    },
                                    "model": {
                                        "type": "string",
                                        "description": "The Lyria model to use. 'lyria-3-pro-preview' (Full songs, $0.08) or 'lyria-3-clip-preview' (30s clips, $0.04). Defaults to Pro.",
                                        "enum": ["lyria-3-pro-preview", "lyria-3-clip-preview"]
                                    },
                                    "format": {
                                        "type": "string",
                                        "description": "Output format (wav, mp3, flac)."
                                    },
                                    "auto_play": { "type": "boolean" }
                                },
                                "required": ["prompt"]
                            }
                        },
                        {
                            "name": "generate_sfx",
                            "description": "Generates isolated, short-duration sound effects and foley (e.g., 'A laser blast' or 'Footsteps on gravel'). Best for specific one-shot audio cues. Uses Lyria-3-clip-preview ($0.04/req).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "prompt": {
                                        "type": "string",
                                        "description": "Description of the sound effect (e.g., 'A heavy metallic door slamming shut')."
                                    },
                                    "format": {
                                        "type": "string",
                                        "description": "Output format (wav, mp3, flac)."
                                    },
                                    "auto_play": { "type": "boolean" }
                                },
                                "required": ["prompt"]
                            }
                        },
                        {
                            "name": "transition_soundscape",
                            "description": "Generates two distinct soundscapes and creates a smooth crossfade transition between them. Ideal for evolving scenes or changing environments.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "from_prompt": { "type": "string" },
                                    "to_prompt": { "type": "string" },
                                    "transition_duration": { "type": "number", "description": "Duration of the crossfade in seconds. Defaults to config value." },
                                    "format": { "type": "string" },
                                    "auto_play": { "type": "boolean", "description": "If true, automatically plays the generated audio." }
                                },
                                "required": ["from_prompt", "to_prompt"]
                            }
                        },
                        {
                            "name": "configure",
                            "description": "View or update persistent server settings like default audio format, sample rate, and automatic cleanup intervals. Call with no arguments to see current values.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "default_format": { "type": "string", "description": "Default file extension (e.g., 'mp3')." },
                                    "default_duration": { "type": "number", "description": "Default duration for soundscapes in seconds." },
                                    "default_bitrate": { "type": "string" },
                                    "default_sample_rate": { "type": "number" },
                                    "default_channels": { "type": "number" },
                                    "default_transition_duration": { "type": "number" },
                                    "auto_cleanup_hours": { "type": "number", "description": "How often to clean up old audio files (in hours)." }
                                }
                            }
                        },
                        {
                            "name": "play_audio",
                            "description": "Plays any local audio file using the system's default media player (e.g., 'afplay' on macOS).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "The absolute path to the audio file." }
                                },
                                "required": ["path"]
                            }
                        },
                        {
                            "name": "cleanup_assets",
                            "description": "Manually trigger deletion of generated audio assets that exceed a certain age (in hours) to save disk space.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "max_age_hours": { "type": "number", "description": "Files older than this will be deleted." }
                                }
                            }
                        },
                        {
                            "name": "check_dependencies",
                            "description": "Verifies that the system has required external tools like FFmpeg installed and accessible.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        }
                    ]
                })
            }
            "tools/call" => {
                let params = req.get("params");
                let name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let arguments = params.and_then(|p| p.get("arguments"));

                match name {
                    "configure" => {
                        if let Some(args) = arguments {
                            if !args.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                                // Update config
                                if let Some(v) = args.get("default_format").and_then(|v| v.as_str())
                                {
                                    config.default_format = v.to_string();
                                }
                                if let Some(v) = args.get("default_duration") {
                                    config.default_duration = v.as_u64().map(|v| v as u32);
                                }
                                if let Some(v) =
                                    args.get("default_bitrate").and_then(|v| v.as_str())
                                {
                                    config.default_bitrate = Some(v.to_string());
                                }
                                if let Some(v) = args.get("default_sample_rate") {
                                    config.default_sample_rate = v.as_u64().map(|v| v as u32);
                                }
                                if let Some(v) = args.get("default_channels") {
                                    config.default_channels = v.as_u64().map(|v| v as u32);
                                }
                                if let Some(v) = args.get("default_transition_duration") {
                                    config.default_transition_duration =
                                        v.as_u64().unwrap_or(5) as u32;
                                }
                                if let Some(v) = args.get("auto_cleanup_hours") {
                                    config.auto_cleanup_hours = v.as_u64().unwrap_or(24);
                                }

                                match config.save() {
                                    Ok(_) => {
                                        json!({"content": [{"type": "text", "text": format!("Configuration updated successfully:\n{}", serde_json::to_string_pretty(&config).unwrap())}]})
                                    }
                                    Err(e) => {
                                        json!({"isError": true, "content": [{"type": "text", "text": format!("Failed to save config: {}", e)}]})
                                    }
                                }
                            } else {
                                // List config
                                json!({"content": [{"type": "text", "text": format!("Current Configuration:\n{}\n\nYou can update these by passing them as arguments to this tool.", serde_json::to_string_pretty(&config).unwrap())}]})
                            }
                        } else {
                            json!({"content": [{"type": "text", "text": format!("Current Configuration:\n{}", serde_json::to_string_pretty(&config).unwrap())}]})
                        }
                    }
                    "generate_music" => {
                        let _permit = semaphore.acquire().await;
                        let prompt = arguments.and_then(|a| a.get("prompt")).and_then(|p| p.as_str()).unwrap_or("");
                        let model = arguments.and_then(|a| a.get("model")).and_then(|m| m.as_str()).unwrap_or("lyria-3-pro-preview");
                        let format = arguments.and_then(|a| a.get("format")).and_then(|f| f.as_str()).unwrap_or(&config.default_format);
                        let auto_play = arguments.and_then(|a| a.get("auto_play")).and_then(|v| v.as_bool()).unwrap_or(false);

                        let audio_options = audio::AudioOptions {
                            bitrate: arguments.and_then(|a| a.get("bitrate")).and_then(|v| v.as_str()).map(|s| s.to_string()).or(config.default_bitrate.clone()),
                            sample_rate: arguments.and_then(|a| a.get("sample_rate")).and_then(|v| v.as_u64()).map(|v| v as u32).or(config.default_sample_rate),
                            channels: arguments.and_then(|a| a.get("channels")).and_then(|v| v.as_u64()).map(|v| v as u32).or(config.default_channels),
                        };

                        if prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Prompt is empty."}]})
                        } else {
                            match gemini::generate_music(prompt, model).await {
                                Ok((audio_bytes, mime_type, description)) => {
                                    let ext = if mime_type.contains("wav") { "wav" } else { "mp3" };
                                    let final_path = if format != ext || arguments.map(|a| a.as_object().unwrap().len() > 2).unwrap_or(false) {
                                        match audio::transcode_encoded(&audio_bytes, format, Some(audio_options)) {
                                            Ok(p) => p,
                                            Err(_) => {
                                                let out_dir = audio::get_output_dir().unwrap();
                                                let p = out_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), ext));
                                                std::fs::write(&p, &audio_bytes).unwrap();
                                                p.to_string_lossy().to_string()
                                            }
                                        }
                                    } else {
                                        let out_dir = audio::get_output_dir().unwrap();
                                        let p = out_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), ext));
                                        std::fs::write(&p, &audio_bytes).unwrap();
                                        p.to_string_lossy().to_string()
                                    };

                                    if auto_play { let _ = audio::play_audio_file(&final_path); }
                                    let result = json!({ "path": final_path, "format": format, "model": model, "description": description });
                                    json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&result).unwrap()}]})
                                }
                                Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini Lyria error: {}", e)}]}),
                            }
                        }
                    }
                    "generate_sfx" => {
                        let _permit = semaphore.acquire().await;
                        let prompt = arguments.and_then(|a| a.get("prompt")).and_then(|p| p.as_str()).unwrap_or("");
                        let format = arguments.and_then(|a| a.get("format")).and_then(|f| f.as_str()).unwrap_or(&config.default_format);
                        let auto_play = arguments.and_then(|a| a.get("auto_play")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let audio_options = audio::AudioOptions {
                            bitrate: arguments.and_then(|a| a.get("bitrate")).and_then(|v| v.as_str()).map(|s| s.to_string()).or(config.default_bitrate.clone()),
                            sample_rate: arguments.and_then(|a| a.get("sample_rate")).and_then(|v| v.as_u64()).map(|v| v as u32).or(config.default_sample_rate),
                            channels: arguments.and_then(|a| a.get("channels")).and_then(|v| v.as_u64()).map(|v| v as u32).or(config.default_channels),
                        };

                        if prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Prompt is empty."}]})
                        } else {
                            match gemini::generate_music(prompt, "lyria-3-clip-preview").await {
                                Ok((audio_bytes, mime_type, description)) => {
                                    let ext = if mime_type.contains("wav") { "wav" } else { "mp3" };
                                    let final_path = if format != ext || arguments.map(|a| a.as_object().unwrap().len() > 1).unwrap_or(false) {
                                        match audio::transcode_encoded(&audio_bytes, format, Some(audio_options)) {
                                            Ok(p) => p,
                                            Err(_) => {
                                                let out_dir = audio::get_output_dir().unwrap();
                                                let p = out_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), ext));
                                                std::fs::write(&p, &audio_bytes).unwrap();
                                                p.to_string_lossy().to_string()
                                            }
                                        }
                                    } else {
                                        let out_dir = audio::get_output_dir().unwrap();
                                        let p = out_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), ext));
                                        std::fs::write(&p, &audio_bytes).unwrap();
                                        p.to_string_lossy().to_string()
                                    };
                                    if auto_play { let _ = audio::play_audio_file(&final_path); }
                                    let result = json!({ "path": final_path, "format": format, "model": "lyria-3-clip-preview", "description": description });
                                    json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&result).unwrap()}]})
                                }
                                Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini SFX error: {}", e)}]}),
                            }
                        }
                    }
                    "generate_voice" => {
                        let _permit = semaphore.acquire().await;
                        let text = arguments.and_then(|a| a.get("text")).and_then(|p| p.as_str()).unwrap_or("");
                        let direction = arguments.and_then(|a| a.get("voice_direction")).and_then(|p| p.as_str()).unwrap_or("");
                        let format = arguments.and_then(|a| a.get("format")).and_then(|f| f.as_str()).unwrap_or(&config.default_format);
                        let auto_play = arguments.and_then(|a| a.get("auto_play")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let audio_options = audio::AudioOptions {
                            bitrate: arguments.and_then(|a| a.get("bitrate")).and_then(|v| v.as_str()).map(|s| s.to_string()).or(config.default_bitrate.clone()),
                            sample_rate: arguments.and_then(|a| a.get("sample_rate")).and_then(|v| v.as_u64()).map(|v| v as u32).or(config.default_sample_rate),
                            channels: arguments.and_then(|a| a.get("channels")).and_then(|v| v.as_u64()).map(|v| v as u32).or(config.default_channels),
                        };

                        if text.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Text is empty."}]})
                        } else {
                            let prompt = if direction.is_empty() {
                                format!("Read the following text: {}", text)
                            } else {
                                format!("Text to read: {}\nVoice Direction: {}", text, direction)
                            };

                            match gemini::generate_audio(&prompt, None).await {
                                Ok((pcm_data, description)) => {
                                    match audio::encode_pcm(&pcm_data, format, Some(audio_options)) {
                                        Ok(p) => {
                                            if auto_play { let _ = audio::play_audio_file(&p); }
                                            let result = json!({ "path": p, "format": format, "description": description });
                                            json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&result).unwrap()}]})
                                        }
                                        Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Encoding error: {}", e)}]}),
                                    }
                                }
                                Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini error: {}", e)}]}),
                            }
                        }
                    }
                    "generate_soundscape" => {
                        let _permit = semaphore.acquire().await;
                        let prompt = arguments.and_then(|a| a.get("prompt")).and_then(|p| p.as_str()).unwrap_or("");
                        let duration = arguments.and_then(|a| a.get("duration")).and_then(|d| d.as_u64()).map(|d| d as u32).or(config.default_duration);
                        let format = arguments.and_then(|a| a.get("format")).and_then(|f| f.as_str()).unwrap_or(&config.default_format);
                        let auto_play = arguments.and_then(|a| a.get("auto_play")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let audio_options = audio::AudioOptions {
                            bitrate: arguments.and_then(|a| a.get("bitrate")).and_then(|v| v.as_str()).map(|s| s.to_string()).or(config.default_bitrate.clone()),
                            sample_rate: arguments.and_then(|a| a.get("sample_rate")).and_then(|v| v.as_u64()).map(|v| v as u32).or(config.default_sample_rate),
                            channels: arguments.and_then(|a| a.get("channels")).and_then(|v| v.as_u64()).map(|v| v as u32).or(config.default_channels),
                        };

                        if prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Prompt is empty."}]})
                        } else {
                            match gemini::generate_audio(prompt, duration).await {
                                Ok((mut pcm_data, description)) => {
                                    if let Some(d) = duration { pcm_data = audio::seamless_loop(pcm_data, d as f32); }
                                    let actual_duration = (pcm_data.len() as f64) / (audio::SAMPLE_RATE as f64 * audio::BYTES_PER_SAMPLE as f64);
                                    match audio::encode_pcm(&pcm_data, format, Some(audio_options.clone())) {
                                        Ok(p) => {
                                            if auto_play { let _ = audio::play_audio_file(&p); }
                                            let result = json!({ "path": p, "format": format, "duration_seconds": actual_duration, "sample_rate": audio_options.sample_rate.unwrap_or(audio::SAMPLE_RATE), "bitrate": audio_options.bitrate.clone().unwrap_or_else(|| "default".to_string()), "description": description });
                                            json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&result).unwrap()}]})
                                        }
                                        Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Encoding error: {}", e)}]}),
                                    }
                                }
                                Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini error: {}", e)}]}),
                            }
                        }
                    }
                    "transition_soundscape" => {
                        let _permit = semaphore.acquire().await;
                        let from_prompt = arguments.and_then(|a| a.get("from_prompt")).and_then(|p| p.as_str()).unwrap_or("");
                        let to_prompt = arguments.and_then(|a| a.get("to_prompt")).and_then(|p| p.as_str()).unwrap_or("");
                        let transition_duration = arguments.and_then(|a| a.get("transition_duration")).and_then(|d| d.as_u64()).map(|d| d as u32).unwrap_or(config.default_transition_duration);
                        let format = arguments.and_then(|a| a.get("format")).and_then(|f| f.as_str()).unwrap_or(&config.default_format);
                        let auto_play = arguments.and_then(|a| a.get("auto_play")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let audio_options = audio::AudioOptions {
                            bitrate: config.default_bitrate.clone(),
                            sample_rate: config.default_sample_rate,
                            channels: config.default_channels,
                        };

                        if from_prompt.is_empty() || to_prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Missing prompts."}]})
                        } else {
                            match tokio::try_join!(gemini::generate_audio(from_prompt, None), gemini::generate_audio(to_prompt, None)) {
                                Ok(((pcm1, desc1), (pcm2, desc2))) => {
                                    let transition_samples = (transition_duration * audio::SAMPLE_RATE) as usize;
                                    let mixed_pcm = mixer::crossfade(&pcm1, &pcm2, transition_samples);
                                    let actual_duration = (mixed_pcm.len() as f64) / (audio::SAMPLE_RATE as f64 * audio::BYTES_PER_SAMPLE as f64);
                                    match audio::encode_pcm(&mixed_pcm, format, Some(audio_options.clone())) {
                                        Ok(p) => {
                                            if auto_play { let _ = audio::play_audio_file(&p); }
                                            let result = json!({ "path": p, "format": format, "duration_seconds": actual_duration, "sample_rate": audio_options.sample_rate.unwrap_or(audio::SAMPLE_RATE), "bitrate": audio_options.bitrate.clone().unwrap_or_else(|| "default".to_string()), "description": format!("Transition from: {}\nTo: {}", desc1, desc2) });
                                            json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&result).unwrap()}]})
                                        }
                                        Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Encoding error: {}", e)}]}),
                                    }
                                }
                                Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini error: {}", e)}]}),
                            }
                        }
                    }
                    "play_audio" => {
                        let path = arguments.and_then(|a| a.get("path")).and_then(|p| p.as_str()).unwrap_or("");
                        match audio::play_audio_file(path) {
                            Ok(_) => json!({"content": [{"type": "text", "text": "Playback started."}]}),
                            Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Playback error: {}", e)}]}),
                        }
                    }
                    "cleanup_assets" => {
                        let max_age_hours = arguments.and_then(|a| a.get("max_age_hours")).and_then(|v| v.as_u64()).unwrap_or(config.auto_cleanup_hours);
                        match audio::cleanup_assets(Duration::from_secs(max_age_hours * 3600)) {
                            Ok(count) => json!({"content": [{"type": "text", "text": format!("Deleted {} old files.", count)}]}),
                            Err(e) => json!({"isError": true, "content": [{"type": "text", "text": format!("Cleanup error: {}", e)}]}),
                        }
                    }
                    "check_dependencies" => match audio::ensure_ffmpeg() {
                        Ok(_) => json!({"content": [{"type": "text", "text": "Dependencies OK."}]}),
                        Err(e) => json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]}),
                    },
                    _ => json!({"isError": true, "content": [{"type": "text", "text": format!("Unknown tool: {}", name)}]}),
                }
            }
            _ => {
                if id.is_some() { json!({"error": {"code": -32601, "message": "Method not found"}}) } else { continue; }
            }
        };

        if let Some(req_id) = id {
            let res = if response.get("error").is_some() {
                json!({"jsonrpc": "2.0", "id": req_id, "error": response["error"]})
            } else {
                json!({"jsonrpc": "2.0", "id": req_id, "result": response})
            };
            let res_str = serde_json::to_string(&res).unwrap();
            writeln!(stdout, "{}", res_str).unwrap();
            stdout.flush().unwrap();
        }
    }
    Ok(())
}
