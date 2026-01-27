//! Plugin system for MoFA Studio
//!
//! Supports two types of plugins:
//! - Native: Rust + Makepad (compiled into the app)
//! - WebView: Python + HTML (dynamically loaded)

mod manifest;
mod loader;
pub mod screen;

pub use manifest::{PluginManifest, PluginType};
pub use loader::{PluginLoader, LoadedPlugin};
pub use screen::{PluginScreen, PluginScreenRef, PluginScreenWidgetRefExt};

use makepad_widgets::Cx;

/// Register plugin widgets
pub fn live_design(cx: &mut Cx) {
    screen::live_design(cx);
}
