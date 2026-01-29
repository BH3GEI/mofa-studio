//! MoFA WebView Demo - Demonstrates WebView embedding in Makepad

pub mod screen;

pub use screen::WebViewDemoScreen;
pub use screen::WebViewDemoScreenWidgetRefExt;

use makepad_widgets::{Cx, live_id, LiveId};
use mofa_widgets::{AppInfo, MofaApp};

/// MoFA WebView Demo app descriptor
pub struct MoFaWebViewDemoApp;

impl MofaApp for MoFaWebViewDemoApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "WebView Demo 1",
            id: "mofa-webview-demo",
            description: "Demonstrates WebView embedding with wry",
            tab_id: Some(live_id!(webview_demo_tab)),
            page_id: Some(live_id!(webview_demo_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}

/// Register all MoFA WebView Demo widgets with Makepad
pub fn live_design(cx: &mut Cx) {
    MoFaWebViewDemoApp::live_design(cx);
}
