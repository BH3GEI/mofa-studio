//! MoFA Hello World (Rust)
//!
//! A simple example app that uses a Rust HTTP server for WebView content

pub mod screen;

use makepad_widgets::*;
use mofa_widgets::{AppInfo, MofaApp};

pub struct MoFaHelloWorldRustApp;

impl MofaApp for MoFaHelloWorldRustApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Hello World (Rust)",
            id: "mofa-hello-world-rust",
            description: "Hello World example with a Rust web server",
            tab_id: Some(live_id!(hello_world_rust_tab)),
            page_id: Some(live_id!(hello_world_rust_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}
