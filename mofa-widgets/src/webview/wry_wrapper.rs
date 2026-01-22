//! Wry WebView wrapper for Makepad integration
//!
//! This module provides a high-level wrapper around wry's WebView,
//! managing lifecycle, positioning, and IPC communication.

use std::sync::Arc;
use parking_lot::Mutex;
use wry::{WebView, WebViewBuilder, Rect};
use raw_window_handle::{HasWindowHandle, HandleError};

use super::ipc::{IpcHandler, IpcMessage};
use super::platform_handle::{get_native_handle, NativeWindowHandle, PlatformHandleError};

/// Configuration for creating a WebView
#[derive(Debug, Clone)]
pub struct WebViewConfig {
    /// Initial URL to load
    pub url: String,
    /// Initial position and size
    pub bounds: WebViewBounds,
    /// Enable developer tools (default: false in release)
    pub devtools: bool,
    /// Transparent background
    pub transparent: bool,
    /// Custom user agent
    pub user_agent: Option<String>,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            url: "about:blank".to_string(),
            bounds: WebViewBounds::default(),
            devtools: cfg!(debug_assertions),
            transparent: false,
            user_agent: None,
        }
    }
}

/// Position and size of the WebView
#[derive(Debug, Clone, Copy, Default)]
pub struct WebViewBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl WebViewBounds {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

impl From<WebViewBounds> for Rect {
    fn from(b: WebViewBounds) -> Self {
        Rect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                b.x as f64,
                b.y as f64,
            )),
            size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                b.width as f64,
                b.height as f64,
            )),
        }
    }
}

/// Error type for WebView operations
#[derive(Debug)]
pub enum WebViewError {
    PlatformHandle(PlatformHandleError),
    WryError(wry::Error),
    NotInitialized,
    AlreadyInitialized,
}

impl std::fmt::Display for WebViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlatformHandle(e) => write!(f, "Platform handle error: {}", e),
            Self::WryError(e) => write!(f, "Wry error: {}", e),
            Self::NotInitialized => write!(f, "WebView not initialized"),
            Self::AlreadyInitialized => write!(f, "WebView already initialized"),
        }
    }
}

impl std::error::Error for WebViewError {}

impl From<PlatformHandleError> for WebViewError {
    fn from(e: PlatformHandleError) -> Self {
        Self::PlatformHandle(e)
    }
}

impl From<wry::Error> for WebViewError {
    fn from(e: wry::Error) -> Self {
        Self::WryError(e)
    }
}

/// A wrapper struct that implements HasWindowHandle for NativeWindowHandle
struct WindowHandleWrapper {
    handle: NativeWindowHandle,
}

impl HasWindowHandle for WindowHandleWrapper {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, HandleError> {
        let raw = self.handle.raw_handle();
        // SAFETY: The handle is valid for the lifetime of this wrapper
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}

/// Managed WebView instance
pub struct ManagedWebView {
    webview: Option<WebView>,
    config: WebViewConfig,
    ipc_handler: Arc<Mutex<IpcHandler>>,
    visible: bool,
}

impl ManagedWebView {
    /// Create a new managed WebView (not yet initialized)
    pub fn new(config: WebViewConfig) -> Self {
        Self {
            webview: None,
            config,
            ipc_handler: Arc::new(Mutex::new(IpcHandler::new())),
            visible: true,
        }
    }

    /// Initialize the WebView with the native window handle
    ///
    /// This must be called after the window is created and visible.
    pub fn initialize(&mut self) -> Result<(), WebViewError> {
        if self.webview.is_some() {
            return Err(WebViewError::AlreadyInitialized);
        }

        // Get the native window handle
        let native_handle = get_native_handle()?;
        let wrapper = WindowHandleWrapper { handle: native_handle };

        // Clone IPC handler for the closure
        let ipc = self.ipc_handler.clone();

        // Build the WebView
        let mut builder = WebViewBuilder::new()
            .with_bounds(self.config.bounds.into())
            .with_url(&self.config.url)
            .with_devtools(self.config.devtools)
            .with_transparent(self.config.transparent)
            .with_ipc_handler(move |msg| {
                let mut handler = ipc.lock();
                handler.handle_message(IpcMessage::from_js(msg.body()));
            });

        if let Some(ref ua) = self.config.user_agent {
            builder = builder.with_user_agent(ua);
        }

        // Build as child window
        let webview = builder.build_as_child(&wrapper)?;

        self.webview = Some(webview);
        Ok(())
    }

