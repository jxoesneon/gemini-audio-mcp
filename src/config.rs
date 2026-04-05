use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistent server configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Default file extension for generated audio (e.g., "wav", "mp3").
    pub default_format: String,
    /// Default target duration for soundscapes in seconds.
    pub default_duration: Option<u32>,
    /// Default bitrate for audio encoding (e.g., "192k").
    pub default_bitrate: Option<String>,
    /// Default sample rate in Hz.
    pub default_sample_rate: Option<u32>,
    /// Default number of audio channels.
    pub default_channels: Option<u32>,
    /// Default duration for crossfade transitions in seconds.
    pub default_transition_duration: u32,
    /// Number of hours before old assets are automatically cleaned up.
    pub auto_cleanup_hours: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_format: "wav".to_string(),
            default_duration: None,
            default_bitrate: None,
            default_sample_rate: Some(24000),
            default_channels: Some(1),
            default_transition_duration: 5,
            auto_cleanup_hours: 24,
        }
    }
}

impl Config {
    /// Returns the absolute path to the configuration file.
    pub fn get_path() -> Result<PathBuf> {
        let base_dir = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| anyhow::anyhow!("Could not determine a suitable data directory"))?;
        let config_dir = base_dir.join("gemini-audio-mcp");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create configuration directory")?;
        }
        Ok(config_dir.join("config.json"))
    }

    /// Loads the configuration from disk, returning Default if not found or invalid.
    pub fn load() -> Self {
        match Self::get_path() {
            Ok(path) => {
                if path.exists() {
                    let content = fs::read_to_string(&path).unwrap_or_default();
                    serde_json::from_str(&content).unwrap_or_else(|e| {
                        tracing::warn!("Failed to parse config file ({}). Using defaults.", e);
                        Config::default()
                    })
                } else {
                    Config::default()
                }
            }
            Err(e) => {
                tracing::warn!("Failed to get config path: {}. Using defaults.", e);
                Config::default()
            }
        }
    }

    /// Persists the configuration to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).context("Failed to write configuration file")?;
        Ok(())
    }
}
