//! Personal News Screen
//!
//! WebView-based Personal News display with embedded Python server

use makepad_widgets::*;
use mofa_widgets::webview::{WebViewAction, WebViewContainerWidgetExt};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;
use std::io::Write;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;
    use mofa_widgets::webview::WebViewContainer;

    // Navigation button style
    NavButton = <Button> {
        width: 32, height: 28
        padding: 0
        margin: {right: 4}
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance pressed: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let base = mix(
                    vec4(0.88, 0.89, 0.91, 1.0),
                    vec4(0.22, 0.24, 0.28, 1.0),
                    self.dark_mode
                );
                let hover_color = mix(
                    vec4(0.82, 0.84, 0.88, 1.0),
                    vec4(0.28, 0.30, 0.35, 1.0),
                    self.dark_mode
                );
                let pressed_color = mix(
                    vec4(0.75, 0.78, 0.82, 1.0),
                    vec4(0.32, 0.34, 0.40, 1.0),
                    self.dark_mode
                );
                let color = mix(mix(base, hover_color, self.hover), pressed_color, self.pressed);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 14.0 }
            fn get_color(self) -> vec4 {
                return mix(
                    vec4(0.3, 0.3, 0.35, 1.0),
                    vec4(0.85, 0.85, 0.9, 1.0),
                    self.dark_mode
                );
            }
        }
    }

    // Start server button style
    StartButton = <Button> {
        width: Fit, height: 28
        padding: {left: 12, right: 12}
        margin: {right: 8}
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance pressed: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let base = mix(
                    vec4(0.25, 0.65, 0.45, 1.0),
                    vec4(0.20, 0.55, 0.38, 1.0),
                    self.dark_mode
                );
                let hover_color = mix(
                    vec4(0.30, 0.70, 0.50, 1.0),
                    vec4(0.25, 0.60, 0.42, 1.0),
                    self.dark_mode
                );
                let pressed_color = mix(
                    vec4(0.20, 0.55, 0.38, 1.0),
                    vec4(0.18, 0.48, 0.32, 1.0),
                    self.dark_mode
                );
                let color = mix(mix(base, hover_color, self.hover), pressed_color, self.pressed);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            text_style: { font_size: 12.0 }
            fn get_color(self) -> vec4 {
                return vec4(1.0, 1.0, 1.0, 0.95);
            }
        }
    }

    // Status indicator dot
    StatusDot = <View> {
        width: 8, height: 8
        show_bg: true
        draw_bg: {
            instance status: 0.0  // 0=disconnected, 1=connected, 2=loading
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.circle(4.0, 4.0, 4.0);
                let color = vec4(0.5, 0.5, 0.5, 1.0);
                if self.status > 0.5 && self.status < 1.5 {
                    color = vec4(0.3, 0.85, 0.4, 1.0);  // Green - connected
                } else if self.status > 1.5 {
                    color = vec4(0.95, 0.7, 0.2, 1.0);  // Orange - loading
                }
                sdf.fill(color);
                return sdf.result;
            }
        }
    }

    // Config panel style
    ConfigPanel = <View> {
        width: Fill, height: Fit
        flow: Right
        align: {y: 0.5}
        padding: {left: 12, right: 12, top: 8, bottom: 8}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix(
                    vec4(0.96, 0.97, 0.98, 1.0),
                    vec4(0.14, 0.15, 0.18, 1.0),
                    self.dark_mode
                );
            }
        }
    }

    // Text input style
    ConfigInput = <TextInput> {
        width: Fill, height: 28
        padding: {left: 8, right: 8}
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let bg = mix(
                    vec4(1.0, 1.0, 1.0, 1.0),
                    vec4(0.18, 0.19, 0.22, 1.0),
                    self.dark_mode
                );
                sdf.fill(bg);
                let border = mix(
                    vec4(0.8, 0.82, 0.85, 1.0),
                    vec4(0.3, 0.32, 0.36, 1.0),
                    self.dark_mode
                );
                sdf.stroke(border, 1.0);
                return sdf.result;
            }
        }
        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 11.0 }
            fn get_color(self) -> vec4 {
                return mix(
                    vec4(0.2, 0.2, 0.25, 1.0),
                    vec4(0.85, 0.85, 0.9, 1.0),
                    self.dark_mode
                );
            }
        }
    }

    pub PersonalNewsScreen = {{PersonalNewsScreen}} {
        width: Fill, height: Fill
        flow: Down
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix(
                    vec4(0.92, 0.93, 0.95, 1.0),
                    vec4(0.10, 0.11, 0.14, 1.0),
                    self.dark_mode
                );
            }
        }

        // Main content area - WebView fills the space
        content = <View> {
            width: Fill, height: Fill

            // WebView container area
            webview_area = <View> {
                width: Fill, height: Fill
                flow: Down
                padding: 0

                // WebView wrapper
                webview_wrapper = <RoundedView> {
                    width: Fill, height: Fill
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: 0.0
                        fn get_color(self) -> vec4 {
                            return mix(
                                vec4(1.0, 1.0, 1.0, 1.0),
                                vec4(0.15, 0.16, 0.20, 1.0),
                                self.dark_mode
                            );
                        }
                    }

                    // The actual WebView
                    webview = <WebViewContainer> {
                        width: Fill, height: Fill
                        url: "about:blank"
                    }
                }
            }
        }

        // Config panel (hidden by default)
        config_panel = <ConfigPanel> {
            visible: false

            python_label = <Label> {
                width: Fit
                margin: {right: 8}
                text: "Python:"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: { font_size: 11.0 }
                    fn get_color(self) -> vec4 {
                        return mix(
                            vec4(0.3, 0.3, 0.35, 1.0),
                            vec4(0.7, 0.7, 0.75, 1.0),
                            self.dark_mode
                        );
                    }
                }
            }

            python_input = <ConfigInput> {
                text: "/opt/homebrew/bin/python3.11"
            }

            <View> { width: 8, height: 1 }

            save_btn = <NavButton> {
                width: Fit
                padding: {left: 12, right: 12}
                text: "Save"
            }
        }

        // Status bar with navigation
        status_bar = <View> {
            width: Fill, height: 36
            flow: Right
            align: {y: 0.5}
            padding: {left: 12, right: 16}
            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                fn pixel(self) -> vec4 {
                    return mix(
                        vec4(0.94, 0.95, 0.96, 1.0),
                        vec4(0.12, 0.13, 0.16, 1.0),
                        self.dark_mode
                    );
                }
            }

            // Start server button
            start_btn = <StartButton> {
                text: "Start Server"
            }

            // Config button
            config_btn = <NavButton> {
                width: Fit
                padding: {left: 8, right: 8}
                text: "⚙"
            }

            // Navigation buttons
            back_btn = <NavButton> {
                text: "<"
            }

            forward_btn = <NavButton> {
                text: ">"
            }

            reload_btn = <NavButton> {
                text: "R"
            }

            <View> { width: 12, height: 1 }  // Spacer

            status_dot = <StatusDot> {}

            <View> { width: 8, height: 1 }  // Spacer

            status_text = <Label> {
                text: "Server not running"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: { font_size: 11.0 }
                    fn get_color(self) -> vec4 {
                        return mix(
                            vec4(0.4, 0.4, 0.45, 1.0),
                            vec4(0.6, 0.6, 0.65, 1.0),
                            self.dark_mode
                        );
                    }
                }
            }

            <View> { width: Fill, height: 1 }  // Spacer

            version_label = <Label> {
                text: "Personal News v0.1"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: { font_size: 10.0 }
                    fn get_color(self) -> vec4 {
                        return mix(
                            vec4(0.5, 0.5, 0.55, 1.0),
                            vec4(0.5, 0.5, 0.55, 1.0),
                            self.dark_mode
                        );
                    }
                }
            }
        }
    }
}

