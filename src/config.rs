use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_format: String,
    pub default_duration: Option<u32>,
    pub default_bitrate: Option<String>,
    pub default_sample_rate: Option<u32>,
    pub default_channels: Option<u32>,
    pub default_transition_duration: u32,
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
    pub fn get_path() -> Result<PathBuf> {
        let base_dir = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| anyhow::anyhow!("Could not determine a suitable data directory"))?;
        let config_dir = base_dir.join("gemini-audio-mcp");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        Ok(config_dir.join("config.json"))
    }

    pub fn load() -> Self {
        match Self::get_path() {
            Ok(path) => {
                if path.exists() {
                    let content = fs::read_to_string(path).unwrap_or_default();
                    serde_json::from_str(&content).unwrap_or_else(|_| Config::default())
                } else {
                    Config::default()
                }
            }
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content).context("Failed to write config file")?;
        Ok(())
    }
}
