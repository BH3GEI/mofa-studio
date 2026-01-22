pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaPersonalNewsApp;

impl MofaApp for MoFaPersonalNewsApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Personal News",
            id: "mofa-personal-news",
            description: "Personal News Broadcast",
            tab_id: Some(live_id!(personal_news_tab)),
            page_id: Some(live_id!(personal_news_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
