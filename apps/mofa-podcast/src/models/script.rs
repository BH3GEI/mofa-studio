//! Podcast script data structures

use serde::{Deserialize, Serialize};

/// Script format enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptFormat {
    Markdown,
    Json,
    PlainText,
}

/// Character role detected in script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRole {
    pub id: String,
    pub name: String,
    pub segment_count: usize,
}

/// A single dialogue segment
#[derive(Debug, Clone)]
pub struct DialogueSegment {
    pub index: usize,
    pub role: String,
    pub text: String,
}

/// Represents a podcast script with content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastScript {
    pub id: String,
    pub title: String,
    pub content: String,
    pub format: ScriptFormat,
    pub roles: Vec<CharacterRole>,
    pub file_path: Option<String>,
}

impl PodcastScript {
    pub fn new(title: String, content: String, format: ScriptFormat) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            content,
            format,
            roles: Vec::new(),
            file_path: None,
        }
    }
}
