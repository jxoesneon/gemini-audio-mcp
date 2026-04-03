use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Generates audio from a prompt using the Gemini Live API.
/// Connects via WebSockets, sends a setup message for the native audio model,
/// then sends the user's prompt and accumulates the resulting raw PCM audio bytes.
/// Returns a tuple of (audio_bytes, description).
pub async fn generate_audio(
    prompt: &str,
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

    // 1. Send Setup message
    let setup = json!({
        "setup": {
            "model": "models/gemini-2.5-flash-native-audio-latest",
            "generationConfig": {
                "responseModalities": ["audio"]
            }
        }
    });

    write.send(Message::Text(setup.to_string())).await?;

    // Wait for setupComplete response from server
    while let Some(msg) = read.next().await {
        let msg = msg?;
        let resp: Value = match msg {
            Message::Text(text) => serde_json::from_str(&text)?,
            Message::Binary(bin) => {
                if let Ok(text) = String::from_utf8(bin.clone()) {
                    serde_json::from_str(&text)?
                } else if let Ok(doc) = bson::Document::from_reader(&mut bin.as_slice()) {
                    serde_json::from_value(serde_json::to_value(doc)?)?
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        if resp.get("setupComplete").is_some() {
            tracing::info!("Setup complete");
            break;
        }
    }

    // 2. Send clientContent message with the user's prompt
    let content = json!({
        "clientContent": {
            "turns": [
                {
                    "role": "user",
                    "parts": [{ "text": prompt }]
                }
            ],
            "turnComplete": true
        }
    });
    write.send(Message::Text(content.to_string())).await?;
    let mut audio_bytes = Vec::new();
    let mut description = String::new();

    // 3. Receive responses and accumulate audio chunks
    while let Some(msg) = read.next().await {
        let msg = msg?;
        let resp: Value = match msg {
            Message::Text(text) => serde_json::from_str(&text)?,
            Message::Binary(bin) => {
                if let Ok(text) = String::from_utf8(bin.clone()) {
                    serde_json::from_str(&text)?
                } else if let Ok(doc) = bson::Document::from_reader(&mut bin.as_slice()) {
                    serde_json::from_value(serde_json::to_value(doc)?)?
                } else {
                    continue;
                }
            }
            Message::Ping(_) => {
                write.send(Message::Pong(vec![])).await?;
                continue;
            }
            _ => continue,
        };

        if let Some(server_content) = resp.get("serverContent") {
            if let Some(model_turn) = server_content.get("modelTurn") {
                if let Some(parts) = model_turn.get("parts") {
                    if let Some(parts_arr) = parts.as_array() {
                        for part in parts_arr {
                            if let Some(inline_data) = part.get("inlineData") {
                                if let Some(data_str) =
                                    inline_data.get("data").and_then(|d| d.as_str())
                                {
                                    let decoded = STANDARD.decode(data_str)?;
                                    audio_bytes.extend(decoded);
                                }
                            }
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                if !description.is_empty() {
                                    description.push('\n');
                                }
                                description.push_str(text);
                            }
                        }
                    }
                }
            }

            if server_content.get("turnComplete") == Some(&json!(true)) {
                tracing::info!(
                    "Turn complete, received {} bytes of audio",
                    audio_bytes.len()
                );
                break;
            }
        }
    }

    if audio_bytes.is_empty() {
        return Err(anyhow!("No audio data received from Gemini API"));
    }

    Ok((audio_bytes, description))
}

/// Generates high-fidelity music using Gemini's Lyria 3 models via REST API.
/// Returns a tuple of (encoded_audio_bytes, mime_type, description).
pub async fn generate_music(prompt: &str, model: &str) -> Result<(Vec<u8>, String, String)> {
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow!("GEMINI_API_KEY environment variable not set"))?;

    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let request_body = json!({
        "contents": [{
            "parts": [{ "text": prompt }]
        }],
        "generationConfig": {
            "response_modalities": ["AUDIO", "TEXT"]
        }
    });

    let response = client
        .post(url)
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?;

    let resp_json: Value = response.json().await?;

    let mut audio_bytes = Vec::new();
    let mut mime_type = String::new();
    let mut description = String::new();

    if let Some(candidates) = resp_json.get("candidates").and_then(|c| c.as_array()) {
        for candidate in candidates {
            if let Some(parts) = candidate
                .get("content")
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.as_array())
            {
                for part in parts {
                    if let Some(inline_data) = part.get("inlineData") {
                        if let Some(data_str) = inline_data.get("data").and_then(|d| d.as_str()) {
                            audio_bytes = STANDARD.decode(data_str)?;
                            mime_type = inline_data
                                .get("mimeType")
                                .and_then(|m| m.as_str())
                                .unwrap_or("audio/mp3")
                                .to_string();
                        }
                    }
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        if !description.is_empty() {
                            description.push('\n');
                        }
                        description.push_str(text);
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
        let _ = tracing_subscriber::fmt::try_init();
        if std::env::var("GEMINI_API_KEY").is_err() {
            println!("Skipping test_generate_audio because GEMINI_API_KEY is not set");
            return;
        }

        let result = generate_audio("test", None).await;
        match result {
            Ok((bytes, desc)) => {
                assert!(!bytes.is_empty(), "Should generate some audio bytes");
                println!(
                    "Generated {} bytes of audio. Description: {}",
                    bytes.len(),
                    desc
                );
            }
            Err(e) => {
                println!("generate_audio failed with error: {}", e);
            }
        }
    }
}
