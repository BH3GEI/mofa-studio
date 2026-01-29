//! MoFA WebView Placeholder
//!
//! WebView app that serves a placeholder frontend via a local Rust HTTP server

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaWebViewPlaceholderApp;

impl MofaApp for MoFaWebViewPlaceholderApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "WebView Demo 2",
            id: "mofa-webview-placeholder",
            description: "Placeholder WebView app (replace with real frontend)",
            tab_id: Some(live_id!(webview_placeholder_tab)),
            page_id: Some(live_id!(webview_placeholder_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
