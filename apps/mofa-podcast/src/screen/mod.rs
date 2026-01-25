//! Podcast Generator Screen
//!
//! Makepad native UI for podcast generation

use makepad_widgets::*;
use crate::models::{PodcastScript, AudioSettings};
use crate::services::{parser, generator::AudioGenerator};
use std::collections::HashMap;
use std::path::PathBuf;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;

    // Voice dropdown values
    VoiceTingTing = LiveId,
    VoiceMeiJia = LiveId,
    VoiceSinji = LiveId,
    VoiceSamantha = LiveId,
    VoiceAlex = LiveId,
    VoiceDaniel = LiveId,

    // Panel with subtle border
    PanelBg = <RoundedView> {
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            instance radius: 6.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.radius);
                let bg = mix(
                    vec4(0.98, 0.98, 0.99, 1.0),
                    vec4(0.16, 0.17, 0.20, 1.0),
                    self.dark_mode
                );
                sdf.fill(bg);
                // Border
                sdf.stroke(mix(
                    vec4(0.88, 0.89, 0.90, 1.0),
                    vec4(0.25, 0.26, 0.30, 1.0),
                    self.dark_mode
                ), 1.0);
                return sdf.result;
            }
        }
    }

    // Section title
    SectionTitle = <Label> {
        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 11.0 }
            fn get_color(self) -> vec4 {
                return mix(
                    vec4(0.35, 0.35, 0.40, 1.0),
                    vec4(0.55, 0.55, 0.60, 1.0),
                    self.dark_mode
                );
            }
        }
    }

    // Primary button
    PrimaryButton = <Button> {
        width: Fill, height: 36
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance pressed: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let base = mix(
                    vec4(0.22, 0.45, 0.78, 1.0),
                    vec4(0.30, 0.52, 0.85, 1.0),
                    self.dark_mode
                );
                let color = mix(base, vec4(0.28, 0.52, 0.88, 1.0), self.hover);
                let color = mix(color, vec4(0.18, 0.40, 0.70, 1.0), self.pressed);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            text_style: { font_size: 12.0 }
            fn get_color(self) -> vec4 {
                return vec4(1.0, 1.0, 1.0, 1.0);
            }
        }
    }

    // Secondary button
    SecondaryButton = <Button> {
        width: Fit, height: 28
        padding: {left: 12, right: 12}
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            instance pressed: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let base = mix(
                    vec4(0.92, 0.93, 0.94, 1.0),
                    vec4(0.25, 0.26, 0.30, 1.0),
                    self.dark_mode
                );
                let color = mix(base, mix(vec4(0.88, 0.89, 0.90, 1.0), vec4(0.30, 0.31, 0.35, 1.0), self.dark_mode), self.hover);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 11.0 }
            fn get_color(self) -> vec4 {
                return mix(
                    vec4(0.25, 0.25, 0.30, 1.0),
                    vec4(0.85, 0.85, 0.90, 1.0),
                    self.dark_mode
                );
            }
        }
    }

    // Danger button (for clear)
    DangerButton = <Button> {
        width: Fit, height: 28
        padding: {left: 12, right: 12}
        draw_bg: {
            instance dark_mode: 0.0
            instance hover: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let base = mix(
                    vec4(0.95, 0.92, 0.92, 1.0),
                    vec4(0.30, 0.22, 0.22, 1.0),
                    self.dark_mode
                );
                let color = mix(base, mix(vec4(0.98, 0.88, 0.88, 1.0), vec4(0.38, 0.25, 0.25, 1.0), self.dark_mode), self.hover);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 11.0 }
            fn get_color(self) -> vec4 {
                return mix(
                    vec4(0.75, 0.25, 0.25, 1.0),
                    vec4(0.95, 0.55, 0.55, 1.0),
                    self.dark_mode
                );
            }
        }
    }

    // Voice select dropdown
    VoiceDropdown = <DropDown> {
        width: Fill, height: 32
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                let bg = mix(
                    vec4(0.96, 0.96, 0.97, 1.0),
                    vec4(0.20, 0.21, 0.24, 1.0),
                    self.dark_mode
                );
                sdf.fill(bg);
                sdf.stroke(mix(
                    vec4(0.85, 0.86, 0.88, 1.0),
                    vec4(0.30, 0.31, 0.35, 1.0),
                    self.dark_mode
                ), 1.0);
                return sdf.result;
            }
        }
        draw_text: {
            instance dark_mode: 0.0
            text_style: { font_size: 11.0 }
            fn get_color(self) -> vec4 {
                return mix(
                    vec4(0.20, 0.20, 0.25, 1.0),
                    vec4(0.88, 0.88, 0.92, 1.0),
                    self.dark_mode
                );
            }
        }
        labels: ["Ting-Ting (Chinese)", "Mei-Jia (Chinese)", "Sin-ji (Cantonese)", "Samantha (English)", "Alex (English)", "Daniel (British)"]
        values: [VoiceTingTing, VoiceMeiJia, VoiceSinji, VoiceSamantha, VoiceAlex, VoiceDaniel]
    }

    pub PodcastScreen = {{PodcastScreen}} {
        width: Fill, height: Fill
        flow: Right
        padding: 0
        spacing: 0

        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix(
                    vec4(0.94, 0.95, 0.96, 1.0),
                    vec4(0.11, 0.12, 0.14, 1.0),
                    self.dark_mode
                );
            }
        }

        // Left: Script editor (larger)
        editor_section = <View> {
            width: Fill, height: Fill
            flow: Down
            padding: 16
            spacing: 12

            // Toolbar row
            toolbar = <View> {
                width: Fill, height: Fit
                flow: Right
                spacing: 8
                align: {y: 0.5}

                <SectionTitle> {
                    text: "SCRIPT"
                }

                <View> { width: Fill, height: 1 }

                import_btn = <SecondaryButton> {
                    text: "Import File"
                }

                clear_btn = <DangerButton> {
                    text: "Clear"
                }
            }

            // Editor panel
            editor_panel = <PanelBg> {
                width: Fill, height: Fill
                padding: 12

                script_input = <TextInput> {
                    width: Fill, height: Fill
                    text: ""

                    draw_bg: {
                        instance dark_mode: 0.0
                        fn pixel(self) -> vec4 {
                            return mix(
                                vec4(0.99, 0.99, 1.0, 1.0),
                                vec4(0.14, 0.15, 0.18, 1.0),
                                self.dark_mode
                            );
                        }
                    }

                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: { font_size: 12.0 }
                        fn get_color(self) -> vec4 {
                            return mix(
                                vec4(0.15, 0.15, 0.20, 1.0),
                                vec4(0.88, 0.88, 0.92, 1.0),
                                self.dark_mode
                            );
                        }
                    }
                }
            }
        }

        // Right: Config panel (fixed width)
        config_section = <View> {
            width: 260, height: Fill
            flow: Down
            padding: {top: 16, right: 16, bottom: 16, left: 0}
            spacing: 12

            // Voice Config header
            <View> {
                width: Fill, height: Fit
                flow: Right
                align: {y: 0.5}

                <SectionTitle> {
                    text: "VOICE CONFIG"
                }

                <View> { width: Fill, height: 1 }

                status_label = <Label> {
                    text: "Ready"
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: { font_size: 10.0 }
                        fn get_color(self) -> vec4 {
                            return mix(
                                vec4(0.45, 0.65, 0.45, 1.0),
                                vec4(0.55, 0.80, 0.55, 1.0),
                                self.dark_mode
                            );
                        }
                    }
                }
            }

            // Config panel
            config_panel = <PanelBg> {
                width: Fill, height: Fill
                flow: Down
                padding: 12
                spacing: 12

                // Role sections (hidden by default)
                role_section_1 = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    spacing: 4
                    visible: false

                    role_1_label = <Label> {
                        text: "Role 1"
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: { font_size: 11.0 }
                            fn get_color(self) -> vec4 {
                                return mix(
                                    vec4(0.25, 0.25, 0.30, 1.0),
                                    vec4(0.75, 0.75, 0.80, 1.0),
                                    self.dark_mode
                                );
                            }
                        }
                    }

                    role_1_voice = <VoiceDropdown> {}
                }

                role_section_2 = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    spacing: 4
                    visible: false

                    role_2_label = <Label> {
                        text: "Role 2"
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: { font_size: 11.0 }
                            fn get_color(self) -> vec4 {
                                return mix(
                                    vec4(0.25, 0.25, 0.30, 1.0),
                                    vec4(0.75, 0.75, 0.80, 1.0),
                                    self.dark_mode
                                );
                            }
                        }
                    }

                    role_2_voice = <VoiceDropdown> {}
                }

                role_section_3 = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    spacing: 4
                    visible: false

                    role_3_label = <Label> {
                        text: "Role 3"
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: { font_size: 11.0 }
                            fn get_color(self) -> vec4 {
                                return mix(
                                    vec4(0.25, 0.25, 0.30, 1.0),
                                    vec4(0.75, 0.75, 0.80, 1.0),
                                    self.dark_mode
                                );
                            }
                        }
                    }

                    role_3_voice = <VoiceDropdown> {}
                }

                // Info text
                info_label = <Label> {
                    width: Fill
                    text: "Paste script or click Import to detect roles automatically"
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: { font_size: 10.0 }
                        fn get_color(self) -> vec4 {
                            return mix(
                                vec4(0.50, 0.50, 0.55, 1.0),
                                vec4(0.50, 0.50, 0.55, 1.0),
                                self.dark_mode
                            );
                        }
                    }
                }

                // Spacer
                <View> { width: Fill, height: Fill }

                // Output info
                output_label = <Label> {
                    width: Fill
                    text: ""
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: { font_size: 10.0 }
                        fn get_color(self) -> vec4 {
                            return mix(
                                vec4(0.30, 0.55, 0.30, 1.0),
                                vec4(0.45, 0.75, 0.45, 1.0),
                                self.dark_mode
                            );
                        }
                    }
                }

                // Generate button
                generate_btn = <PrimaryButton> {
                    text: "Generate Audio"
                }
            }
        }
    }
}

