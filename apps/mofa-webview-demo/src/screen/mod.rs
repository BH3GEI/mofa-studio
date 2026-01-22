//! WebView Demo Screen
//!
//! A beautiful demo showcasing WebView embedding in Makepad

use makepad_widgets::*;
use mofa_widgets::webview::{WebViewAction, WebViewContainerWidgetExt};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;
    use mofa_widgets::webview::WebViewContainer;

    // Gradient header background
    GradientHeader = <View> {
        width: Fill, height: 56
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                let gradient = mix(
                    vec4(0.15, 0.25, 0.45, 1.0),  // Deep blue
                    vec4(0.25, 0.15, 0.40, 1.0),  // Purple tint
                    self.pos.x
                );
                return mix(gradient, gradient * 0.7, self.dark_mode);
            }
        }
    }

    // URL input field style
    UrlInput = <TextInput> {
        width: Fill, height: 36
        padding: {left: 12, right: 12}
        draw_bg: {
            instance dark_mode: 0.0
            instance radius: 8.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius);
                let bg = mix(
                    vec4(1.0, 1.0, 1.0, 0.95),
                    vec4(0.15, 0.17, 0.22, 0.95),
                    self.dark_mode
                );
                sdf.fill(bg);
                return sdf.result;
            }
        }
        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 12.0 }
            fn get_color(self) -> vec4 {
                return mix(
                    vec4(0.1, 0.1, 0.15, 1.0),
                    vec4(0.9, 0.9, 0.95, 1.0),
                    self.dark_mode
                );
            }
        }
        draw_cursor: {
            instance focus: 0.0
            uniform border_radius: 0.5
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                sdf.fill(mix(vec4(0.3, 0.5, 0.9, 1.0), vec4(0.5, 0.7, 1.0, 1.0), self.focus));
                return sdf.result;
            }
        }
        draw_selection: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 2.0);
                sdf.fill(vec4(0.3, 0.5, 0.9, 0.3));
                return sdf.result;
            }
        }
    }

    // Navigation button style
    NavButton = <Button> {
        width: 36, height: 36
        padding: 0
        draw_bg: {
            instance hover: 0.0
            instance pressed: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.circle(self.rect_size.x * 0.5, self.rect_size.y * 0.5, 16.0);
                let base = vec4(1.0, 1.0, 1.0, 0.1);
                let hover_color = vec4(1.0, 1.0, 1.0, 0.2);
                let pressed_color = vec4(1.0, 1.0, 1.0, 0.3);
                let color = mix(mix(base, hover_color, self.hover), pressed_color, self.pressed);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            text_style: { font_size: 16.0 }
            fn get_color(self) -> vec4 {
                return vec4(1.0, 1.0, 1.0, 0.9);
            }
        }
    }

    // Quick link card
    QuickLinkCard = <Button> {
        width: Fill, height: 56
        padding: 12
        margin: {bottom: 8}
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance pressed: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.0);
                let base = mix(
                    vec4(0.95, 0.96, 0.98, 1.0),
                    vec4(0.18, 0.20, 0.25, 1.0),
                    self.dark_mode
                );
                let hover_color = mix(
                    vec4(0.90, 0.92, 0.95, 1.0),
                    vec4(0.22, 0.24, 0.30, 1.0),
                    self.dark_mode
                );
                let color = mix(base, hover_color, self.hover);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 13.0 }
            fn get_color(self) -> vec4 {
                return mix(
                    vec4(0.2, 0.2, 0.25, 1.0),
                    vec4(0.9, 0.9, 0.95, 1.0),
                    self.dark_mode
                );
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

    pub WebViewDemoScreen = {{WebViewDemoScreen}} {
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

        // Header with gradient
        header = <GradientHeader> {
            flow: Right
            align: {y: 0.5}
            padding: {left: 16, right: 16}
            spacing: 12

            // Back button
            back_btn = <NavButton> {
                text: "<"
            }

            // Forward button
            forward_btn = <NavButton> {
                text: ">"
            }

            // Refresh button
            refresh_btn = <NavButton> {
                text: "R"
            }

            // URL bar
            url_bar = <UrlInput> {
                text: "https://example.com"
            }

            // Go button
            go_btn = <NavButton> {
                text: "Go"
                width: 48
            }
        }

        // Main content area
        content = <View> {
            width: Fill, height: Fill
            flow: Right

            // Sidebar with quick links
            sidebar = <View> {
                width: 220, height: Fill
                flow: Down
                padding: 16
                show_bg: true
                draw_bg: {
                    instance dark_mode: 0.0
                    fn pixel(self) -> vec4 {
                        return mix(
                            vec4(0.96, 0.97, 0.98, 1.0),
                            vec4(0.12, 0.13, 0.16, 1.0),
                            self.dark_mode
                        );
                    }
                }

                sidebar_title = <Label> {
                    text: "Quick Links"
                    margin: {bottom: 16}
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: { font_size: 14.0 }
                        fn get_color(self) -> vec4 {
                            return mix(
                                vec4(0.3, 0.3, 0.35, 1.0),
                                vec4(0.7, 0.7, 0.75, 1.0),
                                self.dark_mode
                            );
                        }
                    }
                }

                link_example = <QuickLinkCard> {
                    text: "Example.com"
                }

                link_github = <QuickLinkCard> {
                    text: "GitHub"
                }

                link_rust = <QuickLinkCard> {
                    text: "Rust Lang"
                }

                link_makepad = <QuickLinkCard> {
                    text: "Makepad"
                }

                <View> { width: Fill, height: Fill }  // Spacer

                // IPC Demo section
                ipc_section = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    spacing: 8
                    padding: {top: 16}

                    ipc_title = <Label> {
                        text: "IPC Demo"
                        margin: {bottom: 8}
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: { font_size: 14.0 }
                            fn get_color(self) -> vec4 {
                                return mix(
                                    vec4(0.3, 0.3, 0.35, 1.0),
                                    vec4(0.7, 0.7, 0.75, 1.0),
                                    self.dark_mode
                                );
                            }
                        }
                    }

                    send_msg_btn = <QuickLinkCard> {
                        text: "Send to WebView"
                    }

                    ipc_status = <Label> {
                        text: "Ready"
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: { font_size: 11.0 }
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

            // WebView container area
            webview_area = <View> {
                width: Fill, height: Fill
                flow: Down
                padding: 16

                // WebView wrapper with shadow
                webview_wrapper = <RoundedView> {
                    width: Fill, height: Fill
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: 8.0
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
                        url: "https://example.com"
                    }
                }
            }
        }

        // Status bar
        status_bar = <View> {
            width: Fill, height: 32
            flow: Right
            align: {y: 0.5}
            padding: {left: 16, right: 16}
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

            status_dot = <StatusDot> {}

            <View> { width: 8, height: 1 }  // Spacer

            status_text = <Label> {
                text: "Ready"
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
                text: "MoFA WebView Demo v0.1"
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
pub struct WebViewDemoScreen {
    #[deref]
    view: View,

    #[rust]
    current_url: String,
}

impl Widget for WebViewDemoScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Get actions from event
        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        };

        // Handle WebView events
        for action in actions {
            match action.as_widget_action().cast() {
                WebViewAction::Initialized => {
                    self.set_status(cx, "WebView initialized", 1.0);
                }
                WebViewAction::InitFailed(err) => {
                    self.set_status(cx, &format!("Failed: {}", err), 0.0);
                }
                WebViewAction::IpcMessage { channel, data } => {
                    let display = if data.len() > 30 {
                        format!("{}...", &data[..30])
                    } else {
                        data.clone()
                    };
                    self.view
                        .label(ids!(content.sidebar.ipc_section.ipc_status))
                        .set_text(cx, &format!("[{}] {}", channel, display));
                }
                WebViewAction::UrlChanged(_) | WebViewAction::None => {}
            }
        }

        // Handle button clicks
        if self.view.button(ids!(header.go_btn)).clicked(actions) {
            self.navigate_to_url(cx);
        }
        if self.view.button(ids!(header.refresh_btn)).clicked(actions) {
            self.refresh_page(cx);
        }

        // Quick links
        if self
            .view
            .button(ids!(content.sidebar.link_example))
            .clicked(actions)
        {
            self.load_url(cx, "https://example.com");
        }
        if self
            .view
            .button(ids!(content.sidebar.link_github))
            .clicked(actions)
        {
            self.load_url(cx, "https://github.com");
        }
        if self
            .view
            .button(ids!(content.sidebar.link_rust))
            .clicked(actions)
        {
            self.load_url(cx, "https://www.rust-lang.org");
        }
        if self
            .view
            .button(ids!(content.sidebar.link_makepad))
            .clicked(actions)
        {
            self.load_url(cx, "https://makepad.dev");
        }

        // IPC demo
        if self
            .view
            .button(ids!(content.sidebar.ipc_section.send_msg_btn))
            .clicked(actions)
        {
            self.send_ipc_message(cx);
        }

        // Handle Enter key in URL bar - listen for TextInput Return action
        if self.view.text_input(ids!(header.url_bar)).returned(actions).is_some() {
            self.navigate_to_url(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WebViewDemoScreen {
    fn navigate_to_url(&mut self, cx: &mut Cx) {
        let url = self.view.text_input(ids!(header.url_bar)).text();
        self.load_url(cx, &url);
    }

    fn load_url(&mut self, cx: &mut Cx, url: &str) {
        // Add https:// if missing
        let full_url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("https://{}", url)
        };

        self.current_url = full_url.clone();
        self.view
            .text_input(ids!(header.url_bar))
            .set_text(cx, &full_url);

        // Load in WebView
        let webview = self
            .view
            .web_view_container(ids!(content.webview_area.webview_wrapper.webview));
        if let Err(e) = webview.load_url(&full_url) {
            self.set_status(cx, &format!("Error: {}", e), 0.0);
        } else {
            self.set_status(cx, &format!("Loading {}", full_url), 2.0);
        }
    }

    fn refresh_page(&mut self, cx: &mut Cx) {
        if !self.current_url.is_empty() {
            let url = self.current_url.clone();
            self.load_url(cx, &url);
        }
    }

    fn send_ipc_message(&mut self, cx: &mut Cx) {
        let webview = self
            .view
            .web_view_container(ids!(content.webview_area.webview_wrapper.webview));
        let msg = r#"{"greeting": "Hello from Makepad!", "time": "now"}"#;
        if let Err(e) = webview.send_to_js("demo", msg) {
            self.view
                .label(ids!(content.sidebar.ipc_section.ipc_status))
                .set_text(cx, &format!("Send failed: {}", e));
        } else {
            self.view
                .label(ids!(content.sidebar.ipc_section.ipc_status))
                .set_text(cx, "Message sent!");
        }
        self.view.redraw(cx);
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

impl WebViewDemoScreenRef {
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            // Main background
            inner.view.apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );

            // Header
            inner.view.view(ids!(header)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );

            // URL bar
            inner.view.text_input(ids!(header.url_bar)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                    draw_text: { dark_mode: (dark_mode) }
                },
            );

            // Sidebar
            inner.view.view(ids!(content.sidebar)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );
            inner
                .view
                .label(ids!(content.sidebar.sidebar_title))
                .apply_over(
                    cx,
                    live! {
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );

            // Quick link cards
            inner
                .view
                .button(ids!(content.sidebar.link_example))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );
            inner
                .view
                .button(ids!(content.sidebar.link_github))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );
            inner
                .view
                .button(ids!(content.sidebar.link_rust))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );
            inner
                .view
                .button(ids!(content.sidebar.link_makepad))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );
            inner
                .view
                .button(ids!(content.sidebar.ipc_section.send_msg_btn))
                .apply_over(
                    cx,
                    live! {
                        draw_bg: { dark_mode: (dark_mode) }
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );

            // IPC section labels
            inner
                .view
                .label(ids!(content.sidebar.ipc_section.ipc_title))
                .apply_over(
                    cx,
                    live! {
                        draw_text: { dark_mode: (dark_mode) }
                    },
                );
            inner
                .view
                .label(ids!(content.sidebar.ipc_section.ipc_status))
                .apply_over(
                    cx,
                    live! {
                        draw_text: { dark_mode: (dark_mode) }
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
