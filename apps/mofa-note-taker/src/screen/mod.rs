//! Note Taker Screen
//!
//! WebView-based note-taking application

use makepad_widgets::*;
use mofa_widgets::webview::{WebViewAction, WebViewContainerWidgetExt};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;

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
                    vec4(0.30, 0.55, 0.85, 1.0),
                    vec4(0.25, 0.45, 0.75, 1.0),
                    self.dark_mode
                );
                let hover_color = mix(
                    vec4(0.35, 0.60, 0.90, 1.0),
                    vec4(0.30, 0.50, 0.80, 1.0),
                    self.dark_mode
                );
                let pressed_color = mix(
                    vec4(0.25, 0.50, 0.80, 1.0),
                    vec4(0.22, 0.42, 0.70, 1.0),
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
            instance status: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.circle(4.0, 4.0, 4.0);
                let color = vec4(0.5, 0.5, 0.5, 1.0);
                if self.status > 0.5 && self.status < 1.5 {
                    color = vec4(0.3, 0.85, 0.4, 1.0);
                } else if self.status > 1.5 {
                    color = vec4(0.95, 0.7, 0.2, 1.0);
                }
                sdf.fill(color);
                return sdf.result;
            }
        }
    }

    pub NoteTakerScreen = {{NoteTakerScreen}} {
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

        // Main content area
        content = <View> {
            width: Fill, height: Fill

            webview_area = <View> {
                width: Fill, height: Fill
                flow: Down
                padding: 0

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

                    webview = <WebViewContainer> {
                        width: Fill, height: Fill
                        url: "about:blank"
                    }
                }
            }
        }

        // Status bar
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

            start_btn = <StartButton> {
                text: "Start Server"
            }

            back_btn = <NavButton> {
                text: "<"
            }

            forward_btn = <NavButton> {
                text: ">"
            }

            reload_btn = <NavButton> {
                text: "R"
            }

            <View> { width: 12, height: 1 }

            status_dot = <StatusDot> {}

            <View> { width: 8, height: 1 }

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

            <View> { width: Fill, height: 1 }

            version_label = <Label> {
                text: "Note Taker v0.1"
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

fn find_available_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
}

fn get_python_path() -> Option<PathBuf> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(target_dir) = exe_path.parent() {
            if let Some(workspace) = target_dir.parent().and_then(|p| p.parent()) {
                let python_path = workspace.join("apps/mofa-note-taker/python");
                if python_path.join("app.py").exists() {
                    return Some(python_path);
                }
            }
        }
    }

    let candidates = [
        "apps/mofa-note-taker/python",
        "../apps/mofa-note-taker/python",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.join("app.py").exists() {
            return Some(path);
        }
    }

    None
}

fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".mofa-studio")
        .join("note-taker.json")
}

fn load_python_config() -> String {
    let config_path = get_config_path();
    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(path) = json.get("python_path").and_then(|v| v.as_str()) {
                return path.to_string();
            }
        }
    }
    if std::path::Path::new("/opt/homebrew/bin/python3.11").exists() {
        "/opt/homebrew/bin/python3.11".to_string()
    } else if std::path::Path::new("/opt/homebrew/bin/python3").exists() {
        "/opt/homebrew/bin/python3".to_string()
    } else {
        "python3".to_string()
    }
}

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

    fn start(&mut self) -> Result<u16, String> {
        if self.process.is_some() {
            return Ok(self.port);
        }

        let port = find_available_port().ok_or("Failed to find available port")?;
        let python_path = get_python_path().ok_or("Python files not found")?;

        ::log::info!("Starting Note Taker server on port {}", port);
        ::log::info!("Python path: {:?}", python_path);

        let child = Command::new(&self.python_cmd)
            .current_dir(&python_path)
            .args(["app.py", &port.to_string()])
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
pub struct NoteTakerScreen {
    #[deref]
    view: View,

    #[rust]
    server: Arc<Mutex<PythonServer>>,

    #[rust]
    url_loaded: bool,
}

impl Widget for NoteTakerScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        };

        // Handle start button
        if self.view.button(ids!(status_bar.start_btn)).clicked(actions) {
            self.toggle_server(cx);
        }

        // Handle navigation
        if self.view.button(ids!(status_bar.back_btn)).clicked(actions) {
            self.go_back();
        }
        if self.view.button(ids!(status_bar.forward_btn)).clicked(actions) {
            self.go_forward();
        }
        if self.view.button(ids!(status_bar.reload_btn)).clicked(actions) {
            self.reload();
        }

        // Handle WebView events
        let our_webview = self.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
        let our_uid = our_webview.widget_uid();

        for action in actions {
            if let Some(wa) = action.as_widget_action() {
                if wa.widget_uid == our_uid {
                    match wa.cast() {
                        WebViewAction::Initialized => {
                            ::log::info!("Note Taker WebView initialized");
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

impl NoteTakerScreen {
    fn toggle_server(&mut self, cx: &mut Cx) {
        let is_running = {
            let server = self.server.lock().unwrap();
            server.is_running()
        };

        if is_running {
            let mut server = self.server.lock().unwrap();
            server.stop();
            drop(server);
            self.set_status(cx, "Server stopped", 0.0);
            self.url_loaded = false;
            self.view.button(ids!(status_bar.start_btn)).set_text(cx, "Start Server");
        } else {
            self.set_status(cx, "Starting server...", 2.0);

            let result = {
                let mut server = self.server.lock().unwrap();
                server.start()
            };

            match result {
                Ok(port) => {
                    ::log::info!("Note Taker server started on port {}", port);
                    self.set_status(cx, &format!("Server running on port {}", port), 2.0);
                    self.view.button(ids!(status_bar.start_btn)).set_text(cx, "Stop Server");

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
        self.view.label(ids!(status_bar.status_text)).set_text(cx, text);
        self.view.view(ids!(status_bar.status_dot)).apply_over(
            cx,
            live! {
                draw_bg: { status: (status) }
            },
        );
        self.view.redraw(cx);
    }
}

impl NoteTakerScreenRef {
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );

            inner.view.view(ids!(content.webview_area.webview_wrapper)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );

            inner.view.view(ids!(status_bar)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );

            inner.view.button(ids!(status_bar.start_btn)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );
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

            inner.view.label(ids!(status_bar.status_text)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );
            inner.view.label(ids!(status_bar.version_label)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );

            // Send theme to WebView
            let webview = inner.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
            let js = format!("if(window.setTheme) window.setTheme({});", dark_mode);
            let _ = webview.eval(&js);

            inner.view.redraw(cx);
        }
    }
}