/// Find an available port
fn find_available_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
}

/// Get the Python directory path (relative to the app crate)
fn get_python_path() -> Option<PathBuf> {
    // Try from executable location
    if let Ok(exe_path) = std::env::current_exe() {
        // Check inside app bundle (macOS): .app/Contents/Resources/apps/mofa-personal-news/web
        if let Some(macos_dir) = exe_path.parent() {
            let resources_path = macos_dir
                .parent() // Contents
                .map(|p| p.join("Resources/apps/mofa-personal-news/web"));
            if let Some(ref path) = resources_path {
                if path.join("app.py").exists() {
                    return Some(path.clone());
                }
            }
        }

        // Check development path: target/release -> workspace/apps/...
        if let Some(target_dir) = exe_path.parent() {
            if let Some(workspace) = target_dir.parent().and_then(|p| p.parent()) {
                let python_path = workspace.join("apps/mofa-personal-news/python/web");
                if python_path.join("app.py").exists() {
                    return Some(python_path);
                }
            }
        }
    }

    // Try relative paths
    let candidates = [
        "apps/mofa-personal-news/python/web",
        "../apps/mofa-personal-news/python/web",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.join("app.py").exists() {
            return Some(path);
        }
    }

    None
}

/// Get config file path
fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".mofa-studio")
        .join("personal-news.json")
}

