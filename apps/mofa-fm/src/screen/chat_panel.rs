//! Chat panel methods for MoFaFMScreen
//!
//! Handles chat display, prompt input, and message formatting.

use makepad_widgets::*;

use super::{ChatMessageEntry, MoFaFMScreen};

impl MoFaFMScreen {
    /// Send prompt - TODO: connect to VoiceChatEngine when backend is ready
    pub(super) fn send_prompt(&mut self, cx: &mut Cx) {
        let input_text = self
            .view
            .text_input(ids!(
                left_column
                    .running_tab_content
                    .prompt_container
                    .prompt_section
                    .prompt_row
                    .prompt_input
            ))
            .text();
        // Use default prompt if input is empty
        let prompt_text = if input_text.is_empty() {
            "开始吧".to_string()
        } else {
            input_text
        };

        // TODO: Initialize VoiceChatEngine when backend is implemented
        // self.init_voice_chat(cx);

        // Add user message to chat
        let user_msg = ChatMessageEntry::new("You", prompt_text.clone());
        self.chat_messages.push(user_msg);
        // Keep chat messages bounded (prevents O(n²) slowdown and markdown overflow)
        if self.chat_messages.len() > 500 {
            self.chat_messages.remove(0);
        }
        self.update_chat_display(cx);

        // Clear input field
        self.view
            .text_input(ids!(
                left_column
                    .running_tab_content
                    .prompt_container
                    .prompt_section
                    .prompt_row
                    .prompt_input
            ))
            .set_text(cx, "");

        // TODO: Send through VoiceChatEngine when backend is ready
        // if let Some(ref voice_chat) = self.voice_chat_engine {
        //     if voice_chat.is_running() {
        //         voice_chat.send_prompt(&prompt_text);
        //         self.add_log(
        //             cx,
        //             &format!(
        //                 "[INFO] [App] Sent prompt: {}",
        //                 if prompt_text.len() > 50 {
        //                     format!("{}...", &prompt_text[..50])
        //                 } else {
        //                     prompt_text.to_string()
        //                 }
        //             ),
        //         );
        //     } else {
        //         self.add_log(
        //             cx,
        //             "[WARN] [App] VoiceChatEngine not running - prompt not sent",
        //         );
        //     }
        // }

        // Log locally for now
        self.add_log(
            cx,
            &format!(
                "[INFO] [App] Prompt entered: {}",
                if prompt_text.len() > 50 {
                    format!("{}...", &prompt_text[..50])
                } else {
                    prompt_text.to_string()
                }
            ),
        );

        self.view.redraw(cx);
    }

    /// Reset conversation - TODO: send reset to conference controller
    pub(super) fn reset_conversation(&mut self, cx: &mut Cx) {
        ::log::info!("Reset clicked");

        // TODO: Send reset command to conference controller via VoiceChatEngine
        // if let Some(ref voice_chat) = self.voice_chat_engine {
        //     if voice_chat.is_running() {
        //         voice_chat.send_control("reset");
        //         self.add_log(
        //             cx,
        //             "[INFO] [App] Sent reset command to conference controller",
        //         );
        //     } else {
        //         self.add_log(cx, "[WARN] [App] VoiceChatEngine not running - reset not sent");
        //     }
        // }

        // Clear chat messages
        self.chat_messages.clear();
        self.update_chat_display(cx);

        // Clear prompt input
        self.view
            .text_input(ids!(
                left_column
                    .running_tab_content
                    .prompt_container
                    .prompt_section
                    .prompt_row
                    .prompt_input
            ))
            .set_text(cx, "");

        // Reset audio player buffer
        if let Some(ref audio_player) = self.audio_player {
            audio_player.reset();
            self.add_log(cx, "[INFO] [App] Audio buffer reset");
        }

        self.view.redraw(cx);
    }

    /// Update chat display with current messages
    pub(super) fn update_chat_display(&mut self, cx: &mut Cx) {
        let chat_text = if self.chat_messages.is_empty() {
            "Waiting for conversation...".to_string()
        } else {
            self.chat_messages
                .iter()
                .map(|msg| {
                    let timestamp = Self::format_timestamp(msg.timestamp);
                    let streaming_indicator = if msg.is_streaming { " ⌛" } else { "" };
                    format!(
                        "**{}**{} ({}):  \n{}",
                        msg.sender, streaming_indicator, timestamp, msg.content
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n---\n\n")
        };

        ::log::debug!(
            "[Chat] update_display: text_len={}, messages={}",
            chat_text.len(),
            self.chat_messages.len()
        );

        self.view
            .markdown(ids!(
                left_column
                    .running_tab_content
                    .chat_container
                    .chat_section
                    .chat_scroll
                    .chat_content_wrapper
                    .chat_content
            ))
            .set_text(cx, &chat_text);

        // Auto-scroll to bottom when new messages arrive
        let chat_count = self.chat_messages.len();
        if chat_count > self.last_chat_count {
            self.view
                .view(ids!(
                    left_column
                        .running_tab_content
                        .chat_container
                        .chat_section
                        .chat_scroll
                ))
                .set_scroll_pos(cx, DVec2 { x: 0.0, y: 1e10 });
            self.last_chat_count = chat_count;
        }

        self.view.redraw(cx);
    }

    /// Format Unix timestamp (milliseconds) to readable HH:MM:SS format
    /// Matches conference-dashboard's get_timestamp() format
    pub(super) fn format_timestamp(timestamp_ms: u64) -> String {
        // Convert milliseconds to seconds
        let total_secs = timestamp_ms / 1000;
        // Get time of day (seconds since midnight UTC)
        let secs_in_day = total_secs % 86400;
        let hours = secs_in_day / 3600;
        let minutes = (secs_in_day % 3600) / 60;
        let seconds = secs_in_day % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}
