//! Models View - Local model management panel

use makepad_widgets::*;
use std::path::PathBuf;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;

    // Model category header
    ModelCategoryHeader = <View> {
        width: Fill, height: Fit
        padding: {top: 16, bottom: 8, left: 0, right: 0}

        <Label> {
            draw_text: {
                text_style: <FONT_BOLD>{ font_size: 11.0 }
                color: (SLATE_500)
            }
        }
    }

    // Model item row
    ModelItem = <View> {
        width: Fill, height: Fit
        padding: {top: 12, bottom: 12, left: 16, right: 16}
        margin: {bottom: 4}
        flow: Right
        align: {x: 0.0, y: 0.5}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                let light = #F8FAFC;  // slate-50
                let dark = #1E293B;   // slate-800
                return mix(light, dark, self.dark_mode);
            }
        }

        // Model info (left side)
        info = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 4

            name_label = <Label> {
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_REGULAR>{ font_size: 13.0 }
                    fn get_color(self) -> vec4 {
                        return mix(#334155, #E2E8F0, self.dark_mode);
                    }
                }
            }

            desc_label = <Label> {
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                    fn get_color(self) -> vec4 {
                        return mix(#64748B, #94A3B8, self.dark_mode);
                    }
                }
            }
        }

        // Size label
        size_label = <Label> {
            width: 80, height: Fit
            align: {x: 1.0, y: 0.5}
            margin: {right: 12}
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
                fn get_color(self) -> vec4 {
                    return mix(#64748B, #94A3B8, self.dark_mode);
                }
            }
        }

        // Action button
        action_btn = <Button> {
            width: 80, height: 32
            text: "Download"
            draw_text: {
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
                fn get_color(self) -> vec4 {
                    return #FFFFFF;
                }
            }
            draw_bg: {
                instance downloaded: 0.0
                instance downloading: 0.0

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    // Blue for download, green for installed, gray for downloading
                    let download_color = #3B82F6;
                    let installed_color = #10B981;
                    let downloading_color = #6B7280;

                    let color = mix(
                        mix(download_color, downloading_color, self.downloading),
                        installed_color,
                        self.downloaded
                    );

                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 4.0);
                    sdf.fill(color);
                    return sdf.result;
                }
            }
        }
    }

    // Storage info footer
    StorageInfo = <View> {
        width: Fill, height: Fit
        padding: {top: 16, bottom: 16, left: 0, right: 0}
        margin: {top: 16}
        flow: Down
        spacing: 8
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                let light = #F1F5F9;  // slate-100
                let dark = #0F172A;   // slate-900
                return mix(light, dark, self.dark_mode);
            }
        }

        path_label = <Label> {
            padding: {left: 16}
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
                fn get_color(self) -> vec4 {
                    return mix(#64748B, #94A3B8, self.dark_mode);
                }
            }
            text: "Storage: ~/.dora/models"
        }

        usage_label = <Label> {
            padding: {left: 16}
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_REGULAR>{ font_size: 11.0 }
                fn get_color(self) -> vec4 {
                    return mix(#64748B, #94A3B8, self.dark_mode);
                }
            }
            text: "Used: Calculating..."
        }
    }

    // Main models view
    pub ModelsView = {{ModelsView}} {
        width: Fill, height: Fill
        flow: Down
        padding: {top: 24, bottom: 24, left: 24, right: 24}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((WHITE), (SLATE_900), self.dark_mode);
            }
        }

        // Title
        title = <Label> {
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_BOLD>{ font_size: 18.0 }
                fn get_color(self) -> vec4 {
                    return mix(#1E293B, #F1F5F9, self.dark_mode);
                }
            }
            text: "Local Models"
        }

        subtitle = <Label> {
            margin: {top: 4, bottom: 16}
            draw_text: {
                instance dark_mode: 0.0
                text_style: <FONT_REGULAR>{ font_size: 12.0 }
                fn get_color(self) -> vec4 {
                    return mix(#64748B, #94A3B8, self.dark_mode);
                }
            }
            text: "Manage AI models for speech recognition and synthesis"
        }

        // Scrollable content
        <ScrollYView> {
            width: Fill, height: Fill
            flow: Down
            spacing: 0

            // ASR Section
            asr_header = <View> {
                width: Fill, height: Fit
                padding: {top: 16, bottom: 8}
                <Label> {
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: <FONT_BOLD>{ font_size: 11.0 }
                        fn get_color(self) -> vec4 {
                            return mix(#64748B, #94A3B8, self.dark_mode);
                        }
                    }
                    text: "SPEECH RECOGNITION (ASR)"
                }
            }

            whisper_item = <ModelItem> {
                info = {
                    name_label = { text: "Whisper Medium" }
                    desc_label = { text: "English speech recognition by OpenAI" }
                }
                size_label = { text: "~500 MB" }
                action_btn = { text: "Download" }
            }

            funasr_item = <ModelItem> {
                info = {
                    name_label = { text: "FunASR" }
                    desc_label = { text: "Chinese speech recognition by Alibaba" }
                }
                size_label = { text: "~500 MB" }
                action_btn = { text: "Download" }
            }

            // TTS Section
            tts_header = <View> {
                width: Fill, height: Fit
                padding: {top: 24, bottom: 8}
                <Label> {
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: <FONT_BOLD>{ font_size: 11.0 }
                        fn get_color(self) -> vec4 {
                            return mix(#64748B, #94A3B8, self.dark_mode);
                        }
                    }
                    text: "TEXT TO SPEECH (TTS)"
                }
            }

            kokoro_item = <ModelItem> {
                info = {
                    name_label = { text: "Kokoro-82M" }
                    desc_label = { text: "Fast and lightweight TTS" }
                }
                size_label = { text: "~400 MB" }
                action_btn = { text: "Download" }
            }

            primespeech_item = <ModelItem> {
                info = {
                    name_label = { text: "PrimeSpeech" }
                    desc_label = { text: "High-quality Chinese TTS with voice cloning" }
                }
                size_label = { text: "~1.3 GB" }
                action_btn = { text: "Download" }
            }

            g2pw_item = <ModelItem> {
                info = {
                    name_label = { text: "G2PW" }
                    desc_label = { text: "Chinese text-to-pinyin conversion" }
                }
                size_label = { text: "~600 MB" }
                action_btn = { text: "Download" }
            }

            // Storage info
            storage_info = <StorageInfo> {}
        }
    }
}