fn find_embedded_python_cmd() -> Option<String> {
    let exe_path = std::env::current_exe().ok()?;
    let macos_dir = exe_path.parent()?;
    let resources_dir = macos_dir.parent()?.join("Resources");
    let wrapper = resources_dir.join("python/bin/python3");
    if wrapper.exists() {
        return Some(wrapper.to_string_lossy().to_string());
    }
    let framework_cmd = resources_dir.join("python/Python.framework/Versions/Current/bin/python3");
    if framework_cmd.exists() {
        return Some(framework_cmd.to_string_lossy().to_string());
    }
    let versions_dir = resources_dir.join("python/Python.framework/Versions");
    if let Ok(entries) = fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("bin/python3");
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Load Python path from config
fn load_python_config() -> String {
    let config_path = get_config_path();
    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(path) = json.get("python_path").and_then(|v| v.as_str()) {
                return path.to_string();
            }
        }
    }
    if let Some(cmd) = find_embedded_python_cmd() {
        return cmd;
    }
    // Default: try homebrew first
    if std::path::Path::new("/opt/homebrew/bin/python3.11").exists() {
        "/opt/homebrew/bin/python3.11".to_string()
    } else if std::path::Path::new("/opt/homebrew/bin/python3").exists() {
        "/opt/homebrew/bin/python3".to_string()
    } else {
        "python3".to_string()
    }
}

/// Save Python path to config
fn save_python_config(python_path: &str) -> Result<(), String> {
    let config_path = get_config_path();

    // Create directory if needed
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::json!({
        "python_path": python_path
    });

    let mut file = fs::File::create(&config_path).map_err(|e| e.to_string())?;
    file.write_all(json.to_string().as_bytes()).map_err(|e| e.to_string())?;

    ::log::info!("Saved Python config: {}", python_path);
    Ok(())
}

/// Python server manager
struct PythonServer {
    process: Option<Child>,
    port: u16,
    python_cmd: String,
}

impl Default for PythonServer {
    fn default() -> Self {
        Self {
            process: None,
            port: 0,
            python_cmd: load_python_config(),
        }
    }
}

impl PythonServer {
    fn is_running(&self) -> bool {
        self.process.is_some()
    }

    fn set_python_cmd(&mut self, cmd: String) {
        self.python_cmd = cmd;
    }

