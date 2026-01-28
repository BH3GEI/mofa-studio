//! MoFA Alzheimer Web
//!
//! WebView app that serves the Alzheimer frontend via a local Rust HTTP server

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaAlzheimerWebApp;

impl MofaApp for MoFaAlzheimerWebApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Alzheimer",
            id: "mofa-alzheimer-web",
            description: "Alzheimer prevention frontend",
            tab_id: Some(live_id!(alzheimer_web_tab)),
            page_id: Some(live_id!(alzheimer_web_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
