//! MoFA Hello World
//!
//! A simple example plugin to demonstrate the plugin system

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaHelloWorldApp;

impl MofaApp for MoFaHelloWorldApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Hello World (Python)",
            id: "mofa-hello-world",
            description: "A simple example plugin to demonstrate the plugin system",
            tab_id: Some(live_id!(hello_world_tab)),
            page_id: Some(live_id!(hello_world_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