/// Model info structure
#[derive(Clone, Debug)]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub size: &'static str,
    pub downloaded: bool,
}

/// Available models
const MODELS: &[ModelInfo] = &[
    ModelInfo { id: "whisper", name: "Whisper Medium", description: "English ASR", size: "~500 MB", downloaded: false },
    ModelInfo { id: "funasr", name: "FunASR", description: "Chinese ASR", size: "~500 MB", downloaded: false },
    ModelInfo { id: "kokoro", name: "Kokoro-82M", description: "Fast TTS", size: "~400 MB", downloaded: false },
    ModelInfo { id: "primespeech", name: "PrimeSpeech", description: "Chinese TTS", size: "~1.3 GB", downloaded: false },
    ModelInfo { id: "g2pw", name: "G2PW", description: "Chinese G2P", size: "~600 MB", downloaded: false },
];

#[derive(Live, LiveHook, Widget)]
pub struct ModelsView {
    #[deref]
    view: View,

    #[rust]
    model_status: Vec<(String, bool)>,  // (model_id, is_downloaded)
}

impl Widget for ModelsView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // Handle download button clicks
        let model_buttons = [
            ("whisper", ids!(whisper_item.action_btn)),
            ("funasr", ids!(funasr_item.action_btn)),
            ("kokoro", ids!(kokoro_item.action_btn)),
            ("primespeech", ids!(primespeech_item.action_btn)),
            ("g2pw", ids!(g2pw_item.action_btn)),
        ];

        for (model_id, btn_path) in model_buttons.iter() {
            if self.view.button(btn_path.clone()).clicked(actions) {
                self.download_model(cx, model_id);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ModelsView {
    fn download_model(&mut self, cx: &mut Cx, model_id: &str) {
        // Update button to show downloading state
        let btn_path = match model_id {
            "whisper" => ids!(whisper_item.action_btn),
            "funasr" => ids!(funasr_item.action_btn),
            "kokoro" => ids!(kokoro_item.action_btn),
            "primespeech" => ids!(primespeech_item.action_btn),
            "g2pw" => ids!(g2pw_item.action_btn),
            _ => return,
        };

        self.view.button(btn_path.as_slice()).set_text(cx, "Downloading...");
        self.view.button(btn_path.as_slice()).apply_over(cx, live!{
            draw_bg: { downloading: 1.0 }
        });
        self.view.redraw(cx);

        // TODO: Actually call download_models.py in background
    }

    fn get_models_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".dora")
            .join("models")
    }

    fn check_model_status(&mut self) {
        let models_dir = Self::get_models_dir();

        self.model_status = MODELS.iter().map(|m| {
            let model_path = models_dir.join(m.id);
            (m.id.to_string(), model_path.exists())
        }).collect();
    }
}

impl ModelsViewRef {
    pub fn refresh(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.check_model_status();

            // Update UI based on status
            for (model_id, downloaded) in &inner.model_status {
                let btn_path = match model_id.as_str() {
                    "whisper" => ids!(whisper_item.action_btn),
                    "funasr" => ids!(funasr_item.action_btn),
                    "kokoro" => ids!(kokoro_item.action_btn),
                    "primespeech" => ids!(primespeech_item.action_btn),
                    "g2pw" => ids!(g2pw_item.action_btn),
                    _ => continue,
                };

                if *downloaded {
                    inner.view.button(btn_path.as_slice()).set_text(cx, "Installed");
                    inner.view.button(btn_path.as_slice()).apply_over(cx, live!{
                        draw_bg: { downloaded: 1.0, downloading: 0.0 }
                    });
                } else {
                    inner.view.button(btn_path.as_slice()).set_text(cx, "Download");
                    inner.view.button(btn_path.as_slice()).apply_over(cx, live!{
                        draw_bg: { downloaded: 0.0, downloading: 0.0 }
                    });
                }
            }

            // Update storage info
            let models_dir = ModelsView::get_models_dir();
            inner.view.label(ids!(storage_info.path_label))
                .set_text(cx, &format!("Storage: {}", models_dir.display()));

            inner.view.redraw(cx);
        }
    }

    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Update all labels
            inner.view.label(ids!(title)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(subtitle)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            inner.view.redraw(cx);
        }
    }
}
