//! Voice and audio configuration

use serde::{Deserialize, Serialize};

/// Voice assignment for character roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceAssignment {
    pub role_id: String,
    pub role_name: String,
    pub voice_id: String,
}

/// Supported audio formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AudioFormat {
    Wav,
    Aiff,
}

/// Audio generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub format: AudioFormat,
    pub sample_rate: u32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            format: AudioFormat::Wav,
            sample_rate: 22050,
        }
    }
}

/// macOS system voices
#[derive(Debug, Clone)]
pub struct MacOSVoice {
    pub id: &'static str,
    pub name: &'static str,
    pub language: &'static str,
    pub gender: &'static str,
}

impl MacOSVoice {
    /// Get all available Chinese voices on macOS
    pub fn chinese_voices() -> Vec<MacOSVoice> {
        vec![
            MacOSVoice { id: "Ting-Ting", name: "Ting-Ting", language: "zh-CN", gender: "female" },
            MacOSVoice { id: "Mei-Jia", name: "Mei-Jia", language: "zh-TW", gender: "female" },
            MacOSVoice { id: "Sin-ji", name: "Sin-ji", language: "zh-HK", gender: "female" },
            MacOSVoice { id: "Yu-shu", name: "Yu-shu", language: "zh-CN", gender: "female" },
            MacOSVoice { id: "Lili", name: "Lili", language: "zh-CN", gender: "female" },
        ]
    }

    /// Get all available English voices on macOS
    pub fn english_voices() -> Vec<MacOSVoice> {
        vec![
            MacOSVoice { id: "Samantha", name: "Samantha", language: "en-US", gender: "female" },
            MacOSVoice { id: "Alex", name: "Alex", language: "en-US", gender: "male" },
            MacOSVoice { id: "Daniel", name: "Daniel", language: "en-GB", gender: "male" },
            MacOSVoice { id: "Karen", name: "Karen", language: "en-AU", gender: "female" },
            MacOSVoice { id: "Moira", name: "Moira", language: "en-IE", gender: "female" },
        ]
    }

    /// Get all available voices
    pub fn all_voices() -> Vec<MacOSVoice> {
        let mut voices = Self::chinese_voices();
        voices.extend(Self::english_voices());
        voices
    }
}
