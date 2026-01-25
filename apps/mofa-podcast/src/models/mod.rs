//! Data models for podcast generation

mod script;
mod voice;
mod errors;

pub use script::{PodcastScript, ScriptFormat, CharacterRole, DialogueSegment};
pub use voice::{VoiceAssignment, AudioSettings, AudioFormat, MacOSVoice};
pub use errors::PodcastError;
