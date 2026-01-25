//! TTS service using macOS say command

use crate::models::{PodcastError, MacOSVoice};
use std::process::Command;
use std::path::PathBuf;
use std::io::Read;

/// TTS Engine using macOS say command
pub struct TTSEngine {
    available_voices: Vec<MacOSVoice>,
}

impl TTSEngine {
    pub fn new() -> Self {
        Self {
            available_voices: MacOSVoice::all_voices(),
        }
    }

    /// Get available voices
    pub fn get_voices(&self) -> &[MacOSVoice] {
        &self.available_voices
    }

    /// Synthesize text to audio file using macOS say command
    pub fn synthesize(&self, text: &str, voice_id: &str, output_path: &PathBuf) -> Result<(), PodcastError> {
        ::log::info!("Synthesizing with voice '{}': {} chars", voice_id, text.chars().count());

        // Create temp AIFF file path
        let temp_aiff = output_path.with_extension("aiff");

        // Use say command to generate AIFF
        let output = Command::new("say")
            .arg("-v")
            .arg(voice_id)
            .arg("-o")
            .arg(&temp_aiff)
            .arg(text)
            .output()
            .map_err(|e| PodcastError::TTSError(format!("Failed to run say command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PodcastError::TTSError(format!("say command failed: {}", stderr)));
        }

        // Convert AIFF to WAV using afconvert (macOS built-in)
        let convert_output = Command::new("afconvert")
            .arg("-f")
            .arg("WAVE")
            .arg("-d")
            .arg("LEI16@22050")
            .arg(&temp_aiff)
            .arg(output_path)
            .output()
            .map_err(|e| PodcastError::TTSError(format!("Failed to run afconvert: {}", e)))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_aiff);

        if !convert_output.status.success() {
            let stderr = String::from_utf8_lossy(&convert_output.stderr);
            return Err(PodcastError::TTSError(format!("afconvert failed: {}", stderr)));
        }

        ::log::info!("TTS synthesis complete: {:?}", output_path);
        Ok(())
    }

    /// Synthesize text and return raw audio bytes
    pub fn synthesize_to_bytes(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, PodcastError> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("tts_{}.wav", uuid::Uuid::new_v4()));

        self.synthesize(text, voice_id, &temp_file)?;

        let mut file = std::fs::File::open(&temp_file)
            .map_err(|e| PodcastError::FileError(format!("Failed to read audio: {}", e)))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| PodcastError::FileError(format!("Failed to read audio: {}", e)))?;

        let _ = std::fs::remove_file(&temp_file);

        Ok(buffer)
    }

    /// List available voices using say -v ?
    pub fn list_system_voices() -> Vec<String> {
        let output = Command::new("say")
            .arg("-v")
            .arg("?")
            .output();

        match output {
            Ok(out) => {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter_map(|line| {
                        line.split_whitespace().next().map(|s| s.to_string())
                    })
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    }
}

impl Default for TTSEngine {
    fn default() -> Self {
        Self::new()
    }
}
