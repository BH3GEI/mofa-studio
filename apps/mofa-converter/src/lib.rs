//! MoFA Content Converter
//!
//! A simple tool for converting between audio, video, and text formats

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaConverterApp;

impl MofaApp for MoFaConverterApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Converter",
            id: "mofa-converter",
            description: "Convert between audio, video, and text formats",
            tab_id: Some(live_id!(converter_tab)),
            page_id: Some(live_id!(converter_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
