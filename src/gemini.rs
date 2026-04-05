use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Advanced options for Lyria 3 generation based on official Vertex AI specs.
#[derive(Debug, Clone, Default)]
pub struct LyriaOptions {
    /// Seed for deterministic reproducibility.
    pub seed: Option<i64>,
    /// Elements to exclude from the generated audio.
    pub negative_prompt: Option<String>,
    /// Base64 encoded image to guide the generation.
    pub image_data: Option<String>,
    /// Randomness/creativity level (0.0-2.0).
    pub temperature: Option<f64>,
    /// Number of audio variations to generate.
    pub candidate_count: Option<i32>,
}

/// Connects to the Gemini Live API via WebSockets to generate audio from a prompt.
/// 
/// Returns a tuple containing the raw PCM data and a text description of the audio.
pub async fn generate_audio(
    prompt: &str,
    model: Option<&str>,
    _requested_duration: Option<u32>,
) -> Result<(Vec<u8>, String)> {
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow!("GEMINI_API_KEY environment variable not set"))?;

    let url = format!(
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1alpha.GenerativeService.BidiGenerateContent?key={}",
        api_key
    );

    let (ws_stream, _) = connect_async(url).await?;
    tracing::info!("Connected to Gemini Live API");
    let (mut write, mut read) = ws_stream.split();

    let model_name = model.unwrap_or("models/gemini-2.5-flash-native-audio-latest");

    // 1. Send Setup message
    let setup = json!({
        "setup": {
            "model": model_name,
            "system_instruction": {
                "parts": [{ "text": "You are a professional audio generation engine. Your ONLY purpose is to generate high-fidelity audio based on user prompts. Do NOT provide any conversational responses, text explanations, or narration unless specifically requested. Focus entirely on the sonic quality and texture of the audio output. Never start your response with 'Sure' or 'I can help with that'." }]
            },
            "generationConfig": {
                "response_modalities": ["AUDIO"]
            }
        }
    });

    write.send(Message::Text(setup.to_string().into())).await?;

    // Wait for setup completion
    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let resp: Value = serde_json::from_str(&text)?;
            if resp.get("setupComplete").is_some() {
                tracing::info!("Setup complete");
                break;
            }
        }
    }

    // 2. Send the actual content request
    let content = json!({
        "client_content": {
            "turns": [{
                "parts": [{ "text": prompt }],
                "role": "user"
            }],
            "turn_complete": true
        }
    });

    write
        .send(Message::Text(content.to_string().into()))
        .await?;

    let mut pcm_data = Vec::new();
    let mut description = String::new();

    // 3. Receive responses
    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let resp: Value = serde_json::from_str(&text)?;

            // Extract audio metadata if present
            if let Some(candidates) = resp.get("candidates").and_then(|c| c.as_array()) {
                for candidate in candidates {
                    if let Some(parts) = candidate
                        .get("content")
                        .and_then(|c| c.get("parts"))
                        .and_then(|p| p.as_array())
                    {
                        for part in parts {
                            if let Some(inline_data) = part.get("inline_data") {
                                if let Some(data_str) =
                                    inline_data.get("data").and_then(|d| d.as_str())
                                {
                                    let mut bytes = STANDARD.decode(data_str)?;
                                    pcm_data.append(&mut bytes);
                                }
                            }
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                description.push_str(text);
                            }
                        }
                    }
                }
            }

            // Check if this turn is finished
            if resp
                .get("serverContent")
                .and_then(|s| s.get("turnComplete"))
                .and_then(|t| t.as_bool())
                .unwrap_or(false)
            {
                tracing::info!("Turn complete, received {} bytes of audio", pcm_data.len());
                break;
            }
        }
    }

    if pcm_data.is_empty() {
        return Err(anyhow!("No audio data received from Gemini API"));
    }

    Ok((pcm_data, description))
}

/// Generates high-fidelity music using Gemini's Lyria 3 models via REST API.
/// Returns a tuple of (encoded_audio_bytes, mime_type, description).
pub async fn generate_music(
    prompt: &str,
    model: &str,
    options: Option<LyriaOptions>,
) -> Result<(Vec<u8>, String, String)> {
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow!("GEMINI_API_KEY environment variable not set"))?;

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    let mut generation_config = json!({
        "response_modalities": ["AUDIO", "TEXT"]
    });

    if let Some(ref opts) = options {
        if let Some(seed) = opts.seed {
            generation_config["seed"] = json!(seed);
        }
        if let Some(temp) = opts.temperature {
            generation_config["temperature"] = json!(temp);
        }
        if let Some(count) = opts.candidate_count {
            generation_config["candidateCount"] = json!(count);
        }
    }

    let mut parts = vec![json!({ "text": prompt })];

    if let Some(opts) = options {
        // We do NOT bake BPM/Guidance here because the caller (src/main.rs) already does it via bake_lyria_prompt
        // if they are calling from the tool loop.

        if let Some(img) = opts.image_data {
            let mime = if img.starts_with("iVBOR") {
                "image/png"
            } else {
                "image/jpeg"
            };
            parts.push(json!({
                "inline_data": {
                    "mime_type": mime,
                    "data": img
                }
            }));
        }

        if let Some(neg) = opts.negative_prompt {
            parts.push(json!({ "text": format!("Negative prompt: {}", neg) }));
        }
    }

    let request_body = json!({
        "contents": [{
            "parts": parts
        }],
        "generationConfig": generation_config
    });

    let response = client
        .post(url)
        .header("x-goog-api-key", api_key)
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().await?;
        return Err(anyhow!("Lyria API error ({}): {}", status, err_body));
    }

    let res_json: Value = response.json().await?;

    let mut audio_bytes = Vec::new();
    let mut mime_type = String::new();
    let mut description = String::new();

    if let Some(candidates) = res_json.get("candidates").and_then(|c| c.as_array()) {
        for candidate in candidates {
            if let Some(content) = candidate.get("content") {
                if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                    for part in parts {
                        let inline_data =
                            part.get("inline_data").or_else(|| part.get("inlineData"));
                        if let Some(data_obj) = inline_data {
                            if let Some(data_str) = data_obj.get("data").and_then(|d| d.as_str()) {
                                audio_bytes = STANDARD.decode(data_str)?;
                            }
                            if let Some(m) = data_obj
                                .get("mime_type")
                                .or_else(|| data_obj.get("mimeType"))
                                .and_then(|m| m.as_str())
                            {
                                mime_type = m.to_string();
                            }
                        }
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            description.push_str(text);
                        }
                    }
                }
            }
        }
    }

    if audio_bytes.is_empty() {
        return Err(anyhow!("No music data received from Gemini API. Note: Lyria models require a paid Tier or specific preview access."));
    }

    Ok((audio_bytes, mime_type, description))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_generate_audio() {
        let result = generate_audio("test", None, None).await;
        assert!(result.is_ok());
    }
}