    fn start(&mut self) -> Result<u16, String> {
        if self.process.is_some() {
            return Ok(self.port);
        }

        // Find available port
        let port = find_available_port().ok_or("Failed to find available port")?;

        // Find Python path
        let python_path = get_python_path().ok_or("Python files not found")?;

        ::log::info!("Starting Python server on port {}", port);
        ::log::info!("Python path: {:?}", python_path);
        ::log::info!("Python command: {}", self.python_cmd);

        let child = Command::new(&self.python_cmd)
            .current_dir(&python_path)
            .args(["-c", &format!(
                r#"
import sys
sys.path.insert(0, '.')
sys.path.insert(0, '..')
from app import NewsRequestHandler
from http.server import HTTPServer
server = HTTPServer(('127.0.0.1', {}), NewsRequestHandler)
print('Server started on port {}', flush=True)
server.serve_forever()
"#,
                port, port
            )])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to start Python: {}", e))?;

        self.process = Some(child);
        self.port = port;

        Ok(port)
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
            self.port = 0;
        }
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for PythonServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct PersonalNewsScreen {
    #[deref]
    view: View,

    #[rust]
    server: Arc<Mutex<PythonServer>>,

    #[rust]
    url_loaded: bool,

    #[rust]
    config_visible: bool,

    #[rust]
    config_initialized: bool,
}

impl Widget for PersonalNewsScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Get actions from event
        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        };

        // Initialize config input on first run
        if !self.config_initialized {
            self.config_initialized = true;
            let python_path = load_python_config();
            self.view.text_input(ids!(config_panel.python_input)).set_text(cx, &python_path);
        }

        // Handle start button click
        if self.view.button(ids!(status_bar.start_btn)).clicked(actions) {
            self.start_server(cx);
        }

        // Handle config button click - toggle config panel
        if self.view.button(ids!(status_bar.config_btn)).clicked(actions) {
            self.config_visible = !self.config_visible;
            self.view.view(ids!(config_panel)).set_visible(cx, self.config_visible);
            self.view.redraw(cx);
        }

        // Handle save button click
        if self.view.button(ids!(config_panel.save_btn)).clicked(actions) {
            let python_path = self.view.text_input(ids!(config_panel.python_input)).text();
            if let Err(e) = save_python_config(&python_path) {
                self.set_status(cx, &format!("Save failed: {}", e), 0.0);
            } else {
                // Update server with new path
                let mut server = self.server.lock().unwrap();
                server.set_python_cmd(python_path);
                drop(server);
                self.set_status(cx, "Config saved", 1.0);
                // Hide config panel
                self.config_visible = false;
                self.view.view(ids!(config_panel)).set_visible(cx, false);
                self.view.redraw(cx);
            }
        }

        // Handle navigation button clicks
        if self.view.button(ids!(status_bar.back_btn)).clicked(actions) {
            self.go_back();
        }
        if self.view.button(ids!(status_bar.forward_btn)).clicked(actions) {
            self.go_forward();
        }
        if self.view.button(ids!(status_bar.reload_btn)).clicked(actions) {
            self.reload();
        }

        // Handle WebView events - check if it's from our WebView
        let our_webview = self.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
        let our_uid = our_webview.widget_uid();

        for action in actions {
            if let Some(wa) = action.as_widget_action() {
                // Only handle events from our WebView
                if wa.widget_uid == our_uid {
                    match wa.cast() {
                        WebViewAction::Initialized => {
                            ::log::info!("PersonalNews WebView initialized");
                            // If server is already running, load URL
                            let server = self.server.lock().unwrap();
                            if server.is_running() {
                                drop(server);
                                self.load_url(cx);
                            }
                        }
                        WebViewAction::InitFailed(err) => {
                            self.set_status(cx, &format!("WebView failed: {}", err), 0.0);
                        }
                        WebViewAction::UrlChanged(url) => {
                            ::log::info!("URL changed: {}", url);
                            if url != "about:blank" {
                                self.set_status(cx, "Connected", 1.0);
                            }
                        }
                        WebViewAction::IpcMessage { .. } | WebViewAction::None => {}
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl PersonalNewsScreen {
    fn start_server(&mut self, cx: &mut Cx) {
        let is_running = {
            let server = self.server.lock().unwrap();
            server.is_running()
        };

        if is_running {
            // Stop server
            let mut server = self.server.lock().unwrap();
            server.stop();
            drop(server);
            self.set_status(cx, "Server stopped", 0.0);
            self.url_loaded = false;
            // Update button text
            self.view.button(ids!(status_bar.start_btn)).set_text(cx, "Start Server");
        } else {
            // Start server
            self.set_status(cx, "Starting server...", 2.0);

            let result = {
                let mut server = self.server.lock().unwrap();
                server.start()
            };

            match result {
                Ok(port) => {
                    ::log::info!("Python server started on port {}", port);
                    self.set_status(cx, &format!("Server running on port {}", port), 2.0);
                    // Update button text
                    self.view.button(ids!(status_bar.start_btn)).set_text(cx, "Stop Server");

                    // Wait for server to be ready
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                    self.load_url(cx);
                }
                Err(e) => {
                    ::log::error!("Failed to start server: {}", e);
                    self.set_status(cx, &format!("Error: {}", e), 0.0);
                }
            }
        }
    }

    fn load_url(&mut self, cx: &mut Cx) {
        let url = {
            let server = self.server.lock().unwrap();
            if !server.is_running() {
                return;
            }
            server.url()
        };

        self.url_loaded = true;
        ::log::info!("Loading URL: {}", url);

        let webview = self.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
        if let Err(e) = webview.load_url(&url) {
            self.set_status(cx, &format!("Load error: {}", e), 0.0);
        } else {
            self.set_status(cx, "Loading...", 2.0);
        }
    }

    fn go_back(&self) {
        let webview = self.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
        let _ = webview.go_back();
    }

    fn go_forward(&self) {
        let webview = self.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
        let _ = webview.go_forward();
    }

    fn reload(&self) {
        let webview = self.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
        let _ = webview.reload();
    }

    fn set_status(&mut self, cx: &mut Cx, text: &str, status: f64) {
        self.view
            .label(ids!(status_bar.status_text))
            .set_text(cx, text);
        self.view.view(ids!(status_bar.status_dot)).apply_over(
            cx,
            live! {
                draw_bg: { status: (status) }
            },
        );
        self.view.redraw(cx);
    }
}

impl PersonalNewsScreenRef {
    pub fn start_server(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            let is_running = {
                let server = inner.server.lock().unwrap();
                server.is_running()
            };

            if !is_running {
                inner.start_server(cx);
            }
        }
    }

    pub fn stop_server(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            let is_running = {
                let server = inner.server.lock().unwrap();
                server.is_running()
            };

            if is_running {
                inner.start_server(cx);
            }
        }
    }

    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            // Main background
            inner.view.apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );

            // WebView wrapper
            inner
                .view
                .view(ids!(content.webview_area.webview_wrapper))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                    },
                );

            // Status bar
            inner.view.view(ids!(status_bar)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );

            // Navigation buttons
            inner.view.button(ids!(status_bar.back_btn)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                    draw_text: { dark_mode: (dark_mode) }
                },
            );
            inner.view.button(ids!(status_bar.forward_btn)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                    draw_text: { dark_mode: (dark_mode) }
                },
            );
            inner.view.button(ids!(status_bar.reload_btn)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                    draw_text: { dark_mode: (dark_mode) }
                },
            );

            inner
                .view
                .label(ids!(status_bar.status_text))
                .apply_over(
                    cx,
                    live! {
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );
            inner
                .view
                .label(ids!(status_bar.version_label))
                .apply_over(
                    cx,
                    live! {
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );

            inner.view.redraw(cx);
        }
    }
}
