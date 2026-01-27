//! MoFA Podcast Factory
//!
//! AI-powered multi-episode podcast series generator from books

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaPodcastFactoryApp;

impl MofaApp for MoFaPodcastFactoryApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Book Cast",
            id: "mofa-podcast-factory",
            description: "Generate multi-episode podcast series from books",
            tab_id: Some(live_id!(podcast_factory_tab)),
            page_id: Some(live_id!(podcast_factory_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
