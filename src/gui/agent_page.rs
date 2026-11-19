//! Settings › Agent: a short, guided setup and a chat — with the dense
//! provider-chain editor tucked away under "Advanced".

use std::sync::mpsc::{self, Receiver};

use eframe::egui::{self, RichText};
use parking_lot::Mutex;

use super::{theme, PageCtx, SettingsApp};
use crate::agent::config::{AgentProviderConfig, ApprovalMode, ProviderKind};
use crate::agent::{self, ollama, AgentEvent, Status};

/// Chat input, kept in a static so it survives the frame loop.
static AGENT_INPUT: Mutex<String> = Mutex::new(String::new());

/// Ollama discovery is done on background threads so a slow or absent server
/// never freezes the Settings window. The UI polls these receivers each frame.
struct OllamaUi {
    models: Vec<String>,
    loading: bool,
    error: Option<String>,
    /// Model currently being loaded into memory.
    warming: Option<String>,
    /// Result of the last load, shown as one quiet line.
    warm_note: Option<String>,
    fetch_rx: Option<Receiver<Result<Vec<String>, String>>>,
    warm_rx: Option<Receiver<Result<String, String>>>,
    /// Base URL the automatic fetch was last attempted for (prevents a loop
    /// when Ollama is offline).
    tried: String,
}

impl OllamaUi {
    const fn new() -> Self {
        Self {
            models: Vec::new(),
            loading: false,
            error: None,
            warming: None,
            warm_note: None,
            fetch_rx: None,
            warm_rx: None,
            tried: String::new(),
        }
    }
}

static OLLAMA: Mutex<OllamaUi> = Mutex::new(OllamaUi::new());

fn refresh_models(base_url: &str) {
    let (tx, rx) = mpsc::channel();
    {
        let mut state = OLLAMA.lock();
        state.loading = true;
        state.error = None;
        state.tried = base_url.to_owned();
        state.fetch_rx = Some(rx);
    }
    let base = base_url.to_owned();
    let _ = std::thread::Builder::new()
        .name("agent-notch ollama models".to_owned())
        .spawn(move || {
            let result = ollama::fetch_models(&base)
                .map_err(|e| e.message)
                .and_then(|models| {
                    if models.is_empty() {
                        Err("Ollama replied, but no models were found. Pull one first \
                             (for example: ollama pull llama3.2)."
                            .to_owned())
                    } else {
                        Ok(models)
                    }
                });
            let _ = tx.send(result);
        });
}

fn warm_model(base_url: &str, model: &str) {
    let (tx, rx) = mpsc::channel();
    {
        let mut state = OLLAMA.lock();
        state.warming = Some(model.to_owned());
        state.warm_note = None;
        state.warm_rx = Some(rx);
    }
    let (base, model) = (base_url.to_owned(), model.to_owned());
    let _ = std::thread::Builder::new()
        .name("agent-notch ollama load".to_owned())
        .spawn(move || {
            let result = ollama::keep_loaded(&base, &model)
                .map(|_| model.clone())
                .map_err(|e| e.message);
            let _ = tx.send(result);
        });
}

/// Poll both background jobs. Called once per frame, before the UI is built.
fn poll_ollama() {
    let mut state = OLLAMA.lock();
    if let Some(rx) = &state.fetch_rx {
        if let Ok(result) = rx.try_recv() {
            match result {
                Ok(models) => state.models = models,
                Err(error) => {
                    state.models.clear();
                    state.error = Some(error);
                }
            }
            state.loading = false;
            state.fetch_rx = None;
        }
    }
    if let Some(rx) = &state.warm_rx {
        if let Ok(result) = rx.try_recv() {
            state.warming = None;
            state.warm_rx = None;
            state.warm_note = Some(match result {
                Ok(model) => format!("{model} is loaded and kept warm for 10 minutes."),
                Err(error) => format!("Could not warm the model: {error}"),
            });
        }
    }
}

/// A short labelled line of helper copy.
fn hint(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        RichText::new(text.into())
            .size(11.0)
            .color(theme::text_secondary()),
    );
}

fn event_line(event: &AgentEvent) -> Option<(&'static str, String, egui::Color32)> {
    let (who, text, color) = match event {
        AgentEvent::UserMessage(text) => ("You", text.clone(), theme::accent()),
        AgentEvent::Answer(text) => ("Agent", text.clone(), theme::text_primary()),
        AgentEvent::ToolCall {
            tool, description, ..
        } => (
            "Tool",
            format!("→ {description} ({tool})"),
            theme::text_secondary(),
        ),
        AgentEvent::ToolResult { tool, ok, output } => (
            "Tool",
            format!("← {tool}: {output}{}", if *ok { "" } else { " (failed)" }),
            theme::text_secondary(),
        ),
        AgentEvent::ToolSkipped { tool } => {
            ("Tool", format!("✕ {tool} — denied"), theme::text_tertiary())
        }
        AgentEvent::ApprovalRequired { tool, .. } => (
            "Approval",
            format!("{tool} wants to run"),
            egui::Color32::from_rgb(244, 190, 92),
        ),
        AgentEvent::Error(error) => (
            "Error",
            error.clone(),
            egui::Color32::from_rgb(244, 160, 92),
        ),
    };
    Some((who, text, color))
}

