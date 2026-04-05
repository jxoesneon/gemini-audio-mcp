use anyhow::{anyhow, Context};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Advanced formatting options for audio conversion.
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct AudioOptions {
    /// Bitrate for the output file (e.g., "192k", "320k").
    pub bitrate: Option<String>,
    /// Sample rate for the output file (e.g., 44100, 48000).
    pub sample_rate: Option<u32>,
    /// Number of audio channels (1 for mono, 2 for stereo).
    pub channels: Option<u32>,
}

/// Checks if ffmpeg is installed and available in the system PATH.
pub fn ensure_ffmpeg() -> anyhow::Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(anyhow!(
            "FFmpeg not found. Please install FFmpeg to use all audio formats.\n\
             - macOS: brew install ffmpeg\n\
             - Windows: winget install ffmpeg\n\
             - Linux: sudo apt install ffmpeg"
        )),
    }
}

/// Returns the platform-specific directory for storing audio outputs.
pub fn get_output_dir() -> anyhow::Result<PathBuf> {
    let base_dir = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| anyhow!("Could not determine a suitable data directory"))?;

    let out_dir = base_dir.join("gemini-audio-mcp").join("audio_outputs");

    if !out_dir.exists() {
        fs::create_dir_all(&out_dir).context("Failed to create audio_outputs directory")?;
    }

    Ok(out_dir)
}

use std::io::Write;

/// Transcodes already-encoded audio bytes (e.g., MP3, WAV) to a target format.
/// 
/// Returns the absolute path to the generated file.
pub fn transcode_encoded(
    encoded_bytes: &[u8],
    target_format: &str,
    options: Option<AudioOptions>,
) -> anyhow::Result<String> {
    ensure_ffmpeg()?;

    let format_lower = target_format.to_lowercase();
    let out_dir = get_output_dir()?;
    let filename = format!("{}.{}", Uuid::new_v4(), format_lower);
    let output_path = out_dir.join(filename);

    let mut command = Command::new("ffmpeg");

    // FFmpeg can auto-detect the input format from the pipe if we don't specify -f
    command.arg("-i").arg("pipe:0");

    if let Some(opts) = options {
        if let Some(br) = opts.bitrate {
            command.arg("-b:a").arg(br);
        }
        if let Some(sr) = opts.sample_rate {
            command.arg("-ar").arg(sr.to_string());
        }
        if let Some(ch) = opts.channels {
            command.arg("-ac").arg(ch.to_string());
        }
    }

    let mut child = command
        .arg("-y")
        .arg(&output_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg for transcoding")?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to open stdin"))?;
        stdin
            .write_all(encoded_bytes)
            .context("Failed to write to ffmpeg stdin")?;
    }

    let output = child
        .wait_with_output()
        .context("Failed to wait for ffmpeg")?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg transcoding failed: {}", err_msg);
    }

    Ok(output_path.to_string_lossy().to_string())
}

/// Saves raw audio bytes to the output directory with a unique filename.
/// Returns the absolute path to the saved file.
pub fn save_audio(bytes: &[u8], extension: &str) -> anyhow::Result<String> {
    let out_dir = get_output_dir()?;
    let filename = format!("{}.{}", Uuid::new_v4(), extension);
    let output_path = out_dir.join(filename);
    fs::write(&output_path, bytes).context("Failed to write audio file")?;
    Ok(output_path.to_string_lossy().to_string())
}

/// Encodes raw PCM data directly to the target format using FFmpeg pipes.
/// Returns the absolute path to the generated file.
pub fn encode_pcm(
    pcm_data: &[u8],
    format: &str,
    options: Option<AudioOptions>,
) -> anyhow::Result<String> {
    ensure_ffmpeg()?;

    let format_lower = format.to_lowercase();
    let supported_formats = [
        "mp3", "ogg", "flac", "opus", "aac", "m4a", "aiff", "wma", "ac3", "wav",
    ];

    if !supported_formats.contains(&format_lower.as_str()) {
        anyhow::bail!(
            "Unsupported format: {}. Supported formats are: {:?}",
            format,
            supported_formats
        );
    }

    let out_dir = get_output_dir()?;
    let filename = format!("{}.{}", Uuid::new_v4(), format_lower);
    let output_path = out_dir.join(filename);

    let mut command = Command::new("ffmpeg");

    // Input parameters: raw s16le PCM, 24kHz, Mono
    command
        .arg("-f")
        .arg("s16le")
        .arg("-ar")
        .arg(SAMPLE_RATE.to_string())
        .arg("-ac")
        .arg("1")
        .arg("-i")
        .arg("pipe:0"); // Read from stdin

    // Output parameters
    if let Some(opts) = options {
        if let Some(br) = opts.bitrate {
            command.arg("-b:a").arg(br);
        }
        if let Some(sr) = opts.sample_rate {
            command.arg("-ar").arg(sr.to_string());
        }
        if let Some(ch) = opts.channels {
            command.arg("-ac").arg(ch.to_string());
        }
    }

    let mut child = command
        .arg("-y") // Overwrite if exists
        .arg(&output_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg")?;

    // Write PCM data to ffmpeg's stdin
    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to open stdin"))?;
        stdin
            .write_all(pcm_data)
            .context("Failed to write to ffmpeg stdin")?;
        // Stdin is closed here, signaling EOF to ffmpeg
    }

    let output = child
        .wait_with_output()
        .context("Failed to wait for ffmpeg")?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {}", err_msg);
    }

    Ok(output_path.to_string_lossy().to_string())
}

