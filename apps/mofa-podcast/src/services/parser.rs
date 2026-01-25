//! Script parser service
//! Supports Markdown, JSON, and plain text formats

use crate::models::{PodcastScript, ScriptFormat, CharacterRole, DialogueSegment};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Parse a script file and return PodcastScript model
pub fn parse_script(file_path: &str, content: &str) -> Result<PodcastScript> {
    let format = if file_path.ends_with(".md") {
        ScriptFormat::Markdown
    } else if file_path.ends_with(".json") {
        ScriptFormat::Json
    } else {
        ScriptFormat::PlainText
    };

    let title = extract_title(file_path, content);
    let mut script = PodcastScript::new(title, content.to_string(), format.clone());
    script.roles = detect_roles(content, &format);
    script.file_path = Some(file_path.to_string());

    Ok(script)
}

/// Parse script content directly (without file path)
pub fn parse_content(content: &str) -> Result<PodcastScript> {
    // Try to detect format from content
    let format = if content.trim().starts_with('{') {
        ScriptFormat::Json
    } else if content.contains("**") || content.starts_with('#') {
        ScriptFormat::Markdown
    } else {
        ScriptFormat::PlainText
    };

    let title = "Untitled Script".to_string();
    let mut script = PodcastScript::new(title, content.to_string(), format.clone());
    script.roles = detect_roles(content, &format);

    Ok(script)
}

fn extract_title(file_path: &str, content: &str) -> String {
    if content.starts_with("# ") {
        if let Some(line) = content.lines().next() {
            return line.trim_start_matches("# ").to_string();
        }
    }

    std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled Script")
        .to_string()
}

/// Detect character roles in script content
fn detect_roles(content: &str, format: &ScriptFormat) -> Vec<CharacterRole> {
    match format {
        ScriptFormat::Markdown => detect_markdown_roles(content),
        ScriptFormat::Json => detect_json_roles(content),
        ScriptFormat::PlainText => detect_text_roles(content),
    }
}

/// Detect roles in Markdown format
fn detect_markdown_roles(content: &str) -> Vec<CharacterRole> {
    let mut role_counts: HashMap<String, usize> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip markdown headers
        if trimmed.starts_with('#') {
            continue;
        }

        // Match pattern like "role:" or "role:"
        if let Some(pos) = trimmed.find(|c| c == ':' || c == ':') {
            if pos > 0 && pos < 50 {
                let role_name = trimmed[..pos].trim();
                let role_name = role_name.replace("**", "").trim().to_string();

                if role_name.is_empty() || role_name.starts_with('#') || role_name.len() > 50 {
                    continue;
                }

                // Check if role name contains valid characters
                let has_chinese = role_name.chars().any(|c| {
                    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF)
                });

                if role_name.chars().any(|c| c.is_alphabetic()) || has_chinese {
                    *role_counts.entry(role_name).or_insert(0) += 1;
                }
            }
        }
    }

    role_counts
        .into_iter()
        .enumerate()
        .map(|(idx, (name, count))| CharacterRole {
            id: format!("role_{}", idx),
            name,
            segment_count: count,
        })
        .collect()
}

/// Detect roles in JSON format
fn detect_json_roles(content: &str) -> Vec<CharacterRole> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        let mut role_counts: HashMap<String, usize> = HashMap::new();

        // Try "segments" or "dialogue" arrays
        let dialogue_array = json.get("segments")
            .and_then(|d| d.as_array())
            .or_else(|| json.get("dialogue").and_then(|d| d.as_array()));

        if let Some(dialogue) = dialogue_array {
            for item in dialogue {
                if let Some(speaker) = item.get("speaker").and_then(|s| s.as_str()) {
                    *role_counts.entry(speaker.to_string()).or_insert(0) += 1;
                }
                if let Some(role) = item.get("role").and_then(|r| r.as_str()) {
                    *role_counts.entry(role.to_string()).or_insert(0) += 1;
                }
            }
        }

        return role_counts
            .into_iter()
            .enumerate()
            .map(|(idx, (name, count))| CharacterRole {
                id: format!("role_{}", idx),
                name,
                segment_count: count,
            })
            .collect();
    }

    Vec::new()
}

/// Detect roles in plain text format
fn detect_text_roles(content: &str) -> Vec<CharacterRole> {
    // Same logic as markdown
    detect_markdown_roles(content)
}

/// Parse script into dialogue segments
pub fn parse_segments(script: &PodcastScript) -> Vec<DialogueSegment> {
    match &script.format {
        ScriptFormat::Markdown | ScriptFormat::PlainText => parse_markdown_segments(&script.content),
        ScriptFormat::Json => parse_json_segments(&script.content),
    }
}

fn parse_markdown_segments(content: &str) -> Vec<DialogueSegment> {
    let mut segments = Vec::new();
    let re = Regex::new(r"(?m)^(?:\*\*)?([^\*:\n]+?)(?:\*\*)?[:：]\s*([^\n]+)").unwrap();

    for (index, capture) in re.captures_iter(content).enumerate() {
        if let (Some(role_match), Some(text_match)) = (capture.get(1), capture.get(2)) {
            let role = role_match.as_str().trim().replace("**", "");
            let text = text_match.as_str().trim().to_string();

            // Skip headers and empty lines
            if role.starts_with('#') || role.is_empty() || text.is_empty() {
                continue;
            }

            segments.push(DialogueSegment {
                index,
                role,
                text,
            });
        }
    }

    segments
}

fn parse_json_segments(content: &str) -> Vec<DialogueSegment> {
    let mut segments = Vec::new();

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        let dialogue_array = json.get("segments")
            .and_then(|d| d.as_array())
            .or_else(|| json.get("dialogue").and_then(|d| d.as_array()));

        if let Some(dialogue) = dialogue_array {
            for (index, item) in dialogue.iter().enumerate() {
                let role = item.get("speaker")
                    .or_else(|| item.get("role"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let text = item.get("text")
                    .or_else(|| item.get("content"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();

                if !text.is_empty() {
                    segments.push(DialogueSegment { index, role, text });
                }
            }
        }
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_parsing() {
        let content = r#"
# Test Script

Host: Welcome to our podcast!
Guest: Thank you for having me.
Host: Let's talk about AI.
"#;
        let script = parse_content(content).unwrap();
        assert_eq!(script.roles.len(), 2);

        let segments = parse_segments(&script);
        assert_eq!(segments.len(), 3);
    }

    #[test]
    fn test_chinese_parsing() {
        let content = r#"
主持人：欢迎收听我们的播客！
嘉宾：谢谢邀请。
主持人：今天我们来聊聊人工智能。
"#;
        let script = parse_content(content).unwrap();
        assert_eq!(script.roles.len(), 2);

        let segments = parse_segments(&script);
        assert_eq!(segments.len(), 3);
    }
}
