//! MoFA Transcriber App
//!
//! AI-powered audio/video transcription and summarization

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaTranscriberApp;

impl MofaApp for MoFaTranscriberApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Transcriber",
            id: "mofa-transcriber",
            description: "AI audio/video transcription and summarization",
            tab_id: Some(live_id!(transcriber_tab)),
            page_id: Some(live_id!(transcriber_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
