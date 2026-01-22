//! # WebView Container Widget
//!
//! Embeds a wry WebView within a Makepad application without modifying Makepad internals.
//!
//! ## Architecture
//!
//! This module uses platform-native APIs to obtain window handles, then creates
//! a wry WebView as a child window that overlays the Makepad rendering area.
//!
//! ## Usage
//!
//! ```rust,ignore
//! live_design! {
//!     use mofa_widgets::webview::WebViewContainer;
//!
//!     MyScreen = <View> {
//!         webview = <WebViewContainer> {
//!             width: Fill,
//!             height: 400,
//!             url: "http://localhost:5173"
//!         }
//!     }
//! }
//! ```
//!
//! ## Limitations
//!
//! - **Z-order**: WebView is always on top; Makepad elements cannot overlay it
//! - **Linux Wayland**: Only X11 is supported (wry limitation)
//! - **Multi-window**: Uses key window by default; multi-window needs extra handling
//! - **Timing**: Must initialize after window is created

pub mod ipc;
pub mod platform_handle;
pub mod wry_wrapper;

use makepad_widgets::*;
use std::sync::Arc;
use parking_lot::Mutex;

pub use self::ipc::{IpcHandler, IpcMessage};
pub use self::wry_wrapper::{ManagedWebView, WebViewBounds, WebViewConfig, WebViewError};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::SLATE_800;

    pub WebViewContainer = {{WebViewContainer}} <View> {
        width: Fill, height: 300
        show_bg: true

        draw_bg: {
            fn pixel(self) -> vec4 {
                // Placeholder background - WebView will overlay this
                return (SLATE_800);
            }
        }
    }
}

/// Actions emitted by WebViewContainer
#[derive(Clone, Debug, DefaultNone)]
pub enum WebViewAction {
    None,
    /// WebView has been initialized
    Initialized,
    /// WebView initialization failed
    InitFailed(String),
    /// Received IPC message from JavaScript
    IpcMessage { channel: String, data: String },
    /// URL navigation occurred
    UrlChanged(String),
}

/// WebViewContainer widget that embeds a wry WebView
#[derive(Live, LiveHook, Widget)]
pub struct WebViewContainer {
    #[deref]
    view: View,

    /// URL to load in the WebView
    #[live]
    url: String,

    /// Enable developer tools
    #[live(true)]
    devtools: bool,

    /// Transparent background
    #[live(false)]
    transparent: bool,

    /// Whether WebView is active (controls initialization and visibility)
    /// Set to false by default - must be activated explicitly
    #[rust]
    active: bool,

    /// The managed WebView instance
    #[rust]
    webview: Option<ManagedWebView>,

    /// Number of initialization attempts
    #[rust]
    init_attempts: u32,

    /// Cached absolute position
    #[rust]
    cached_rect: Option<Rect>,

    /// Frame count for delayed initialization
    #[rust]
    frame_count: u32,

    /// Last initialization attempt frame
    #[rust]
    last_init_frame: u32,
}

impl WebViewContainer {
    /// Maximum number of initialization attempts
    const MAX_INIT_ATTEMPTS: u32 = 10;
    /// Frames between retry attempts
    const RETRY_INTERVAL: u32 = 30;
    /// Initial delay frames before first attempt
    const INITIAL_DELAY: u32 = 10;

    /// Initialize the WebView
    ///
    /// This should be called after the window is created and the widget
    /// has been laid out at least once.
    fn initialize_webview(&mut self, cx: &mut Cx) {
        if self.webview.is_some() || self.init_attempts >= Self::MAX_INIT_ATTEMPTS {
            return;
        }

        self.init_attempts += 1;
        self.last_init_frame = self.frame_count;

        // Get widget bounds
        let bounds = if let Some(rect) = self.cached_rect {
            WebViewBounds {
                x: rect.pos.x as i32,
                y: rect.pos.y as i32,
                width: rect.size.x.max(1.0) as u32,
                height: rect.size.y.max(1.0) as u32,
            }
        } else {
            WebViewBounds {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            }
        };

        let url = if self.url.is_empty() {
            "about:blank".to_string()
        } else {
            self.url.clone()
        };

        let config = WebViewConfig {
            url,
            bounds,
            devtools: self.devtools,
            transparent: self.transparent,
            user_agent: None,
        };

        let mut webview = ManagedWebView::new(config);

        match webview.initialize() {
            Ok(()) => {
                ::log::info!("[WebViewContainer] WebView initialized successfully");

                // Inject IPC bridge
                if let Err(e) = webview.inject_ipc_bridge() {
                    ::log::warn!("[WebViewContainer] Failed to inject IPC bridge: {}", e);
                }

                self.webview = Some(webview);
                cx.widget_action(
                    self.widget_uid(),
                    &Scope::empty().path,
                    WebViewAction::Initialized,
                );
            }
            Err(e) => {
                ::log::error!("[WebViewContainer] Failed to initialize WebView: {}", e);
                cx.widget_action(
                    self.widget_uid(),
                    &Scope::empty().path,
                    WebViewAction::InitFailed(e.to_string()),
                );
            }
        }
    }

    /// Update the WebView position to match widget bounds
    fn sync_bounds(&mut self, rect: Rect) {
        if let Some(ref mut webview) = self.webview {
            let bounds = WebViewBounds {
                x: rect.pos.x as i32,
                y: rect.pos.y as i32,
                width: rect.size.x.max(1.0) as u32,
                height: rect.size.y.max(1.0) as u32,
            };

            if let Err(e) = webview.set_bounds(bounds) {
                ::log::warn!("[WebViewContainer] Failed to sync bounds: {}", e);
            }
        }
    }

