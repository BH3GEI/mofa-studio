//! MoFA Podcast - AI Podcast Generator
//!
//! Generate podcast audio from scripts using macOS TTS

pub mod models;
pub mod services;
pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaPodcastApp;

impl MofaApp for MoFaPodcastApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Podcast",
            id: "mofa-podcast",
            description: "AI podcast generator from scripts",
            tab_id: Some(live_id!(podcast_tab)),
            page_id: Some(live_id!(podcast_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
