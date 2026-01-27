//! Generic plugin screen - WebView container for dynamic plugins

use makepad_widgets::*;
use crate::webview::{WebViewAction, WebViewContainerWidgetExt};
use super::PluginLoader;
use std::sync::{Arc, Mutex};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::*;
    use crate::webview::WebViewContainer;

    // Status bar button
    PluginNavButton = <Button> {
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
                let color = mix(base, hover_color, self.hover);
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

    PluginStartButton = <Button> {
        width: Fit, height: 28
        padding: {left: 12, right: 12}
        margin: {right: 8}
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
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
                sdf.fill(mix(base, hover_color, self.hover));
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

    pub PluginScreen = {{PluginScreen}} {
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

            start_btn = <PluginStartButton> {
                text: "Start"
            }

            back_btn = <PluginNavButton> { text: "<" }
            forward_btn = <PluginNavButton> { text: ">" }
            reload_btn = <PluginNavButton> { text: "R" }

            <View> { width: 12, height: 1 }

            status_dot = <StatusDot> {}

            <View> { width: 8, height: 1 }

            status_text = <Label> {
                text: "Not running"
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

            plugin_name = <Label> {
                text: "Plugin"
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

#[derive(Live, LiveHook, Widget)]
pub struct PluginScreen {
    #[deref]
    view: View,

    /// Plugin ID this screen is bound to
    #[rust]
    plugin_id: Option<String>,

    /// Reference to the plugin loader (shared)
    #[rust]
    loader: Option<Arc<Mutex<PluginLoader>>>,

    /// Whether URL has been loaded
    #[rust]
    url_loaded: bool,

    /// Timer for delayed URL loading after server start
    #[rust]
    load_url_timer: Timer,

    /// Whether we're waiting to load URL
    #[rust]
    pending_url_load: bool,
}

impl Widget for PluginScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Handle timer for delayed URL loading
        if self.load_url_timer.is_event(event).is_some() {
            if self.pending_url_load {
                self.pending_url_load = false;
                self.load_url(cx);
            }
        }

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
                            if self.is_server_running() {
                                self.load_url(cx);
                            }
                        }
                        WebViewAction::UrlChanged(url) => {
                            if url != "about:blank" {
                                self.set_status(cx, "Connected", 1.0);
                            }
                        }
                        WebViewAction::InitFailed(err) => {
                            self.set_status(cx, &format!("WebView error: {}", err), 0.0);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl PluginScreen {
    /// Bind this screen to a plugin
    pub fn bind_plugin(&mut self, cx: &mut Cx, plugin_id: String, loader: Arc<Mutex<PluginLoader>>) {
        self.plugin_id = Some(plugin_id.clone());
        self.loader = Some(loader.clone());

        // Update UI with plugin info
        if let Ok(loader) = loader.lock() {
            if let Some(plugin) = loader.get_plugin(&plugin_id) {
                let name = format!("{} v{}", plugin.manifest.name, plugin.manifest.version);
                self.view.label(ids!(status_bar.plugin_name)).set_text(cx, &name);
            }
        }
    }

    fn toggle_server(&mut self, cx: &mut Cx) {
        let plugin_id = match &self.plugin_id {
            Some(id) => id.clone(),
            None => return,
        };
        let loader = match &self.loader {
            Some(l) => l.clone(),
            None => return,
        };

        let is_running = self.is_server_running();

        if is_running {
            if let Ok(mut loader) = loader.lock() {
                loader.stop_plugin(&plugin_id);
            }
            self.set_status(cx, "Stopped", 0.0);
            self.url_loaded = false;
            self.view.button(ids!(status_bar.start_btn)).set_text(cx, "Start");
        } else {
            self.set_status(cx, "Starting...", 2.0);

            let result = if let Ok(mut loader) = loader.lock() {
                loader.start_plugin(&plugin_id)
            } else {
                Err("Loader unavailable".to_string())
            };

            match result {
                Ok(port) => {
                    self.set_status(cx, &format!("Running on port {}", port), 2.0);
                    self.view.button(ids!(status_bar.start_btn)).set_text(cx, "Stop");

                    // Schedule URL load after server has time to start
                    self.pending_url_load = true;
                    self.load_url_timer = cx.start_timeout(1.0); // 1 second delay
                }
                Err(e) => {
                    self.set_status(cx, &format!("Error: {}", e), 0.0);
                }
            }
        }
    }

    fn is_server_running(&self) -> bool {
        let Some(plugin_id) = &self.plugin_id else { return false };
        let Some(loader) = &self.loader else { return false };

        if let Ok(loader) = loader.lock() {
            if let Some(plugin) = loader.get_plugin(plugin_id) {
                return plugin.is_server_running();
            }
        }
        false
    }

    fn load_url(&mut self, cx: &mut Cx) {
        let Some(plugin_id) = &self.plugin_id else { return };
        let Some(loader) = &self.loader else { return };

        let url = if let Ok(loader) = loader.lock() {
            loader.get_plugin(plugin_id).and_then(|p| p.get_url())
        } else {
            None
        };

        if let Some(url) = url {
            self.url_loaded = true;
            let webview = self.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
            if let Err(e) = webview.load_url(&url) {
                self.set_status(cx, &format!("Load error: {}", e), 0.0);
            } else {
                self.set_status(cx, "Loading...", 2.0);
            }
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

impl PluginScreenRef {
    pub fn bind_plugin(&self, cx: &mut Cx, plugin_id: String, loader: Arc<Mutex<PluginLoader>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.bind_plugin(cx, plugin_id, loader);
        }
    }

    /// Bind plugin and automatically start the server
    pub fn bind_plugin_and_start(&self, cx: &mut Cx, plugin_id: String, loader: Arc<Mutex<PluginLoader>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.bind_plugin(cx, plugin_id, loader);
            // Auto-start server if not already running
            if !inner.is_server_running() {
                inner.toggle_server(cx);
            }
        }
    }

    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } });
            inner.view.view(ids!(content.webview_area.webview_wrapper)).apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } });
            inner.view.view(ids!(status_bar)).apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } });
            inner.view.button(ids!(status_bar.start_btn)).apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } });
            inner.view.button(ids!(status_bar.back_btn)).apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } draw_text: { dark_mode: (dark_mode) } });
            inner.view.button(ids!(status_bar.forward_btn)).apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } draw_text: { dark_mode: (dark_mode) } });
            inner.view.button(ids!(status_bar.reload_btn)).apply_over(cx, live! { draw_bg: { dark_mode: (dark_mode) } draw_text: { dark_mode: (dark_mode) } });
            inner.view.label(ids!(status_bar.status_text)).apply_over(cx, live! { draw_text: { dark_mode: (dark_mode) } });
            inner.view.label(ids!(status_bar.plugin_name)).apply_over(cx, live! { draw_text: { dark_mode: (dark_mode) } });

            // Send theme to WebView
            let webview = inner.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
            let js = format!("if(window.setTheme) window.setTheme({});", dark_mode);
            let _ = webview.eval(&js);

            inner.view.redraw(cx);
        }
    }

    pub fn set_active(&self, cx: &mut Cx, active: bool) {
        if let Some(inner) = self.borrow() {
            let webview = inner.view.web_view_container(ids!(content.webview_area.webview_wrapper.webview));
            webview.set_active(cx, active);
        }
    }
}