    /// Navigate to a URL
    pub fn load_url(&self, url: &str) -> Result<(), WebViewError> {
        if let Some(ref webview) = self.webview {
            webview.load_url(url)
        } else {
            Err(WebViewError::NotInitialized)
        }
    }

    /// Execute JavaScript in the WebView
    pub fn eval(&self, js: &str) -> Result<(), WebViewError> {
        if let Some(ref webview) = self.webview {
            webview.eval(js)
        } else {
            Err(WebViewError::NotInitialized)
        }
    }

    /// Send a message to JavaScript
    pub fn send_to_js(&self, channel: &str, data: &str) -> Result<(), WebViewError> {
        if let Some(ref webview) = self.webview {
            webview.send_to_js(channel, data)
        } else {
            Err(WebViewError::NotInitialized)
        }
    }

    /// Get the IPC handler for registering callbacks
    pub fn ipc_handler(&self) -> Option<Arc<Mutex<IpcHandler>>> {
        self.webview.as_ref().map(|w| w.ipc_handler())
    }

    /// Check if WebView is initialized
    pub fn is_initialized(&self) -> bool {
        self.webview.as_ref().map_or(false, |w| w.is_initialized())
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) -> Result<(), WebViewError> {
        if let Some(ref mut webview) = self.webview {
            webview.set_visible(visible)
        } else {
            Err(WebViewError::NotInitialized)
        }
    }

    /// Set active state - controls whether WebView initializes and shows
    pub fn set_active(&mut self, cx: &mut Cx, active: bool) {
        if self.active == active {
            return;
        }
        self.active = active;

        if active {
            // Request NextFrame to trigger initialization
            cx.new_next_frame();
        } else {
            // Hide WebView when inactive
            if let Some(ref mut webview) = self.webview {
                let _ = webview.set_visible(false);
            }
        }
    }

    /// Get active state
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Widget for WebViewContainer {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Process IPC messages
        if let Some(ref webview) = self.webview {
            let messages = webview.ipc_handler().lock().poll_messages();
            for msg in messages {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    WebViewAction::IpcMessage {
                        channel: msg.channel,
                        data: msg.data,
                    },
                );
            }
        }

        match event {
            Event::NextFrame(_) => {
                self.frame_count += 1;

                // Only initialize when active
                if !self.active {
                    return;
                }

                // Delay initialization to ensure window is ready
                // Initial delay, then retry with interval if failed
                if self.webview.is_none() && self.init_attempts < Self::MAX_INIT_ATTEMPTS {
                    let should_try = if self.init_attempts == 0 {
                        // First attempt after initial delay
                        self.frame_count >= Self::INITIAL_DELAY
                    } else {
                        // Retry attempts with interval
                        self.frame_count >= self.last_init_frame + Self::RETRY_INTERVAL
                    };

                    if should_try {
                        ::log::info!(
                            "[WebViewContainer] Attempting initialization (attempt {}/{})",
                            self.init_attempts + 1,
                            Self::MAX_INIT_ATTEMPTS
                        );
                        self.initialize_webview(cx);
                    }
                } else if self.webview.is_some() {
                    // WebView is initialized and active - make sure it's visible
                    if let Some(ref mut webview) = self.webview {
                        let _ = webview.set_visible(true);
                    }
                }
            }
            Event::WindowGeomChange(_) => {
                // Sync bounds when window geometry changes (only if active)
                if self.active {
                    if let Some(rect) = self.cached_rect {
                        self.sync_bounds(rect);
                    }
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Request NextFrame for initialization timing and retries (only when active)
        if self.active && self.webview.is_none() && self.init_attempts < Self::MAX_INIT_ATTEMPTS {
            cx.new_next_frame();
        }

        let result = self.view.draw_walk(cx, scope, walk);

        // Cache the absolute rect for WebView positioning
        let rect = cx.turtle().rect();
        let new_rect = Rect {
            pos: rect.pos,
            size: rect.size,
        };

        // Update bounds if changed (only sync if active)
        if self.cached_rect != Some(new_rect) {
            self.cached_rect = Some(new_rect);
            if self.active && self.webview.is_some() {
                self.sync_bounds(new_rect);
            }
        }

        result
    }
}

impl WebViewContainerRef {
    /// Navigate to a URL
    pub fn load_url(&self, url: &str) -> Result<(), WebViewError> {
        if let Some(inner) = self.borrow() {
            inner.load_url(url)
        } else {
            Err(WebViewError::NotInitialized)
        }
    }

    /// Execute JavaScript
    pub fn eval(&self, js: &str) -> Result<(), WebViewError> {
        if let Some(inner) = self.borrow() {
            inner.eval(js)
        } else {
            Err(WebViewError::NotInitialized)
        }
    }

    /// Send message to JavaScript
    pub fn send_to_js(&self, channel: &str, data: &str) -> Result<(), WebViewError> {
        if let Some(inner) = self.borrow() {
            inner.send_to_js(channel, data)
        } else {
            Err(WebViewError::NotInitialized)
        }
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.borrow().map_or(false, |inner| inner.is_initialized())
    }

    /// Set visibility
    pub fn set_visible(&self, visible: bool) -> Result<(), WebViewError> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_visible(visible)
        } else {
            Err(WebViewError::NotInitialized)
        }
    }

    /// Set active state
    pub fn set_active(&self, cx: &mut Cx, active: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_active(cx, active);
        }
    }

    /// Get active state
    pub fn is_active(&self) -> bool {
        self.borrow().map_or(false, |inner| inner.is_active())
    }
}

