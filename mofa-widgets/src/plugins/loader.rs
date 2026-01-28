//! Plugin loader - discovers and loads plugins from the plugins directory

use super::{PluginManifest, PluginType};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::net::TcpListener;

/// A loaded plugin with its runtime state
#[derive(Debug)]
pub struct LoadedPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,

    /// Plugin directory path
    pub dir: PathBuf,

    /// Running Python server process (for WebView plugins)
    pub server_process: Option<Child>,

    /// Server port (for WebView plugins)
    pub server_port: Option<u16>,

    /// Whether the plugin is enabled
    pub enabled: bool,
}

impl LoadedPlugin {
    /// Create a new loaded plugin
    pub fn new(manifest: PluginManifest, dir: PathBuf) -> Self {
        Self {
            manifest,
            dir,
            server_process: None,
            server_port: None,
            enabled: true,
        }
    }

    /// Get the URL for this plugin's WebView
    pub fn get_url(&self) -> Option<String> {
        self.server_port.map(|port| format!("http://127.0.0.1:{}", port))
    }

    /// Start the plugin's Python server
    pub fn start_server(&mut self, python_cmd: &str) -> Result<u16, String> {
        if self.manifest.r#type != PluginType::WebView {
            return Err("Not a WebView plugin".to_string());
        }

        if self.server_process.is_some() {
            return self.server_port.ok_or_else(|| "Server running but no port".to_string());
        }

        // Find available port
        let port = find_available_port()
            .ok_or_else(|| "No available port".to_string())?;

        // Get Python entry path
        let python_entry = self.dir.join(self.manifest.get_python_entry());
        if !python_entry.exists() {
            return Err(format!("Python entry not found: {:?}", python_entry));
        }

        // Start the server
        let child = Command::new(python_cmd)
            .current_dir(&self.dir)
            .arg(&python_entry)
            .arg(port.to_string())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to start plugin server: {}", e))?;

        self.server_process = Some(child);
        self.server_port = Some(port);

        Ok(port)
    }

    /// Stop the plugin's server
    pub fn stop_server(&mut self) {
        if let Some(mut child) = self.server_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.server_port = None;
    }

    /// Check if server is running
    pub fn is_server_running(&self) -> bool {
        self.server_process.is_some()
    }
}

impl Drop for LoadedPlugin {
    fn drop(&mut self) {
        self.stop_server();
    }
}

/// Plugin loader - discovers and manages plugins
pub struct PluginLoader {
    /// Plugins directory
    plugins_dir: PathBuf,

    /// Loaded plugins by ID
    plugins: HashMap<String, LoadedPlugin>,

    /// Python command to use
    python_cmd: String,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        let plugins_dir = get_plugins_dir();

        // Ensure plugins directory exists
        if !plugins_dir.exists() {
            let _ = std::fs::create_dir_all(&plugins_dir);
        }

        Self {
            plugins_dir,
            plugins: HashMap::new(),
            python_cmd: get_python_cmd(),
        }
    }

    /// Get the plugins directory path
    pub fn plugins_dir(&self) -> &PathBuf {
        &self.plugins_dir
    }

    /// Scan and load all plugins from the plugins directory
    pub fn scan_plugins(&mut self) -> Vec<String> {
        let mut loaded = Vec::new();

        let entries = match std::fs::read_dir(&self.plugins_dir) {
            Ok(e) => e,
            Err(_) => return loaded,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            match PluginManifest::from_file(&manifest_path) {
                Ok(manifest) => {
                    let id = manifest.id.clone();
                    log::info!("Loaded plugin: {} v{}", manifest.name, manifest.version);
                    self.plugins.insert(id.clone(), LoadedPlugin::new(manifest, path));
                    loaded.push(id);
                }
                Err(e) => {
                    log::warn!("Failed to load plugin from {:?}: {}", path, e);
                }
            }
        }

        loaded
    }

    /// Get all loaded plugins
    pub fn plugins(&self) -> impl Iterator<Item = &LoadedPlugin> {
        self.plugins.values()
    }

    /// Get all loaded plugins (mutable)
    pub fn plugins_mut(&mut self) -> impl Iterator<Item = &mut LoadedPlugin> {
        self.plugins.values_mut()
    }

    /// Get a plugin by ID
    pub fn get_plugin(&self, id: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(id)
    }

    /// Get a plugin by ID (mutable)
    pub fn get_plugin_mut(&mut self, id: &str) -> Option<&mut LoadedPlugin> {
        self.plugins.get_mut(id)
    }

    /// Start a plugin's server
    pub fn start_plugin(&mut self, id: &str) -> Result<u16, String> {
        let python_cmd = self.python_cmd.clone();
        let plugin = self.plugins.get_mut(id)
            .ok_or_else(|| format!("Plugin not found: {}", id))?;

        plugin.start_server(&python_cmd)
    }

    /// Stop a plugin's server
    pub fn stop_plugin(&mut self, id: &str) {
        if let Some(plugin) = self.plugins.get_mut(id) {
            plugin.stop_server();
        }
    }

    /// Stop all plugin servers
    pub fn stop_all(&mut self) {
        for plugin in self.plugins.values_mut() {
            plugin.stop_server();
        }
    }

    /// Get plugins that should show in sidebar
    pub fn sidebar_plugins(&self) -> Vec<&LoadedPlugin> {
        self.plugins
            .values()
            .filter(|p| p.enabled && p.manifest.show_in_sidebar)
            .collect()
    }

    /// Get plugin count
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginLoader {
    fn drop(&mut self) {
        self.stop_all();
    }
}

/// Get the plugins directory path
fn get_plugins_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".mofa-studio")
        .join("plugins")
}

/// Find an available port
fn find_available_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
}

fn find_embedded_python_cmd() -> Option<String> {
    let exe_path = std::env::current_exe().ok()?;
    let macos_dir = exe_path.parent()?;
    let resources_dir = macos_dir.parent()?.join("Resources");
    let wrapper = resources_dir.join("python/bin/python3");
    if wrapper.exists() {
        return Some(wrapper.to_string_lossy().to_string());
    }
    let framework_cmd = resources_dir.join("python/Python.framework/Versions/Current/bin/python3");
    if framework_cmd.exists() {
        return Some(framework_cmd.to_string_lossy().to_string());
    }
    let versions_dir = resources_dir.join("python/Python.framework/Versions");
    if let Ok(entries) = std::fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("bin/python3");
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Get the Python command to use
fn get_python_cmd() -> String {
    if let Some(cmd) = find_embedded_python_cmd() {
        return cmd;
    }
    // Check common locations
    let candidates = [
        "/opt/homebrew/bin/python3.11",
        "/opt/homebrew/bin/python3",
        "/usr/local/bin/python3",
        "python3",
    ];

    for cmd in candidates {
        if std::path::Path::new(cmd).exists() || cmd == "python3" {
            return cmd.to_string();
        }
    }

    "python3".to_string()
}