const VOICE_IDS: &[&str] = &["Ting-Ting", "Mei-Jia", "Sin-ji", "Samantha", "Alex", "Daniel"];

#[derive(Live, LiveHook, Widget)]
pub struct PodcastScreen {
    #[deref]
    view: View,

    #[rust]
    detected_roles: Vec<String>,

    #[rust]
    role_voice_mapping: HashMap<String, String>,

    #[rust]
    script: Option<PodcastScript>,
}

impl Widget for PodcastScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // Import button
        if self.view.button(ids!(editor_section.toolbar.import_btn)).clicked(actions) {
            self.import_script(cx);
        }

        // Clear button
        if self.view.button(ids!(editor_section.toolbar.clear_btn)).clicked(actions) {
            self.clear_all(cx);
        }

        // Generate button
        if self.view.button(ids!(config_section.config_panel.generate_btn)).clicked(actions) {
            self.generate_audio(cx);
        }

        // Handle dropdown changes
        for i in 0..3 {
            let dropdown_id = match i {
                0 => ids!(config_section.config_panel.role_section_1.role_1_voice),
                1 => ids!(config_section.config_panel.role_section_2.role_2_voice),
                _ => ids!(config_section.config_panel.role_section_3.role_3_voice),
            };

            if let Some(selected) = self.view.drop_down(dropdown_id).selected(actions) {
                if i < self.detected_roles.len() {
                    let role = &self.detected_roles[i];
                    let voice_id = VOICE_IDS.get(selected).unwrap_or(&"Ting-Ting");
                    self.role_voice_mapping.insert(role.clone(), voice_id.to_string());
                    ::log::info!("Assigned voice {} to role {}", voice_id, role);
                }
            }
        }

        // Check for text changes to detect roles
        if self.view.text_input(ids!(editor_section.editor_panel.script_input)).changed(actions).is_some() {
            self.parse_script_content(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl PodcastScreen {
    fn import_script(&mut self, cx: &mut Cx) {
        ::log::info!("Import button clicked");

        let file_dialog = rfd::FileDialog::new()
            .add_filter("Script files", &["md", "txt", "json"])
            .add_filter("All files", &["*"])
            .set_title("Select script file");

        if let Some(file_path) = file_dialog.pick_file() {
            ::log::info!("File selected: {:?}", file_path);

            match std::fs::read_to_string(&file_path) {
                Ok(content) => {
                    self.view.text_input(ids!(editor_section.editor_panel.script_input)).set_text(cx, &content);
                    self.parse_script_content(cx);
                    self.set_status(cx, &format!("Loaded: {}", file_path.file_name().unwrap_or_default().to_string_lossy()));
                }
                Err(e) => {
                    self.set_status(cx, &format!("Error: {}", e));
                }
            }
        }
    }

    fn parse_script_content(&mut self, cx: &mut Cx) {
        let content = self.view.text_input(ids!(editor_section.editor_panel.script_input)).text();

        if content.trim().is_empty() {
            self.detected_roles.clear();
            self.update_role_ui(cx);
            return;
        }

        match parser::parse_content(&content) {
            Ok(script) => {
                self.detected_roles = script.roles.iter().map(|r| r.name.clone()).collect();
                self.script = Some(script);

                // Set default voice assignments
                for (i, role) in self.detected_roles.iter().enumerate() {
                    let default_voice = VOICE_IDS.get(i % VOICE_IDS.len()).unwrap_or(&"Ting-Ting");
                    self.role_voice_mapping.insert(role.clone(), default_voice.to_string());
                }

                self.update_role_ui(cx);

                if !self.detected_roles.is_empty() {
                    self.set_status(cx, &format!("{} roles found", self.detected_roles.len()));
                }
            }
            Err(e) => {
                ::log::error!("Parse error: {}", e);
            }
        }
    }

    fn update_role_ui(&mut self, cx: &mut Cx) {
        let role_sections = [
            (ids!(config_section.config_panel.role_section_1), ids!(config_section.config_panel.role_section_1.role_1_label), ids!(config_section.config_panel.role_section_1.role_1_voice)),
            (ids!(config_section.config_panel.role_section_2), ids!(config_section.config_panel.role_section_2.role_2_label), ids!(config_section.config_panel.role_section_2.role_2_voice)),
            (ids!(config_section.config_panel.role_section_3), ids!(config_section.config_panel.role_section_3.role_3_label), ids!(config_section.config_panel.role_section_3.role_3_voice)),
        ];

        for (i, (section_id, label_id, dropdown_id)) in role_sections.iter().enumerate() {
            if i < self.detected_roles.len() {
                self.view.view(*section_id).set_visible(cx, true);
                self.view.label(*label_id).set_text(cx, &self.detected_roles[i]);

                // Set default selection
                if let Some(voice) = self.role_voice_mapping.get(&self.detected_roles[i]) {
                    if let Some(idx) = VOICE_IDS.iter().position(|v| *v == voice) {
                        self.view.drop_down(*dropdown_id).set_selected_item(cx, idx);
                    }
                }
            } else {
                self.view.view(*section_id).set_visible(cx, false);
            }
        }

        // Update info label
        if self.detected_roles.is_empty() {
            self.view.label(ids!(config_section.config_panel.info_label))
                .set_text(cx, "Paste script or click Import to detect roles automatically");
        } else if self.detected_roles.len() > 3 {
            self.view.label(ids!(config_section.config_panel.info_label))
                .set_text(cx, &format!("{} roles detected (showing first 3)", self.detected_roles.len()));
        } else {
            self.view.label(ids!(config_section.config_panel.info_label))
                .set_text(cx, "Select a voice for each role");
        }

        self.view.redraw(cx);
    }

    fn clear_all(&mut self, cx: &mut Cx) {
        self.view.text_input(ids!(editor_section.editor_panel.script_input)).set_text(cx, "");
        self.detected_roles.clear();
        self.role_voice_mapping.clear();
        self.script = None;
        self.update_role_ui(cx);
        self.set_status(cx, "Ready");
        self.view.label(ids!(config_section.config_panel.output_label)).set_text(cx, "");
    }

    fn generate_audio(&mut self, cx: &mut Cx) {
        ::log::info!("Generate button clicked");

        let content = self.view.text_input(ids!(editor_section.editor_panel.script_input)).text();

        if content.trim().is_empty() {
            self.set_status(cx, "No script");
            return;
        }

        if self.detected_roles.is_empty() {
            self.set_status(cx, "No roles");
            return;
        }

        // Check voice assignments
        for role in &self.detected_roles {
            if !self.role_voice_mapping.contains_key(role) {
                self.set_status(cx, "Missing voice");
                return;
            }
        }

        self.set_status(cx, "Generating...");

        // Get output directory
        let output_dir = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("MoFaPodcast");

        match AudioGenerator::new(output_dir) {
            Ok(generator) => {
                if let Some(ref script) = self.script {
                    let settings = AudioSettings::default();

                    match generator.generate(script, &self.role_voice_mapping, &settings, None) {
                        Ok(output_path) => {
                            self.set_status(cx, "Complete!");
                            let filename = output_path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            self.view.label(ids!(config_section.config_panel.output_label))
                                .set_text(cx, &format!("Saved: {}", filename));
                            ::log::info!("Audio generated: {:?}", output_path);
                        }
                        Err(e) => {
                            self.set_status(cx, "Error");
                            self.view.label(ids!(config_section.config_panel.output_label))
                                .set_text(cx, &format!("{}", e));
                            ::log::error!("Generation failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                self.set_status(cx, "Error");
                self.view.label(ids!(config_section.config_panel.output_label))
                    .set_text(cx, &format!("{}", e));
            }
        }
    }

    fn set_status(&mut self, cx: &mut Cx, text: &str) {
        self.view.label(ids!(config_section.status_label)).set_text(cx, text);
        self.view.redraw(cx);
    }
}

impl PodcastScreenRef {
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Editor section
            inner.view.view(ids!(editor_section.editor_panel)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.text_input(ids!(editor_section.editor_panel.script_input)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });

            // Config section
            inner.view.view(ids!(config_section.config_panel)).apply_over(cx, live! {
                draw_bg: { dark_mode: (dark_mode) }
            });

            inner.view.redraw(cx);
        }
    }
}