/// Plays an audio file using the system's default player.
pub fn play_audio_file(path: &str) -> anyhow::Result<()> {
    if !Path::new(path).exists() {
        anyhow::bail!("Audio file not found: {}", path);
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("afplay").arg(path).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try common linux players
        if Command::new("paplay").arg("-version").status().is_ok() {
            Command::new("paplay").arg(path).spawn()?;
        } else if Command::new("aplay").arg("--version").status().is_ok() {
            Command::new("aplay").arg(path).spawn()?;
        } else {
            Command::new("xdg-open").arg(path).spawn()?;
        }
    }

    Ok(())
}

/// Deletes audio files older than the specified age.
pub fn cleanup_assets(max_age: Duration) -> anyhow::Result<usize> {
    let out_dir = get_output_dir()?;
    let mut count = 0;

    for entry in fs::read_dir(out_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let metadata = entry.metadata()?;
            let modified = metadata.modified()?;
            if SystemTime::now().duration_since(modified)? > max_age {
                fs::remove_file(path)?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Decodes already-encoded audio bytes (e.g., MP3, WAV) back to raw PCM (24kHz 16-bit mono).
pub fn decode_to_pcm(encoded_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    ensure_ffmpeg()?;

    let mut command = Command::new("ffmpeg");
    command
        .arg("-i")
        .arg("pipe:0")
        .arg("-f")
        .arg("s16le")
        .arg("-ar")
        .arg(SAMPLE_RATE.to_string())
        .arg("-ac")
        .arg("1")
        .arg("pipe:1");

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg for decoding")?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to open stdin"))?;
        stdin
            .write_all(encoded_bytes)
            .context("Failed to write to ffmpeg stdin")?;
    }

    let output = child
        .wait_with_output()
        .context("Failed to wait for ffmpeg")?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg decoding failed: {}", err_msg);
    }

    Ok(output.stdout)
}

/// PCM constants for 24kHz 16-bit mono audio.
pub const SAMPLE_RATE: u32 = 24000;
pub const BYTES_PER_SAMPLE: usize = 2;

/// Trims PCM data to a specific duration in seconds.
pub fn trim_audio(pcm_data: Vec<u8>, duration_secs: f32) -> Vec<u8> {
    let target_bytes = (duration_secs * SAMPLE_RATE as f32 * BYTES_PER_SAMPLE as f32) as usize;
    // Align to sample boundary (2 bytes for 16-bit mono)
    let target_bytes = (target_bytes / BYTES_PER_SAMPLE) * BYTES_PER_SAMPLE;

    if pcm_data.len() <= target_bytes {
        pcm_data
    } else {
        pcm_data[..target_bytes].to_vec()
    }
}