    /// Check if the WebView is initialized
    pub fn is_initialized(&self) -> bool {
        self.webview.is_some()
    }

    /// Update the WebView bounds
    pub fn set_bounds(&mut self, bounds: WebViewBounds) -> Result<(), WebViewError> {
        if let Some(ref webview) = self.webview {
            webview.set_bounds(bounds.into())?;
            self.config.bounds = bounds;
        }
        Ok(())
    }

    /// Get current bounds
    pub fn bounds(&self) -> WebViewBounds {
        self.config.bounds
    }

    /// Navigate to a URL
    pub fn load_url(&self, url: &str) -> Result<(), WebViewError> {
        if let Some(ref webview) = self.webview {
            webview.load_url(url)?;
        } else {
            return Err(WebViewError::NotInitialized);
        }
        Ok(())
    }

    /// Execute JavaScript in the WebView
    pub fn eval(&self, js: &str) -> Result<(), WebViewError> {
        if let Some(ref webview) = self.webview {
            webview.evaluate_script(js)?;
        } else {
            return Err(WebViewError::NotInitialized);
        }
        Ok(())
    }

    /// Go back in navigation history
    pub fn go_back(&self) -> Result<(), WebViewError> {
        // Use JavaScript history API since wry doesn't expose direct back/forward
        self.eval("history.back()")
    }

    /// Go forward in navigation history
    pub fn go_forward(&self) -> Result<(), WebViewError> {
        // Use JavaScript history API since wry doesn't expose direct back/forward
        self.eval("history.forward()")
    }

    /// Reload the current page
    pub fn reload(&self) -> Result<(), WebViewError> {
        self.eval("location.reload()")
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) -> Result<(), WebViewError> {
        if let Some(ref webview) = self.webview {
            webview.set_visible(visible)?;
            self.visible = visible;
        }
        Ok(())
    }

    /// Check visibility
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the IPC handler for registering callbacks
    pub fn ipc_handler(&self) -> Arc<Mutex<IpcHandler>> {
        self.ipc_handler.clone()
    }

    /// Send a message to JavaScript
    pub fn send_to_js(&self, channel: &str, data: &str) -> Result<(), WebViewError> {
        let js = format!(
            r#"
            if (window.__mofa_ipc && window.__mofa_ipc.receive) {{
                window.__mofa_ipc.receive("{}", {});
            }}
            "#,
            channel,
            data
        );
        self.eval(&js)
    }

    /// Inject the IPC bridge JavaScript
    pub fn inject_ipc_bridge(&self) -> Result<(), WebViewError> {
        let js = r#"
            window.__mofa_ipc = {
                callbacks: {},

                // Send message to Rust
                send: function(channel, data) {
                    window.ipc.postMessage(JSON.stringify({
                        channel: channel,
                        data: data
                    }));
                },

                // Register callback for messages from Rust
                on: function(channel, callback) {
                    if (!this.callbacks[channel]) {
                        this.callbacks[channel] = [];
                    }
                    this.callbacks[channel].push(callback);
                },

                // Called by Rust to deliver messages
                receive: function(channel, data) {
                    if (this.callbacks[channel]) {
                        this.callbacks[channel].forEach(function(cb) {
                            try { cb(data); } catch(e) { console.error(e); }
                        });
                    }
                }
            };
            console.log('[MoFA] IPC bridge initialized');
        "#;
        self.eval(js)
    }
}

impl Drop for ManagedWebView {
    fn drop(&mut self) {
        // WebView cleanup is handled by wry
        self.webview = None;
    }
}
