//! MoFA Note Taker
//!
//! A simple note-taking application with WebView UI

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaNoteTakerApp;

impl MofaApp for MoFaNoteTakerApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Note Taker",
            id: "mofa-note-taker",
            description: "A simple note-taking application",
            tab_id: Some(live_id!(note_taker_tab)),
            page_id: Some(live_id!(note_taker_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
