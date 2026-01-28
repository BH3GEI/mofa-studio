//! WebView Placeholder Screen
//!
//! WebView-based app with an embedded Rust HTTP server

use makepad_widgets::*;
use mofa_widgets::webview::{WebViewAction, WebViewContainerWidgetExt};
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const FALLBACK_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>WebView Placeholder</title>
  <style>
    body { font-family: system-ui, sans-serif; padding: 32px; background: #0f1115; color: #e0e0e5; }
    code { background: #1a1d24; padding: 2px 6px; border-radius: 4px; }
    a { color: #4a9eff; }
  </style>
</head>
<body>
  <h1>WebView Placeholder</h1>
  <p>This page is a placeholder. Replace it with your real frontend build.</p>
  <p>Suggested flow:</p>
  <pre><code>cd external/webview-placeholder
npm install
npm run build</code></pre>
  <p>Then reopen this page.</p>
</body>
</html>"#;

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

    pub WebViewPlaceholderScreen = {{WebViewPlaceholderScreen}} {
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
                text: "WebView Placeholder v1.0"
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

struct ServerAssets {
    index_html: String,
    static_root: Option<PathBuf>,
}

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn resolve_static_root() -> Option<PathBuf> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(macos_dir) = exe_path.parent() {
            let resources_path = macos_dir
                .parent()
                .map(|p| p.join("Resources/external/webview-placeholder/dist"));
            if let Some(ref path) = resources_path {
                if path.join("index.html").exists() {
                    return Some(path.clone());
                }
            }
        }

        if let Some(target_dir) = exe_path.parent() {
            if let Some(workspace) = target_dir.parent().and_then(|p| p.parent()) {
                let dist_path = workspace.join("external/webview-placeholder/dist");
                if dist_path.join("index.html").exists() {
                    return Some(dist_path);
                }
            }
        }
    }

    let candidates = [
        "external/webview-placeholder/dist",
        "../external/webview-placeholder/dist",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.join("index.html").exists() {
            return Some(path);
        }
    }

    None
}

fn load_index_html(static_root: Option<&PathBuf>) -> String {
    if let Some(root) = static_root {
        let path = root.join("index.html");
        if let Ok(content) = fs::read_to_string(&path) {
            return content;
        }
    }
    FALLBACK_HTML.to_string()
}

fn read_request(reader: &mut BufReader<TcpStream>) -> std::io::Result<Option<HttpRequest>> {
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(None);
    }
    if request_line.trim().is_empty() {
        return Ok(None);
    }

    let mut parts = request_line.trim().split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let raw_path = parts.next().unwrap_or("/").to_string();
    let path = raw_path.split('?').next().unwrap_or("/").to_string();

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
        if trimmed.is_empty() {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            if key.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    Ok(Some(HttpRequest { method, path, body }))
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

fn content_type_for_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "json" => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn load_static_file(path: &str, assets: &ServerAssets) -> Option<(Vec<u8>, &'static str)> {
    let root = assets.static_root.as_ref()?;
    let rel = path.trim_start_matches('/');
    if rel.is_empty() || rel.contains("..") {
        return None;
    }
    let full = root.join(rel);
    if !full.is_file() {
        return None;
    }
    let bytes = fs::read(full).ok()?;
    Some((bytes, content_type_for_path(rel)))
}

fn handle_connection(mut stream: TcpStream, assets: &ServerAssets) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let mut reader = match stream.try_clone() {
        Ok(s) => BufReader::new(s),
        Err(_) => return,
    };

    let request = match read_request(&mut reader) {
        Ok(Some(req)) => req,
        _ => return,
    };

    let method = request.method.as_str();
    let path = request.path.as_str();

    let (status, content_type, body) = match (method, path) {
        ("GET", "/health") => {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_millis() as u64;
            let body = serde_json::to_vec(&json!({
                "status": "ok",
                "timestamp": now_ms
            }))
            .unwrap_or_else(|_| b"{}".to_vec());
            ("200 OK", "application/json; charset=utf-8", body)
        }
        ("GET", "/") | ("GET", "/index.html") => {
            let body = assets.index_html.as_bytes().to_vec();
            ("200 OK", "text/html; charset=utf-8", body)
        }
        ("GET", _) => {
            if let Some((bytes, ctype)) = load_static_file(path, assets) {
                ("200 OK", ctype, bytes)
            } else {
                // SPA fallback
                let body = assets.index_html.as_bytes().to_vec();
                ("200 OK", "text/html; charset=utf-8", body)
            }
        }
        _ => {
            let body = b"Method Not Allowed".to_vec();
            ("405 Method Not Allowed", "text/plain; charset=utf-8", body)
        }
    };

    let _ = write_response(&mut stream, status, content_type, &body);
}

fn server_loop(listener: TcpListener, shutdown_rx: mpsc::Receiver<()>, assets: Arc<ServerAssets>) {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                handle_connection(stream, &assets);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(30));
            }
            Err(_) => break,
        }
    }
}

struct RustServer {
    handle: Option<thread::JoinHandle<()>>,
    shutdown: Option<mpsc::Sender<()>>,
    port: u16,
}

impl Default for RustServer {
    fn default() -> Self {
        Self {
            handle: None,
            shutdown: None,
            port: 0,
        }
    }
}

impl RustServer {
    fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    fn start(&mut self) -> Result<u16, String> {
        if self.handle.is_some() {
            return Ok(self.port);
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to bind server: {}", e))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("Failed to read port: {}", e))?
            .port();
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

        let static_root = resolve_static_root();
        let assets = Arc::new(ServerAssets {
            index_html: load_index_html(static_root.as_ref()),
            static_root,
        });

        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || server_loop(listener, rx, assets));

        self.shutdown = Some(tx);
        self.handle = Some(handle);
        self.port = port;

        Ok(port)
    }

    fn stop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        self.port = 0;
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for RustServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct WebViewPlaceholderScreen {
    #[deref]
    view: View,

    #[rust]
    server: Arc<Mutex<RustServer>>,
}

impl Widget for WebViewPlaceholderScreen {
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
                            ::log::info!("WebView placeholder initialized");
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

impl WebViewPlaceholderScreen {
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
            self.view.button(ids!(status_bar.start_btn)).set_text(cx, "Start Server");
        } else {
            self.set_status(cx, "Starting server...", 2.0);

            let result = {
                let mut server = self.server.lock().unwrap();
                server.start()
            };

            match result {
                Ok(port) => {
                    ::log::info!("WebView placeholder server started on port {}", port);
                    self.set_status(cx, &format!("Server running on port {}", port), 2.0);
                    self.view.button(ids!(status_bar.start_btn)).set_text(cx, "Stop Server");
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

impl WebViewPlaceholderScreenRef {
    pub fn start_server(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            let is_running = {
                let server = inner.server.lock().unwrap();
                server.is_running()
            };

            if !is_running {
                inner.toggle_server(cx);
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
                inner.toggle_server(cx);
            }
        }
    }

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