/// Loops PCM data until it meets the target duration, applying a 100ms micro-crossfade
/// at loop points to eliminate audible clicks.
pub fn seamless_loop(pcm_data: Vec<u8>, target_duration_secs: f32) -> Vec<u8> {
    if pcm_data.is_empty() || target_duration_secs <= 0.0 {
        return Vec::new();
    }

    let total_bytes_needed =
        (target_duration_secs * SAMPLE_RATE as f32 * BYTES_PER_SAMPLE as f32) as usize;

    // If already long enough, just trim
    if pcm_data.len() >= total_bytes_needed {
        return trim_audio(pcm_data, target_duration_secs);
    }

    // Convert input to i16 samples once to avoid repeated conversions
    let input_samples: Vec<i16> = pcm_data
        .chunks_exact(BYTES_PER_SAMPLE)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    if input_samples.is_empty() {
        return Vec::new();
    }

    // 100ms transition
    let transition_samples = (SAMPLE_RATE as f32 * 0.1) as usize;
    // Safety: transition cannot be longer than half the clip
    let actual_transition = transition_samples.min(input_samples.len() / 2);

    let mut result_samples = input_samples.clone();

    while (result_samples.len() * BYTES_PER_SAMPLE) < total_bytes_needed {
        let pcm1_len = result_samples.len();
        let pcm2_len = input_samples.len();

        let mut next_iteration = Vec::with_capacity(pcm1_len + pcm2_len - actual_transition);

        // 1. Keep pcm1 before transition
        next_iteration.extend_from_slice(&result_samples[..pcm1_len - actual_transition]);

        // 2. Crossfade end of pcm1 with start of pcm2
        if actual_transition > 0 {
            for i in 0..actual_transition {
                let alpha = i as f32 / actual_transition as f32;
                let s1 = result_samples[pcm1_len - actual_transition + i];
                let s2 = input_samples[i];
                let mixed = ((1.0 - alpha) * s1 as f32 + alpha * s2 as f32) as i16;
                next_iteration.push(mixed);
            }
        }

        // 3. Add pcm2 after transition
        if pcm2_len > actual_transition {
            next_iteration.extend_from_slice(&input_samples[actual_transition..]);
        }

        result_samples = next_iteration;

        // Safety: break if we're not making progress
        if result_samples.len() <= pcm1_len && pcm2_len > actual_transition {
            break;
        }
    }

    // Convert back to little-endian bytes
    let mut result_bytes = Vec::with_capacity(result_samples.len() * BYTES_PER_SAMPLE);
    for sample in result_samples {
        result_bytes.extend_from_slice(&sample.to_le_bytes());
    }

    trim_audio(result_bytes, target_duration_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_audio() {
        // 1 second of audio = 48000 bytes
        let pcm_data = vec![0u8; 48000];
        let trimmed = trim_audio(pcm_data, 0.5);
        assert_eq!(trimmed.len(), 24000); // 0.5s = 24000 bytes
    }

    #[test]
    fn test_seamless_loop_duration() {
        // 100ms of audio (2400 samples = 4800 bytes)
        let mut pcm_data = Vec::with_capacity(4800);
        for i in 0..2400 {
            pcm_data.extend_from_slice(&(i as i16).to_le_bytes());
        }

        // Loop to 500ms (12000 samples = 24000 bytes)
        let looped = seamless_loop(pcm_data, 0.5);
        assert_eq!(looped.len(), 24000);
    }

    #[test]
    fn test_seamless_loop_short_clip() {
        // Very short clip (10 samples)
        let mut pcm_data = Vec::with_capacity(20);
        for i in 0..10 {
            pcm_data.extend_from_slice(&(i as i16).to_le_bytes());
        }

        // Loop to 1 second
        let looped = seamless_loop(pcm_data, 1.0);
        assert_eq!(looped.len(), 48000);
    }

    #[test]
    fn test_encode_pcm_basic() {
        if ensure_ffmpeg().is_err() {
            return;
        }
        // 1 second of silence
        let pcm_data = vec![0u8; 48000];
        let result = encode_pcm(&pcm_data, "mp3", None);
        assert!(result.is_ok(), "Encoding failed: {:?}", result.err());
        let path = result.unwrap();
        assert!(Path::new(&path).exists());
        assert!(path.ends_with(".mp3"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_encode_pcm_with_options() {
        if ensure_ffmpeg().is_err() {
            return;
        }
        let pcm_data = vec![0u8; 48000];
        let options = AudioOptions {
            bitrate: Some("128k".to_string()),
            sample_rate: Some(44100),
            channels: Some(2),
        };
        let result = encode_pcm(&pcm_data, "wav", Some(options));
        assert!(result.is_ok(), "Encoding failed: {:?}", result.err());
        let path = result.unwrap();
        assert!(Path::new(&path).exists());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_decode_to_pcm() {
        if ensure_ffmpeg().is_err() {
            return;
        }
        // Create a 100ms 440Hz sine wave WAV file in memory
        let mut pcm_data = Vec::with_capacity(4800);
        for i in 0..2400 {
            let val = (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 24000.0).sin();
            let sample = (val * i16::MAX as f32) as i16;
            pcm_data.extend_from_slice(&sample.to_le_bytes());
        }

        let encoded_wav_path = encode_pcm(&pcm_data, "wav", None).unwrap();
        let encoded_bytes = fs::read(&encoded_wav_path).unwrap();
        let _ = fs::remove_file(encoded_wav_path);

        let decoded_pcm = decode_to_pcm(&encoded_bytes).unwrap();
        // Should be roughly the same size (FFmpeg might add some padding/metadata header stuff but the raw PCM should be back)
        assert!(decoded_pcm.len() >= 4800);
        
        // Check first few samples
        let s1 = i16::from_le_bytes([pcm_data[0], pcm_data[1]]);
        let d1 = i16::from_le_bytes([decoded_pcm[0], decoded_pcm[1]]);
        // Allow some small difference due to lossy-ish conversion or dithering (though WAV should be exact)
        assert!((s1 - d1).abs() < 100);
    }

    #[test]
    fn test_seamless_loop_crossfade_logic() {
        // Create 1 second of a constant tone
        let mut pcm_data = Vec::with_capacity(48000);
        for i in 0..24000 {
            pcm_data.extend_from_slice(&(1000i16).to_le_bytes());
        }

        // Loop to 2 seconds
        let looped = seamless_loop(pcm_data, 2.0);
        assert_eq!(looped.len(), 96000);

        // Check if the middle (where transition happened) is still roughly 1000
        // Transition is 100ms (2400 samples = 4800 bytes)
        // Original was 48000 bytes.
        // Loop point should be around 48000 - 4800 = 43200 bytes offset?
        // Wait, the logic is: next_iteration.extend_from_slice(&result_samples[..pcm1_len - actual_transition]);
        // then crossfade.
        
        let mid_sample_idx = 24000 - 1200; // Middle of transition
        let offset = mid_sample_idx * 2;
        let sample = i16::from_le_bytes([looped[offset], looped[offset + 1]]);
        assert_eq!(sample, 1000);
    }
}
