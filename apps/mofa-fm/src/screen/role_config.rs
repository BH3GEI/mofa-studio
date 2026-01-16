//! Role configuration handling - load and save TOML config files

use serde::Deserialize;
use std::path::PathBuf;

/// Role configuration loaded from TOML file
#[derive(Debug, Clone, Default)]
pub struct RoleConfig {
    pub default_model: String,
    pub system_prompt: String,
    pub models: Vec<String>,
    pub config_path: Option<PathBuf>,
}

/// Model definition from TOML
#[derive(Debug, Deserialize)]
struct TomlModel {
    id: String,
    #[allow(dead_code)]
    route: Option<TomlRoute>,
}

#[derive(Debug, Deserialize)]
struct TomlRoute {
    #[allow(dead_code)]
    provider: String,
    #[allow(dead_code)]
    model: String,
}

/// Partial TOML structure for reading
#[derive(Debug, Deserialize)]
struct TomlConfig {
    default_model: Option<String>,
    system_prompt: Option<String>,
    models: Option<Vec<TomlModel>>,
}

impl RoleConfig {
    /// Load configuration from a TOML file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: TomlConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        let models = config.models
            .map(|m| m.into_iter().map(|model| model.id).collect())
            .unwrap_or_default();

        Ok(RoleConfig {
            default_model: config.default_model.unwrap_or_default(),
            system_prompt: config.system_prompt.unwrap_or_default(),
            models,
            config_path: Some(path.clone()),
        })
    }

    /// Save model and system prompt back to the TOML file
    /// This preserves other fields in the file by doing a partial update
    pub fn save(&self) -> Result<(), String> {
        let path = self.config_path.as_ref()
            .ok_or_else(|| "No config path set".to_string())?;

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        // Parse the existing content to preserve other fields
        let mut doc: toml::Table = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        // Update only the fields we manage
        doc.insert("default_model".to_string(), toml::Value::String(self.default_model.clone()));
        doc.insert("system_prompt".to_string(), toml::Value::String(self.system_prompt.clone()));

        // Serialize back
        let new_content = toml::to_string_pretty(&doc)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(path, new_content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }
}

/// Get the config file path for a role
pub fn get_role_config_path(dataflow_path: Option<&PathBuf>, role: &str) -> PathBuf {
    let config_name = match role {
        "student1" => "study_config_student1.toml",
        "student2" => "study_config_student2.toml",
        "tutor" => "study_config_tutor.toml",
        _ => "study_config_student1.toml",
    };

    // Try to use the dataflow_path if set
    if let Some(dataflow_path) = dataflow_path {
        if let Some(parent) = dataflow_path.parent() {
            let config_path = parent.join(config_name);
            if config_path.exists() {
                return config_path;
            }
        }
    }

    // Fallback: search common locations
    let cwd = std::env::current_dir().unwrap_or_default();

    // First try: apps/mofa-fm/dataflow/ (workspace root)
    let app_path = cwd.join("apps").join("mofa-fm").join("dataflow").join(config_name);
    if app_path.exists() {
        return app_path;
    }

    // Second try: dataflow/ (run from app directory)
    let local_path = cwd.join("dataflow").join(config_name);
    if local_path.exists() {
        return local_path;
    }

    // Default
    app_path
}
