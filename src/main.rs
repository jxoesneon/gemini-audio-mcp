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

/// Appends musical metadata (BPM, Key, Intensity, etc.) to a raw prompt to guide the Lyria model.
fn bake_lyria_prompt(
    raw_prompt: &str,
    bpm: Option<f64>,
    key: Option<&str>,
    intensity: Option<f64>,
    guidance: Option<f64>,
    lyrics: Option<&str>,
    vocal_profile: Option<&str>,
) -> String {
    let mut prompt = raw_prompt.to_string();

    if let Some(profile) = vocal_profile {
        prompt = format!("Singer profile: {}. {}", profile, prompt);
    }

    if let Some(b) = bpm {
        prompt.push_str(&format!(" {} BPM.", b));
    }

    if let Some(k) = key {
        prompt.push_str(&format!(" Key: {}.", k));
    }

    if let Some(g) = guidance {
        prompt.push_str(&format!(" Guidance: {}.", g));
    }

    if let Some(i) = intensity {
        let descriptor = match i {
            v if v <= 2.0 => "(Very low)",
            v if v <= 4.0 => "(Low)",
            v if v <= 6.0 => "(Moderate)",
            v if v <= 8.0 => "(High)",
            _ => "(Maximum)",
        };
        prompt.push_str(&format!(" Intensity: {}/10 {}.", i, descriptor));
    }

    if let Some(l) = lyrics {
        prompt.push_str(&format!("\nLyrics: {}", l));
    }

    prompt
}

