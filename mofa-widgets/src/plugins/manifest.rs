//! Plugin manifest parsing

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    /// WebView-based plugin (Python + HTML)
    WebView,
    /// Native Makepad plugin (requires compilation)
    Native,
}

impl Default for PluginType {
    fn default() -> Self {
        Self::WebView
    }
}

/// Plugin manifest (manifest.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Version string
    pub version: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Author name
    #[serde(default)]
    pub author: String,

    /// Plugin type
    #[serde(default)]
    pub r#type: PluginType,

    /// Icon file (relative to plugin directory)
    #[serde(default)]
    pub icon: Option<String>,

    /// Python entry point (for WebView plugins)
    #[serde(default)]
    pub python_entry: Option<String>,

    /// Static files directory (for WebView plugins)
    #[serde(default)]
    pub static_dir: Option<String>,

    /// Whether to show in sidebar
    #[serde(default = "default_true")]
    pub show_in_sidebar: bool,

    /// Minimum MoFA Studio version required
    #[serde(default)]
    pub min_version: Option<String>,

    /// Plugin homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// Plugin repository URL
    #[serde(default)]
    pub repository: Option<String>,
}

fn default_true() -> bool {
    true
}

impl PluginManifest {
    /// Load manifest from a JSON file
    pub fn from_file(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))
    }

    /// Get the Python entry point path relative to plugin directory
    pub fn get_python_entry(&self) -> &str {
        self.python_entry.as_deref().unwrap_or("python/app.py")
    }

    /// Get the static directory path relative to plugin directory
    pub fn get_static_dir(&self) -> &str {
        self.static_dir.as_deref().unwrap_or("static")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let json = r#"{
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "Test Author",
            "type": "webview",
            "python_entry": "python/app.py"
        }"#;

        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.r#type, PluginType::WebView);
    }
}
