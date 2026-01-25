//! Error types for podcast generation

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PodcastError {
    #[error("Failed to parse script: {0}")]
    ParseError(String),

    #[error("TTS synthesis failed for text: {0}")]
    TTSError(String),

    #[error("Audio processing error: {0}")]
    AudioError(String),

    #[error("File error: {0}")]
    FileError(String),

    #[error("No roles detected in script")]
    NoRolesDetected,

    #[error("Voice not assigned for role: {0}")]
    VoiceNotAssigned(String),
}