/// Reads a local image file and returns its Base64-encoded representation for multimodal prompts.
fn encode_image_base64(path: &str) -> Option<String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    std::fs::read(path).ok().map(|bytes| STANDARD.encode(bytes))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

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
                            "description": "Generates immersive, high-quality environmental soundscapes. (PAID MODELS). Use 'high' quality for 3-minute professional foley.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "prompt": {
                                        "type": "string",
                                        "description": "The prompt describing the soundscape (e.g., 'A rainy forest')."
                                    },
                                    "quality": {
                                        "type": "string",
                                        "description": "Audio quality and duration level. 'mid' (30s clips) or 'high' (up to 3m). Defaults to 'mid'.",
                                        "enum": ["mid", "high"]
                                    },
                                    "bpm": { "type": "number", "description": "Beats per minute. Set to 0 or low values to suppress rhythmic pumping in soundscapes." },
                                    "song_key": { "type": "string", "description": "Harmonic center (e.g., 'A major', 'D minor')." },
                                    "intensity": { "type": "number", "description": "Dynamic energy level (1-10)." },
                                    "image_path": { "type": "string", "description": "Optional local path to an image to guide the acoustic mood." },
                                    "seed": { "type": "integer", "description": "Integer for deterministic reproducibility." },
                                    "duration": { "type": "number", "description": "Optional target duration in seconds. Uses seamless looping to meet length." },
                                    "format": { "type": "string" },
                                    "auto_play": { "type": "boolean" }
                                },
                                "required": ["prompt"]
                            }
                        },
                        {
                            "name": "generate_voice",
                            "description": "Generates expressive speech and narration from text. Uses Gemini 2.5 Native Audio.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "text": { "type": "string" },
                                    "voice_direction": { "type": "string", "description": "e.g., 'Whispery', 'Excited', 'Fast-talking'." },
                                    "format": { "type": "string" },
                                    "auto_play": { "type": "boolean" }
                                },
                                "required": ["text"]
                            }
                        },
                        {
                            "name": "generate_music",
                            "description": "Generates high-fidelity musical compositions. (PAID MODELS).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "prompt": { "type": "string", "description": "e.g., 'Upbeat jazz track'." },
                                    "lyrics": { "type": "string", "description": "Text for the model to sing. Use (brackets) for backing vocals." },
                                    "quality": {
                                        "type": "string",
                                        "description": "Audio quality and duration level. 'mid' (30s) or 'high' (up to 3m).",
                                        "enum": ["mid", "high"]
                                    },
                                    "bpm": { "type": "number", "description": "Target beats per minute." },
                                    "song_key": { "type": "string", "description": "Musical key (e.g., 'C major')." },
                                    "intensity": { "type": "number", "description": "1-10 scale." },
                                    "guidance": { "type": "number", "description": "Prompt adherence (0.0-6.0)." },
                                    "temperature": { "type": "number", "description": "Randomness/Creativity (0.0-2.0)." },
                                    "candidate_count": { "type": "integer", "description": "Number of audio variations to generate (default 1)." },
                                    "vocal_profile": { "type": "string", "description": "Singer profile (e.g., 'Breathy female soprano')." },
                                    "image_path": { "type": "string", "description": "Optional local path to an image to guide visual-to-audio synthesis." },
                                    "seed": { "type": "integer" },
                                    "negative_prompt": { "type": "string", "description": "Acoustic elements to EXCLUDE (e.g., 'drums, fast tempo')." },
                                    "format": { "type": "string" },
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
                        },
                        {
                            "name": "generate_custom",
                            "description": "Advanced tool for full control over audio generation. No internal prompt engineering is applied. Agents must provide their own positive/negative prompts.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "prompt": {
                                        "type": "string",
                                        "description": "The RAW text prompt. No prefixes or suffixes will be added."
                                    },
                                    "engine": {
                                        "type": "string",
                                        "description": "The generation engine to use.",
                                        "enum": ["lyria", "gemini-live"]
                                    },
                                    "model": {
                                        "type": "string",
                                        "description": "The specific model name (e.g., 'lyria-3-pro-preview', 'gemini-2.5-flash-native-audio-latest'). Defaults based on engine."
                                    },
                                    "duration": {
                                        "type": "number",
                                        "description": "Optional target duration in seconds. Uses seamless looping if supported by engine output."
                                    },
                                    "format": { "type": "string" },
                                    "bitrate": { "type": "string" },
                                    "sample_rate": { "type": "number" },
                                    "channels": { "type": "number" },
                                    "auto_play": { "type": "boolean" },
                                    "seed": { "type": "integer" },
                                    "negative_prompt": { "type": "string" },
                                    "image_path": { "type": "string" },
                                    "bpm": { "type": "number" },
                                    "song_key": { "type": "string" },
                                    "intensity": { "type": "number" },
                                    "guidance": { "type": "number" },
                                    "temperature": { "type": "number" },
                                    "candidate_count": { "type": "integer" }
                                },
                                "required": ["prompt", "engine"]
                            }
                        },
                        {
                            "name": "list_models",
                            "description": "Returns a comprehensive list of available audio generation models categorized by engine.",
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
                        let raw_prompt = arguments
                            .and_then(|a| a.get("prompt"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let quality = arguments
                            .and_then(|a| a.get("quality"))
                            .and_then(|m| m.as_str())
                            .unwrap_or("high");

                        let model = match quality {
                            "mid" => "lyria-3-clip-preview",
                            _ => "lyria-3-pro-preview",
                        };

                        let bpm = arguments
                            .and_then(|a| a.get("bpm"))
                            .and_then(|v| v.as_f64());
                        let key = arguments
                            .and_then(|a| a.get("song_key"))
                            .and_then(|v| v.as_str());
                        let intensity = arguments
                            .and_then(|a| a.get("intensity"))
                            .and_then(|v| v.as_f64());
                        let lyrics = arguments
                            .and_then(|a| a.get("lyrics"))
                            .and_then(|v| v.as_str());
                        let profile = arguments
                            .and_then(|a| a.get("vocal_profile"))
                            .and_then(|v| v.as_str());
                        let image_path = arguments
                            .and_then(|a| a.get("image_path"))
                            .and_then(|v| v.as_str());
                        let seed = arguments
                            .and_then(|a| a.get("seed"))
                            .and_then(|v| v.as_i64());
                        let negative_prompt = arguments
                            .and_then(|a| a.get("negative_prompt"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let guidance = arguments
                            .and_then(|a| a.get("guidance"))
                            .and_then(|v| v.as_f64());
                        let temperature = arguments
                            .and_then(|a| a.get("temperature"))
                            .and_then(|v| v.as_f64());
                        let candidate_count = arguments
                            .and_then(|a| a.get("candidate_count"))
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32);

                        let format = arguments
                            .and_then(|a| a.get("format"))
                            .and_then(|f| f.as_str())
                            .unwrap_or(&config.default_format);
                        let auto_play = arguments
                            .and_then(|a| a.get("auto_play"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        let audio_options = audio::AudioOptions {
                            bitrate: arguments
                                .and_then(|a| a.get("bitrate"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .or(config.default_bitrate.clone()),
                            sample_rate: arguments
                                .and_then(|a| a.get("sample_rate"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_sample_rate),
                            channels: arguments
                                .and_then(|a| a.get("channels"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_channels),
                        };

                        if raw_prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Prompt is empty."}]})
                        } else {
                            let prompt = bake_lyria_prompt(
                                raw_prompt, bpm, key, intensity, guidance, lyrics, profile,
                            );
                            let lyria_opts = gemini::LyriaOptions {
                                seed,
                                negative_prompt,
                                image_data: image_path.and_then(encode_image_base64),
                                temperature,
                                candidate_count,
                            };

                            match gemini::generate_music(&prompt, model, Some(lyria_opts)).await {
                                Ok((audio_bytes, mime_type, description)) => {
                                    let ext = if mime_type.contains("wav") {
                                        "wav"
                                    } else {
                                        "mp3"
                                    };
                                    let final_path = if format != ext
                                        || arguments
                                            .and_then(|a| a.as_object())
                                            .map(|o| o.len() > 2)
                                            .unwrap_or(false)
                                    {
                                        match audio::transcode_encoded(
                                            &audio_bytes,
                                            format,
                                            Some(audio_options),
                                        ) {
                                            Ok(p) => p,
                                            Err(e) => {
                                                tracing::error!("Transcoding failed, falling back to original: {}", e);
                                                audio::save_audio(&audio_bytes, ext).unwrap_or_default()
                                            }
                                        }
                                    } else {
                                        audio::save_audio(&audio_bytes, ext).unwrap_or_default()
                                    };

                                    if final_path.is_empty() {
                                        json!({"isError": true, "content": [{"type": "text", "text": "Failed to save audio file."}]})
                                    } else {
                                        if auto_play {
                                            let _ = audio::play_audio_file(&final_path);
                                        }
                                        let text_output = format!("✅ Music generated successfully!\n\nFile Path: {}\nQuality: {}\nModel: {}\nDescription: {}\n\nFormat: {}", final_path, quality, model, description, format);
                                        json!({"content": [{"type": "text", "text": text_output}]})
                                    }
                                }
                                Err(e) => {
                                    json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini Lyria error: {}", e)}]})
                                }
                            }
                        }
                    }
                    "generate_sfx" => {
                        let _permit = semaphore.acquire().await;
                        let raw_prompt = arguments
                            .and_then(|a| a.get("prompt"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let format = arguments
                            .and_then(|a| a.get("format"))
                            .and_then(|f| f.as_str())
                            .unwrap_or(&config.default_format);
                        let auto_play = arguments
                            .and_then(|a| a.get("auto_play"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let audio_options = audio::AudioOptions {
                            bitrate: arguments
                                .and_then(|a| a.get("bitrate"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .or(config.default_bitrate.clone()),
                            sample_rate: arguments
                                .and_then(|a| a.get("sample_rate"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_sample_rate),
                            channels: arguments
                                .and_then(|a| a.get("channels"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_channels),
                        };

                        if raw_prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Prompt is empty."}]})
                        } else {
                            // Aggressive prompts for SFX to ensure isolation and no background music/voices.
                            let prompt = format!("{}, ISOLATED SOUND EFFECT. HIGH-FIDELITY FOLEY. NO BACKGROUND MUSIC. NO VOICES. NO SINGING. NO HUMMING. PURE ONE-SHOT SOUND EFFECT.", raw_prompt);

                            match gemini::generate_music(&prompt, "lyria-3-clip-preview", None)
                                .await
                            {
                                Ok((audio_bytes, mime_type, description)) => {
                                    let ext = if mime_type.contains("wav") {
                                        "wav"
                                    } else {
                                        "mp3"
                                    };
                                    let final_path = if format != ext
                                        || arguments
                                            .and_then(|a| a.as_object())
                                            .map(|o| o.len() > 1)
                                            .unwrap_or(false)
                                    {
                                        match audio::transcode_encoded(
                                            &audio_bytes,
                                            format,
                                            Some(audio_options),
                                        ) {
                                            Ok(p) => p,
                                            Err(e) => {
                                                tracing::error!("Transcoding failed, falling back to original: {}", e);
                                                audio::save_audio(&audio_bytes, ext).unwrap_or_default()
                                            }
                                        }
                                    } else {
                                        audio::save_audio(&audio_bytes, ext).unwrap_or_default()
                                    };

                                    if final_path.is_empty() {
                                        json!({"isError": true, "content": [{"type": "text", "text": "Failed to save SFX file."}]})
                                    } else {
                                        if auto_play {
                                            let _ = audio::play_audio_file(&final_path);
                                        }
                                        let text_output = format!("✅ SFX generated successfully!\n\nFile Path: {}\nModel: Lyria-3-clip-preview\nDescription: {}\n\nFormat: {}", final_path, description, format);
                                        json!({"content": [{"type": "text", "text": text_output}]})
                                    }
                                }
                                Err(e) => {
                                    json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini SFX error: {}", e)}]})
                                }
                            }
                        }
                    }
                    "generate_voice" => {
                        let _permit = semaphore.acquire().await;
                        let text = arguments
                            .and_then(|a| a.get("text"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let direction = arguments
                            .and_then(|a| a.get("voice_direction"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let format = arguments
                            .and_then(|a| a.get("format"))
                            .and_then(|f| f.as_str())
                            .unwrap_or(&config.default_format);
                        let auto_play = arguments
                            .and_then(|a| a.get("auto_play"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let audio_options = audio::AudioOptions {
                            bitrate: arguments
                                .and_then(|a| a.get("bitrate"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .or(config.default_bitrate.clone()),
                            sample_rate: arguments
                                .and_then(|a| a.get("sample_rate"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_sample_rate),
                            channels: arguments
                                .and_then(|a| a.get("channels"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_channels),
                        };

                        if text.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Text is empty."}]})
                        } else {
                            let prompt = if direction.is_empty() {
                                format!("Read the following text: {}", text)
                            } else {
                                format!("Text to read: {}\nVoice Direction: {}", text, direction)
                            };

                            match gemini::generate_audio(&prompt, None, None).await {
                                Ok((pcm_data, description)) => {
                                    match audio::encode_pcm(&pcm_data, format, Some(audio_options))
                                    {
                                        Ok(p) => {
                                            if auto_play {
                                                let _ = audio::play_audio_file(&p);
                                            }
                                            let text_output = format!("✅ Voice generated successfully!\n\nFile Path: {}\nDescription: {}\n\nFormat: {}", p, description, format);
                                            json!({"content": [{"type": "text", "text": text_output}]})
                                        }
                                        Err(e) => {
                                            json!({"isError": true, "content": [{"type": "text", "text": format!("Encoding error: {}", e)}]})
                                        }
                                    }
                                }
                                Err(e) => {
                                    json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini error: {}", e)}]})
                                }
                            }
                        }
                    }
                    "generate_soundscape" => {
                        let _permit = semaphore.acquire().await;
                        let raw_prompt = arguments
                            .and_then(|a| a.get("prompt"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");

                        let quality = arguments
                            .and_then(|a| a.get("quality"))
                            .and_then(|m| m.as_str())
                            .unwrap_or("mid");

                        let model = match quality {
                            "high" => "lyria-3-pro-preview",
                            _ => "lyria-3-clip-preview",
                        };

                        let duration = arguments
                            .and_then(|a| a.get("duration"))
                            .and_then(|d| d.as_u64())
                            .map(|d| d as u32)
                            .or(config.default_duration);

                        let bpm = arguments
                            .and_then(|a| a.get("bpm"))
                            .and_then(|v| v.as_f64());
                        let key = arguments
                            .and_then(|a| a.get("song_key"))
                            .and_then(|v| v.as_str());
                        let intensity = arguments
                            .and_then(|a| a.get("intensity"))
                            .and_then(|v| v.as_f64());
                        let image_path = arguments
                            .and_then(|a| a.get("image_path"))
                            .and_then(|v| v.as_str());
                        let seed = arguments
                            .and_then(|a| a.get("seed"))
                            .and_then(|v| v.as_i64());
                        let guidance = arguments
                            .and_then(|a| a.get("guidance"))
                            .and_then(|v| v.as_f64());
                        let temperature = arguments
                            .and_then(|a| a.get("temperature"))
                            .and_then(|v| v.as_f64());
                        let candidate_count = arguments
                            .and_then(|a| a.get("candidate_count"))
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32);

                        let format = arguments
                            .and_then(|a| a.get("format"))
                            .and_then(|f| f.as_str())
                            .unwrap_or(&config.default_format);
                        let auto_play = arguments
                            .and_then(|a| a.get("auto_play"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let audio_options = audio::AudioOptions {
                            bitrate: arguments
                                .and_then(|a| a.get("bitrate"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .or(config.default_bitrate.clone()),
                            sample_rate: arguments
                                .and_then(|a| a.get("sample_rate"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_sample_rate),
                            channels: arguments
                                .and_then(|a| a.get("channels"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_channels),
                        };

                        if raw_prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Prompt is empty."}]})
                        } else {
                            // HEAVY vocal suppression for Lyria.
                            let base_prompt = format!("{}, ENVIRONMENTAL AMBIENCE ONLY. NO VOCALS. NO SINGING. NO SPEECH. NO LYRICS. NO VOICES. NO HUMMING. PURE INSTRUMENTAL TEXTURE. 100% NON-VOCAL.", raw_prompt);
                            let prompt = bake_lyria_prompt(
                                &base_prompt,
                                bpm,
                                key,
                                intensity,
                                guidance,
                                None,
                                None,
                            );

                            let lyria_opts = gemini::LyriaOptions {
                                seed,
                                negative_prompt: Some(
                                    "vocals, singing, lyrics, speech, humming".to_string(),
                                ),
                                image_data: image_path.and_then(encode_image_base64),
                                temperature,
                                candidate_count,
                            };

                            match gemini::generate_music(&prompt, model, Some(lyria_opts)).await {
                                Ok((audio_bytes, mime_type, description)) => {
                                    let ext = if mime_type.contains("wav") {
                                        "wav"
                                    } else {
                                        "mp3"
                                    };

                                    let final_path = if let Some(d) = duration {
                                        // 1. Decode Lyria's output to raw PCM
                                        match audio::decode_to_pcm(&audio_bytes) {
                                            Ok(pcm_data) => {
                                                // 2. Apply seamless looping to meet the target duration
                                                let looped_pcm =
                                                    audio::seamless_loop(pcm_data, d as f32);
                                                // 3. Re-encode to the requested format
                                                match audio::encode_pcm(
                                                    &looped_pcm,
                                                    format,
                                                    Some(audio_options),
                                                ) {
                                                    Ok(p) => p,
                                                    Err(e) => {
                                                        tracing::error!(
                                                            "Failed to encode looped PCM: {}",
                                                            e
                                                        );
                                                        // Fallback to simple transcode if looping fails
                                                        audio::transcode_encoded(
                                                            &audio_bytes,
                                                            format,
                                                            None,
                                                        )
                                                        .unwrap_or_else(|_| "".to_string())
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to decode Lyria output for looping: {}",
                                                    e
                                                );
                                                audio::transcode_encoded(&audio_bytes, format, None)
                                                    .unwrap_or_else(|_| "".to_string())
                                            }
                                        }
                                    } else {
                                        // Standard transcode/save if no duration requested
                                        if format != ext
                                            || arguments
                                                .map(|a| a.as_object().unwrap().len() > 1)
                                                .unwrap_or(false)
                                        {
                                            audio::transcode_encoded(
                                                &audio_bytes,
                                                format,
                                                Some(audio_options),
                                            )
                                            .unwrap_or_else(|_| "".to_string())
                                        } else {
                                            let out_dir = audio::get_output_dir().unwrap();
                                            let p = out_dir.join(format!(
                                                "{}.{}",
                                                uuid::Uuid::new_v4(),
                                                ext
                                            ));
                                            std::fs::write(&p, &audio_bytes).unwrap();
                                            p.to_string_lossy().to_string()
                                        }
                                    };

                                    if auto_play {
                                        let _ = audio::play_audio_file(&final_path);
                                    }
                                    let text_output = format!("✅ Soundscape generated successfully!\n\nFile Path: {}\nQuality: {}\nDescription: {}\n\nModel: {}, Format: {}", final_path, quality, description, model, format);
                                    json!({"content": [{"type": "text", "text": text_output}]})
                                }
                                Err(e) => {
                                    json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini Soundscape error: {}", e)}]})
                                }
                            }
                        }
                    }
                    "transition_soundscape" => {
                        let _permit = semaphore.acquire().await;
                        let from_prompt = arguments
                            .and_then(|a| a.get("from_prompt"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let to_prompt = arguments
                            .and_then(|a| a.get("to_prompt"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let transition_duration = arguments
                            .and_then(|a| a.get("transition_duration"))
                            .and_then(|d| d.as_u64())
                            .map(|d| d as u32)
                            .unwrap_or(config.default_transition_duration);
                        let format = arguments
                            .and_then(|a| a.get("format"))
                            .and_then(|f| f.as_str())
                            .unwrap_or(&config.default_format);
                        let auto_play = arguments
                            .and_then(|a| a.get("auto_play"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let audio_options = audio::AudioOptions {
                            bitrate: config.default_bitrate.clone(),
                            sample_rate: config.default_sample_rate,
                            channels: config.default_channels,
                        };

                        if from_prompt.is_empty() || to_prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Missing prompts."}]})
                        } else {
                            // Apply heavy instrumental prompts to both environments in the transition
                            let enhanced_from = format!("{}, ENVIRONMENTAL AMBIENCE ONLY. NO VOCALS. NO SINGING. PURE INSTRUMENTAL TEXTURE.", from_prompt);
                            let enhanced_to = format!("{}, ENVIRONMENTAL AMBIENCE ONLY. NO VOCALS. NO SINGING. PURE INSTRUMENTAL TEXTURE.", to_prompt);

                            match tokio::try_join!(
                                gemini::generate_music(
                                    &enhanced_from,
                                    "lyria-3-clip-preview",
                                    None
                                ),
                                gemini::generate_music(&enhanced_to, "lyria-3-clip-preview", None)
                            ) {
                                Ok(((encoded1, _mime1, desc1), (encoded2, _mime2, desc2))) => {
                                    // Decode both to PCM for crossfading
                                    match (
                                        audio::decode_to_pcm(&encoded1),
                                        audio::decode_to_pcm(&encoded2),
                                    ) {
                                        (Ok(pcm1), Ok(pcm2)) => {
                                            let transition_samples =
                                                (transition_duration * audio::SAMPLE_RATE) as usize;
                                            let mixed_pcm =
                                                mixer::crossfade(&pcm1, &pcm2, transition_samples);
                                            let actual_duration = (mixed_pcm.len() as f64)
                                                / (audio::SAMPLE_RATE as f64
                                                    * audio::BYTES_PER_SAMPLE as f64);
                                            match audio::encode_pcm(
                                                &mixed_pcm,
                                                format,
                                                Some(audio_options.clone()),
                                            ) {
                                                Ok(p) => {
                                                    if auto_play {
                                                        let _ = audio::play_audio_file(&p);
                                                    }
                                                    let text_output = format!("✅ Transition generated successfully!\n\nFile Path: {}\nDescription: Transition from: {}\nTo: {}\n\nDuration: {:.2}s, Format: {}", p, desc1, desc2, actual_duration, format);
                                                    json!({"content": [{"type": "text", "text": text_output}]})
                                                }
                                                Err(e) => {
                                                    json!({"isError": true, "content": [{"type": "text", "text": format!("Encoding error: {}", e)}]})
                                                }
                                            }
                                        }
                                        _ => {
                                            json!({"isError": true, "content": [{"type": "text", "text": "Failed to decode audio for transition."}]})
                                        }
                                    }
                                }
                                Err(e) => {
                                    json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini error: {}", e)}]})
                                }
                            }
                        }
                    }
                    "play_audio" => {
                        let path = arguments
                            .and_then(|a| a.get("path"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        match audio::play_audio_file(path) {
                            Ok(_) => {
                                json!({"content": [{"type": "text", "text": "Playback started."}]})
                            }
                            Err(e) => {
                                json!({"isError": true, "content": [{"type": "text", "text": format!("Playback error: {}", e)}]})
                            }
                        }
                    }
                    "cleanup_assets" => {
                        let max_age_hours = arguments
                            .and_then(|a| a.get("max_age_hours"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(config.auto_cleanup_hours);
                        match audio::cleanup_assets(Duration::from_secs(max_age_hours * 3600)) {
                            Ok(count) => {
                                json!({"content": [{"type": "text", "text": format!("Deleted {} old files.", count)}]})
                            }
                            Err(e) => {
                                json!({"isError": true, "content": [{"type": "text", "text": format!("Cleanup error: {}", e)}]})
                            }
                        }
                    }
                    "check_dependencies" => match audio::ensure_ffmpeg() {
                        Ok(_) => json!({"content": [{"type": "text", "text": "Dependencies OK."}]}),
                        Err(e) => {
                            json!({"isError": true, "content": [{"type": "text", "text": e.to_string()}]})
                        }
                    },
                    "generate_custom" => {
                        let _permit = semaphore.acquire().await;
                        let prompt = arguments
                            .and_then(|a| a.get("prompt"))
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let engine = arguments
                            .and_then(|a| a.get("engine"))
                            .and_then(|e| e.as_str())
                            .unwrap_or("gemini-live");
                        let model_arg = arguments
                            .and_then(|a| a.get("model"))
                            .and_then(|m| m.as_str());
                        let bpm = arguments
                            .and_then(|a| a.get("bpm"))
                            .and_then(|v| v.as_f64());
                        let key = arguments
                            .and_then(|a| a.get("song_key"))
                            .and_then(|v| v.as_str());
                        let intensity = arguments
                            .and_then(|a| a.get("intensity"))
                            .and_then(|v| v.as_f64());
                        let guidance = arguments
                            .and_then(|a| a.get("guidance"))
                            .and_then(|v| v.as_f64());
                        let temperature = arguments
                            .and_then(|a| a.get("temperature"))
                            .and_then(|v| v.as_f64());
                        let seed = arguments
                            .and_then(|a| a.get("seed"))
                            .and_then(|v| v.as_i64());
                        let negative_prompt = arguments
                            .and_then(|a| a.get("negative_prompt"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let image_path = arguments
                            .and_then(|a| a.get("image_path"))
                            .and_then(|v| v.as_str());
                        let candidate_count = arguments
                            .and_then(|a| a.get("candidate_count"))
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32);

                        let duration = arguments
                            .and_then(|a| a.get("duration"))
                            .and_then(|d| d.as_u64())
                            .map(|d| d as u32);
                        let format = arguments
                            .and_then(|a| a.get("format"))
                            .and_then(|f| f.as_str())
                            .unwrap_or(&config.default_format);
                        let auto_play = arguments
                            .and_then(|a| a.get("auto_play"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        let audio_options = audio::AudioOptions {
                            bitrate: arguments
                                .and_then(|a| a.get("bitrate"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .or(config.default_bitrate.clone()),
                            sample_rate: arguments
                                .and_then(|a| a.get("sample_rate"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_sample_rate),
                            channels: arguments
                                .and_then(|a| a.get("channels"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .or(config.default_channels),
                        };

                        if prompt.is_empty() {
                            json!({"isError": true, "content": [{"type": "text", "text": "Prompt is empty."}]})
                        } else {
                            match engine {
                                "lyria" => {
                                    let model = model_arg.unwrap_or("lyria-3-clip-preview");
                                    let final_prompt = bake_lyria_prompt(
                                        prompt, bpm, key, intensity, guidance, None, None,
                                    );
                                    let lyria_opts = gemini::LyriaOptions {
                                        seed,
                                        negative_prompt,
                                        image_data: image_path.and_then(encode_image_base64),
                                        temperature,
                                        candidate_count,
                                    };
                                    match gemini::generate_music(
                                        &final_prompt,
                                        model,
                                        Some(lyria_opts),
                                    )
                                    .await
                                    {
                                        Ok((audio_bytes, mime_type, description)) => {
                                            let ext = if mime_type.contains("wav") {
                                                "wav"
                                            } else {
                                                "mp3"
                                            };
                                            let final_path = if let Some(d) = duration {
                                                match audio::decode_to_pcm(&audio_bytes) {
                                                    Ok(pcm_data) => {
                                                        let looped_pcm = audio::seamless_loop(
                                                            pcm_data, d as f32,
                                                        );
                                                        match audio::encode_pcm(
                                                            &looped_pcm,
                                                            format,
                                                            Some(audio_options),
                                                        ) {
                                                            Ok(p) => p,
                                                            Err(_) => {
                                                                // Fallback to simple transcode if looping fails
                                                                audio::transcode_encoded(
                                                                    &audio_bytes,
                                                                    format,
                                                                    None,
                                                                )
                                                                .unwrap_or_else(|_| {
                                                                    let out_dir =
                                                                        audio::get_output_dir()
                                                                            .unwrap();
                                                                    let p = out_dir.join(format!(
                                                                        "{}.{}",
                                                                        uuid::Uuid::new_v4(),
                                                                        ext
                                                                    ));
                                                                    std::fs::write(
                                                                        &p,
                                                                        &audio_bytes,
                                                                    )
                                                                    .unwrap();
                                                                    p.to_string_lossy().to_string()
                                                                })
                                                            }
                                                        }
                                                    }
                                                    Err(_) => audio::transcode_encoded(
                                                        &audio_bytes,
                                                        format,
                                                        None,
                                                    )
                                                    .unwrap_or_else(|_| {
                                                        let out_dir =
                                                            audio::get_output_dir().unwrap();
                                                        let p = out_dir.join(format!(
                                                            "{}.{}",
                                                            uuid::Uuid::new_v4(),
                                                            ext
                                                        ));
                                                        std::fs::write(&p, &audio_bytes).unwrap();
                                                        p.to_string_lossy().to_string()
                                                    }),
                                                }
                                            } else {
                                                audio::transcode_encoded(
                                                    &audio_bytes,
                                                    format,
                                                    Some(audio_options),
                                                )
                                                .unwrap_or_else(|_| {
                                                    let out_dir = audio::get_output_dir().unwrap();
                                                    let p = out_dir.join(format!(
                                                        "{}.{}",
                                                        uuid::Uuid::new_v4(),
                                                        ext
                                                    ));
                                                    std::fs::write(&p, &audio_bytes).unwrap();
                                                    p.to_string_lossy().to_string()
                                                })
                                            };
                                            if auto_play {
                                                let _ = audio::play_audio_file(&final_path);
                                            }
                                            json!({"content": [{"type": "text", "text": format!("✅ Custom Lyria Audio Generated!\n\nPath: {}\nModel: {}\nDescription: {}", final_path, model, description)}]})
                                        }
                                        Err(e) => {
                                            json!({"isError": true, "content": [{"type": "text", "text": format!("Lyria error: {}", e)}]})
                                        }
                                    }
                                }
                                "gemini-live" => {
                                    match gemini::generate_audio(prompt, model_arg, duration).await
                                    {
                                        Ok((mut pcm_data, description)) => {
                                            if let Some(d) = duration {
                                                pcm_data = audio::seamless_loop(pcm_data, d as f32);
                                            }
                                            match audio::encode_pcm(
                                                &pcm_data,
                                                format,
                                                Some(audio_options),
                                            ) {
                                                Ok(p) => {
                                                    if auto_play {
                                                        let _ = audio::play_audio_file(&p);
                                                    }
                                                    json!({"content": [{"type": "text", "text": format!("✅ Custom Gemini-Live Audio Generated!\n\nPath: {}\nDescription: {}", p, description)}]})
                                                }
                                                Err(e) => {
                                                    json!({"isError": true, "content": [{"type": "text", "text": format!("Encoding error: {}", e)}]})
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            json!({"isError": true, "content": [{"type": "text", "text": format!("Gemini error: {}", e)}]})
                                        }
                                    }
                                }
                                _ => {
                                    json!({"isError": true, "content": [{"type": "text", "text": format!("Unknown engine: {}", engine)}]})
                                }
                            }
                        }
                    }
                    "list_models" => {
                        let models = json!({
                            "engines": [
                                {
                                    "name": "lyria",
                                    "description": "High-fidelity music and rhythmic loops (REST API).",
                                    "models": [
                                        {
                                            "id": "lyria-3-pro-preview",
                                            "description": "Full-length musical compositions with high structural coherence.",
                                            "cost": "$0.08 per request"
                                        },
                                        {
                                            "id": "lyria-3-clip-preview",
                                            "description": "Short (30s) rhythmic loops, stings, and sound effects.",
                                            "cost": "$0.04 per request"
                                        }
                                    ]
                                },
                                {
                                    "name": "gemini-live",
                                    "description": "Multimodal Live API for real-time audio and environmental sounds (WebSockets).",
                                    "models": [
                                        {
                                            "id": "models/gemini-2.5-flash-native-audio-latest",
                                            "description": "Latest multimodal native audio model. Optimized for low-latency voice and environmental sounds.",
                                            "cost": "Free (standard tier)"
                                        },
                                        {
                                            "id": "models/gemini-2.0-flash-exp",
                                            "description": "Experimental version of the 2.0 multimodal model.",
                                            "cost": "Free"
                                        }
                                    ]
                                }
                            ]
                        });
                        json!({"content": [{"type": "text", "text": format!("Available Audio Models:\n{}", serde_json::to_string_pretty(&models).unwrap())}]})
                    }
                    _ => {
                        json!({"isError": true, "content": [{"type": "text", "text": format!("Unknown tool: {}", name)}]})
                    }
                }
            }
            _ => {
                if id.is_some() {
                    json!({"error": {"code": -32601, "message": "Method not found"}})
                } else {
                    continue;
                }
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