impl SettingsApp {
    pub(super) fn tab_agent(ui: &mut egui::Ui, cx: &mut PageCtx<'_>) {
        let host = match agent::global() {
            Some(host) => host,
            None => {
                ui.label("The agent brain is not running.");
                return;
            }
        };
        poll_ollama();
        let transcript = host.transcript();
        let mut edited = false;

        ui.label(
            RichText::new(
                "Chat with the agent here, or from the Agent slide in the notch. \
                 Set up a local model in two clicks below — no API key needed.",
            )
            .size(11.0)
            .color(theme::text_secondary()),
        );
        ui.add_space(8.0);

        // -- Status ---------------------------------------------------------
        let status_text = match host.status() {
            Status::Idle => "Idle — send a message below.".to_owned(),
            Status::Thinking => "Thinking…".to_owned(),
            Status::Approval(tool) => format!("{tool} wants to run — approve it below."),
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new("Status").strong());
            ui.label(RichText::new(&status_text).color(theme::text_secondary()));
        });

        // -- Pending approval ------------------------------------------------
        if let Some(p) = host.pending_approval() {
            ui.add_space(8.0);
            egui::Frame::none()
                .fill(theme::surface())
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("Approve “{}”?\n{}", p.tool, p.description))
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
                });
        }

        // -- Local model setup ----------------------------------------------
        ui.add_space(12.0);
        ui.separator();
        ui.label(RichText::new("Local model (Ollama)").strong());
        hint(
            ui,
            "The models already on your machine are listed automatically.",
        );

        let local_kind = ProviderKind::Local;
        let local_index = cx.cfg.agent.chain.iter().position(|p| p.kind == local_kind);
        if local_index.is_none() {
            if ui.button("Add local Ollama provider").clicked() {
                let mut provider = AgentProviderConfig::new("ollama", local_kind, "");
                provider.decision_model = true;
                cx.cfg.agent.chain.insert(0, provider);
                edited = true;
            }
            ui.add_space(4.0);
        }

        if let Some(index) = local_index {
            let provider = &mut cx.cfg.agent.chain[index];
            if provider.base_url.is_empty() {
                provider.base_url = ollama::DEFAULT_BASE_URL.to_owned();
                edited = true;
            }
            let base_url = provider.base_url.clone();

            // Auto-fetch once per base URL when this page is first opened.
            {
                let state = OLLAMA.lock();
                if state.fetch_rx.is_none()
                    && !state.loading
                    && state.tried != base_url
                    && state.models.is_empty()
                {
                    drop(state);
                    refresh_models(&base_url);
                }
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Base URL");
                let mut url = base_url.clone();
                if ui
                    .add(egui::TextEdit::singleline(&mut url).desired_width(240.0))
                    .changed()
                {
                    provider.base_url = url;
                    edited = true;
                }
                if ui.button("Refresh models").clicked() {
                    refresh_models(&base_url);
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Model");
                let mut selected = if ollama::is_ollama(provider.kind, &provider.base_url)
                    && !provider.model.is_empty()
                {
                    provider.model.clone()
                } else {
                    String::new()
                };
                let combo = egui::ComboBox::from_id_source("ollama-model-picker")
                    .selected_text(if selected.is_empty() {
                        "Choose a model…".to_owned()
                    } else {
                        selected.clone()
                    })
                    .width(230.0);
                combo.show_ui(ui, |ui| {
                    if selected.is_empty() {
                        ui.label("— select —");
                    }
                    let state = OLLAMA.lock();
                    for model in &state.models {
                        if ui.selectable_label(selected == *model, model).clicked() {
                            selected = model.clone();
                        }
                    }
                });
                if !selected.is_empty() && selected != provider.model {
                    provider.model = selected.clone();
                    provider.decision_model = true;
                    cx.cfg.agent.enabled = true;
                    edited = true;
                    warm_model(&base_url, &selected);
                }
                if ui.button("Reload model").clicked() && !provider.model.is_empty() {
                    warm_model(&base_url, &provider.model);
                }
            });

            {
                let state = OLLAMA.lock();
                if state.loading {
                    hint(ui, "Looking for Ollama…");
                } else if let Some(error) = &state.error {
                    ui.label(
                        RichText::new(error)
                            .size(11.0)
                            .color(egui::Color32::from_rgb(244, 160, 92)),
                    );
                } else if let Some(warming) = &state.warming {
                    hint(ui, format!("Loading {warming} into memory…"));
                } else if let Some(note) = &state.warm_note {
                    hint(ui, note);
                } else if state.models.is_empty() {
                    hint(
                        ui,
                        "No models yet — press Refresh models after starting Ollama.",
                    );
                } else {
                    hint(
                        ui,
                        format!("{} models found on this machine.", state.models.len()),
                    );
                }
            }
        }

        ui.add_space(4.0);
        if ui
            .checkbox(&mut cx.cfg.agent.enabled, "Agent enabled")
            .changed()
        {
            edited = true;
        }

        // -- Conversation ----------------------------------------------------
        ui.add_space(12.0);
        ui.separator();
        ui.label(RichText::new("Conversation").strong());
        egui::ScrollArea::vertical()
            .max_height(190.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if transcript.is_empty() {
                    hint(
                        ui,
                        "Nothing yet. Try: “what app has focus?” or “take a screenshot”.",
                    );
                }
                for event in &transcript {
                    if let Some((who, text, color)) = event_line(event) {
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(who).small().strong().color(color));
                            ui.label(RichText::new(text).small());
                        });
                    }
                }
            });

        // -- Input -----------------------------------------------------------
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let mut input = AGENT_INPUT.lock().clone();
            let response = ui.add(
                egui::TextEdit::multiline(&mut input)
                    .min_size(egui::vec2(0.0, 46.0))
                    .hint_text("Message the agent… (Enter sends, Shift+Enter newline)"),
            );
            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
            let send = ui.button("Send");
            let can_send = !input.trim().is_empty();
            if (send.clicked() || (enter && response.has_focus())) && can_send {
                host.send(std::mem::take(&mut input));
            }
            *AGENT_INPUT.lock() = input;
        });

        // -- Advanced --------------------------------------------------------
        ui.add_space(12.0);
        ui.collapsing("Advanced — hosted providers & controls", |ui| {
            ui.label(
                RichText::new(
                    "Providers are tried in order; one that runs out (rate limit, quota, 5xx) \
                     is marked down and the next takes over. API keys are read from the \
                     environment variable you name, never stored in config.json.",
                )
                .size(11.0)
                .color(theme::text_secondary()),
            );

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Approvals")
                        .size(11.0)
                        .color(theme::text_secondary()),
                );
                for mode in [ApprovalMode::Ask, ApprovalMode::Autonomous] {
                    if ui
                        .selectable_value(&mut cx.cfg.agent.approval_mode, mode, mode.label())
                        .clicked()
                    {
                        edited = true;
                    }
                }
            });

            ui.add_space(8.0);
            let mut remove_at: Option<usize> = None;
            for (index, provider) in cx.cfg.agent.chain.iter_mut().enumerate() {
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(format!("{}.", index + 1)).strong());
                    if ui
                        .add(egui::TextEdit::singleline(&mut provider.name).desired_width(80.0))
                        .changed()
                    {
                        edited = true;
                    }
                    let mut kind = provider.kind;
                    egui::ComboBox::from_id_source(index)
                        .selected_text(provider.kind.label())
                        .show_ui(ui, |ui| {
                            for option in [
                                ProviderKind::Local,
                                ProviderKind::OpenRouter,
                                ProviderKind::Groq,
                                ProviderKind::OpenAICompatible,
                            ] {
                                if ui
                                    .selectable_label(option == kind, option.label())
                                    .clicked()
                                {
                                    kind = option;
                                    if option == ProviderKind::Local {
                                        provider.base_url = option.default_base_url();
                                    } else if option != ProviderKind::OpenAICompatible
                                        && provider.base_url.is_empty()
                                    {
                                        provider.base_url = option.default_base_url();
                                    }
                                    if option == ProviderKind::OpenAICompatible {
                                        provider.base_url.clear();
                                    }
                                    provider.kind = option;
                                    edited = true;
                                }
                            }
                        });
                    for field in [&mut provider.base_url, &mut provider.model] {
                        if ui
                            .add(egui::TextEdit::singleline(field).desired_width(120.0))
                            .changed()
                        {
                            edited = true;
                        }
                    }
                    let mut key_env = provider.api_key_env.clone().unwrap_or_default();
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut key_env)
                                .desired_width(90.0)
                                .hint_text("Key env var"),
                        )
                        .changed()
                    {
                        provider.api_key_env = if key_env.is_empty() {
                            None
                        } else {
                            Some(key_env)
                        };
                        edited = true;
                    }
                    if ui
                        .checkbox(&mut provider.decision_model, "Decision")
                        .changed()
                    {
                        edited = true;
                    }
                    if ui.checkbox(&mut provider.vision, "Vision").changed() {
                        edited = true;
                    }
                    if ui.button("Remove").clicked() {
                        remove_at = Some(index);
                    }
                });
            }
            if let Some(index) = remove_at {
                cx.cfg.agent.chain.remove(index);
                edited = true;
            }
            if ui.button("Add provider").clicked() {
                let next = cx.cfg.agent.chain.len() + 1;
                cx.cfg.agent.chain.push(AgentProviderConfig::new(
                    format!("provider {next}"),
                    local_kind,
                    "",
                ));
                edited = true;
            }
        });

        if edited {
            *cx.changed = true;
            host.reconfigure(&cx.cfg.agent);
        }
    }
}
