//! Settings › Agent: the agent brain — chat with it, see what it does,
//! approve what it wants to run, and configure the provider chain.

use eframe::egui::{self, RichText};
use parking_lot::Mutex;

use super::{PageCtx, SettingsApp, theme};
use crate::agent::config::{AgentProviderConfig, ApprovalMode, ProviderKind};
use crate::agent::{self, AgentEvent, Status};

/// Input box text, kept in a static so it survives the frame loop.
static AGENT_INPUT: Mutex<String> = Mutex::new(String::new());

impl SettingsApp {
    pub(super) fn tab_agent(ui: &mut egui::Ui, cx: &mut PageCtx<'_>) {
        let host = match agent::global() {
            Some(host) => host,
            None => {
                ui.label("The agent brain is not running.");
                return;
            }
        };
        let events = host.drain_events();

        // Intro.
        ui.label(
            RichText::new(
                "The agent brain reads the screen, sees the clipboard, and acts on \
                 the desktop — clicking, typing, opening apps. You decide what it \
                 may run; mutations ask for approval unless you turn off asking.",
            )
            .size(11.0)
            .color(theme::text_secondary()),
        );
        ui.add_space(8.0);

        // Status.
        let status_text = match host.status() {
            Status::Idle => "Idle — send a message below.".to_owned(),
            Status::Thinking => "Thinking…".to_owned(),
            Status::Approval(tool) => format!("{tool} wants to run — approve it below."),
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new("Status").strong());
            ui.label(RichText::new(&status_text).color(theme::text_secondary()));
        });

        // Pending approval.
        if let Some(p) = host.pending_approval() {
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!("Approve {}: {}", p.tool, p.description))
                    .strong()
                    .size(12.0),
            );
            ui.horizontal(|ui| {
                if ui.button("Approve").clicked() {
                    host.set_approval(true);
                }
                if ui.button("Deny").clicked() {
                    host.set_approval(false);
                }
            });
        }

        // Conversation.
        ui.add_space(10.0);
        ui.separator();
        ui.label(RichText::new("Conversation").strong());
        for event in events {
            ui.add_space(2.0);
            match event {
                AgentEvent::UserMessage(text) => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("You").color(theme::accent()));
                        ui.label(RichText::new(&text).color(theme::text_secondary()));
                    });
                }
                AgentEvent::Answer(text) => {
                    ui.label(RichText::new(&text).color(theme::text_secondary()));
                }
                AgentEvent::ToolCall { tool, description, .. } => {
                    ui.label(
                        RichText::new(format!("→ {description} ({tool})"))
                            .size(11.0)
                            .color(theme::text_secondary()),
                    );
                }
                AgentEvent::ToolResult { tool, ok, output } => {
                    let mut line = format!("← {tool}: {output}");
                    if !ok {
                        line.insert_str(0, "failed: ");
                    }
                    ui.label(RichText::new(line).size(11.0).color(theme::text_secondary()));
                }
                AgentEvent::ToolSkipped { tool } => {
                    ui.label(RichText::new(format!("✕ {tool} — denied")).size(11.0).color(theme::text_secondary()));
                }
                AgentEvent::ApprovalRequired { .. } => {
                    // Shown by the approval panel above.
                }
                AgentEvent::Error(error) => {
                    ui.label(
                        RichText::new(format!("Error: {error}"))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(244, 160, 92)),
                    );
                }
            }
        }

        // Input + send.
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let mut input = AGENT_INPUT.lock().clone();
            let response = ui
                .add(egui::TextEdit::singleline(&mut input).hint_text("Message the agent…"));
            let entered = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let send = ui.button("Send");
            let can_send = !input.trim().is_empty();
            if (send.clicked() || (entered && response.lost_focus() && can_send)) && can_send {
                host.send(std::mem::take(&mut input));
            }
            *AGENT_INPUT.lock() = input;
        });

        // Provider chain.
        ui.add_space(10.0);
        ui.separator();
        ui.label(RichText::new("Provider chain").strong());
        ui.label(
            RichText::new(
                "Providers are tried in order; a provider that runs out (rate limit, \
                 quota, 5xx) is marked down and the next one takes over.",
            )
            .size(11.0)
            .color(theme::text_secondary()),
        );
        let mut remove_at: Option<usize> = None;
        for (index, provider) in cx.cfg.agent.chain.iter_mut().enumerate() {
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(format!("{}.", index + 1)).strong());
                ui.add(egui::TextEdit::singleline(&mut provider.name).desired_width(90.0));
                egui::ComboBox::from_id_source(index)
                    .selected_text(provider.kind.label())
                    .show_ui(ui, |ui| {
                        for kind in [ProviderKind::Local, ProviderKind::OpenRouter, ProviderKind::Groq, ProviderKind::OpenAICompatible] {
                            if ui.selectable_label(kind == provider.kind, kind.label()).clicked() {
                                provider.kind = kind;
                                if kind != ProviderKind::OpenAICompatible && provider.base_url.is_empty() {
                                    provider.base_url = kind.default_base_url();
                                }
                                if kind == ProviderKind::OpenAICompatible {
                                    provider.base_url.clear();
                                }
                                *cx.changed = true;
                            }
                        }
                    });
                ui.add(
                    egui::TextEdit::singleline(&mut provider.base_url)
                        .desired_width(140.0)
                        .hint_text("Base URL"),
                );
                let mut key_env = provider.api_key_env.clone().unwrap_or_default();
                ui.add(
                    egui::TextEdit::singleline(&mut key_env)
                        .desired_width(90.0)
                        .hint_text("Key env var"),
                );
                provider.api_key_env = if key_env.is_empty() { None } else { Some(key_env) };
                ui.add(
                    egui::TextEdit::singleline(&mut provider.model)
                        .desired_width(90.0)
                        .hint_text("Model"),
                );
                ui.checkbox(&mut provider.decision_model, "Decision model");
                ui.checkbox(&mut provider.vision, "Vision");
                if ui.button("Remove").clicked() {
                    remove_at = Some(index);
                }
            });
        }
        if let Some(index) = remove_at {
            cx.cfg.agent.chain.remove(index);
            *cx.changed = true;
        }
        if ui.button("Add provider").clicked() {
            let next = cx.cfg.agent.chain.len() + 1;
            cx.cfg.agent.chain.push(AgentProviderConfig::new(
                format!("provider {next}"),
                ProviderKind::Local,
                "",
            ));
            *cx.changed = true;
        }
        *cx.changed = true;

        // Switch and approval mode.
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut cx.cfg.agent.enabled, "Agent enabled");
            ui.label(
                RichText::new("Approvals")
                    .size(11.0)
                    .color(theme::text_secondary()),
            );
            for mode in [ApprovalMode::Ask, ApprovalMode::Autonomous] {
                if ui.selectable_value(&mut cx.cfg.agent.approval_mode, mode, mode.label()).clicked() {
                    *cx.changed = true;
                }
            }
        });
    }
}
