//! MoFA.fm Web
//!
//! Embedded MoFA.fm website in a WebView

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaFmWebApp;

impl MofaApp for MoFaFmWebApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "MoFA.fm",
            id: "mofa-fm-web",
            description: "Embedded MoFA.fm website",
            tab_id: Some(live_id!(mofa_fm_web_tab)),
            page_id: Some(live_id!(mofa_fm_web_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
