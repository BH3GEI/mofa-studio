//! Audio generation orchestrator

use crate::models::{PodcastScript, AudioSettings, PodcastError};
use crate::services::parser;
use crate::services::tts::TTSEngine;
use std::path::PathBuf;
use std::collections::HashMap;

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send>;

/// Audio generator
pub struct AudioGenerator {
    tts_engine: TTSEngine,
    output_dir: PathBuf,
}

impl AudioGenerator {
    pub fn new(output_dir: PathBuf) -> Result<Self, PodcastError> {
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| PodcastError::FileError(format!("Failed to create output dir: {}", e)))?;

        Ok(Self {
            tts_engine: TTSEngine::new(),
            output_dir,
        })
    }

    /// Generate podcast audio from script
    pub fn generate(
        &self,
        script: &PodcastScript,
        voice_assignments: &HashMap<String, String>,
        settings: &AudioSettings,
        progress: Option<ProgressCallback>,
    ) -> Result<PathBuf, PodcastError> {
        ::log::info!("Starting audio generation for: {}", script.title);

        // Parse segments
        let segments = parser::parse_segments(script);
        if segments.is_empty() {
            return Err(PodcastError::ParseError("No dialogue segments found".into()));
        }

        let total_steps = segments.len() + 2;
        let report = |step: usize, msg: &str| {
            if let Some(ref cb) = progress {
                cb(step, total_steps, msg);
            }
        };

        report(1, "Parsing script...");

        // Generate audio for each segment
        let mut audio_files: Vec<PathBuf> = Vec::new();
        let temp_dir = std::env::temp_dir().join("mofa_podcast");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| PodcastError::FileError(e.to_string()))?;

        for (idx, segment) in segments.iter().enumerate() {
            report(idx + 2, &format!("Generating segment {}/{}...", idx + 1, segments.len()));

            let voice_id = voice_assignments.get(&segment.role)
                .ok_or_else(|| PodcastError::VoiceNotAssigned(segment.role.clone()))?;

            let output_file = temp_dir.join(format!("segment_{:04}.wav", idx));
            self.tts_engine.synthesize(&segment.text, voice_id, &output_file)?;
            audio_files.push(output_file);
        }

        report(total_steps - 1, "Concatenating audio...");

        // Concatenate all segments
        let output_file = self.output_dir.join(format!("{}.wav", script.title.replace(" ", "_")));
        self.concatenate_wav_files(&audio_files, &output_file)?;

        // Clean up temp files
        for file in &audio_files {
            let _ = std::fs::remove_file(file);
        }

        report(total_steps, "Complete!");
        ::log::info!("Audio generated: {:?}", output_file);

        Ok(output_file)
    }

    /// Concatenate WAV files using sox or manual method
    fn concatenate_wav_files(&self, input_files: &[PathBuf], output: &PathBuf) -> Result<(), PodcastError> {
        if input_files.is_empty() {
            return Err(PodcastError::AudioError("No input files".into()));
        }

        if input_files.len() == 1 {
            std::fs::copy(&input_files[0], output)
                .map_err(|e| PodcastError::FileError(e.to_string()))?;
            return Ok(());
        }

        // Try using sox for concatenation
        let sox_result = std::process::Command::new("sox")
            .args(input_files.iter().map(|p| p.as_os_str()))
            .arg(output.as_os_str())
            .output();

        if let Ok(output_result) = sox_result {
            if output_result.status.success() {
                return Ok(());
            }
        }

        // Fallback: manual concatenation using hound
        self.manual_concatenate(input_files, output)
    }

    fn manual_concatenate(&self, input_files: &[PathBuf], output: &PathBuf) -> Result<(), PodcastError> {
        use hound::{WavReader, WavWriter, WavSpec};

        // Read first file to get spec
        let first_reader = WavReader::open(&input_files[0])
            .map_err(|e| PodcastError::AudioError(format!("Failed to read WAV: {}", e)))?;
        let spec = first_reader.spec();

        // Create output writer
        let mut writer = WavWriter::create(output, spec)
            .map_err(|e| PodcastError::AudioError(format!("Failed to create WAV: {}", e)))?;

        // Write all samples
        for file in input_files {
            let mut reader = WavReader::open(file)
                .map_err(|e| PodcastError::AudioError(format!("Failed to read WAV: {}", e)))?;

            for sample in reader.samples::<i16>() {
                let sample = sample.map_err(|e| PodcastError::AudioError(e.to_string()))?;
                writer.write_sample(sample)
                    .map_err(|e| PodcastError::AudioError(e.to_string()))?;
            }
        }

        writer.finalize()
            .map_err(|e| PodcastError::AudioError(e.to_string()))?;

        Ok(())
    }
}
