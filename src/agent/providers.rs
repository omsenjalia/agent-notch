//! Provider chain: OpenAI-compatible HTTP clients with automatic failover.
//!
//! Providers are tried in chain order. A provider that fails with a
//! failover-class error (rate limit, quota, 5xx, network) is marked
//! unhealthy and the next provider runs — that is the "one runs out,
//! switch to another" behaviour. A 401/403 marks the provider dead until
//! it recovers or the chain is rebuilt.
//!
//! All clients are OpenAI-compatible servers (Ollama, Groq, OpenRouter,
//! LM Studio, llama.cpp...); the brain talks to the
//! `<base_url>/chat/completions` endpoint.

use std::time::{Duration, Instant};

use serde_json::Value;
use ureq::Error as UreqError;

use crate::agent::config::AgentProviderConfig;
use crate::agent::desktop;
use crate::agent::message::{ChatRequest, ChatResponse, messages_to_openai, ToolCall};

/// How a provider failure should be treated by the chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    /// The provider refused or could not handle the request — try the next.
    Failover,
    /// The provider is unusable as configured (bad key) — treat as down.
    Fatal,
}

/// A provider-call failure, classified for the chain.
#[derive(Clone, Debug)]
pub struct AgentError {
    pub kind: FailureKind,
    pub message: String,
}

impl AgentError {
    pub fn failover(message: String) -> Self {
        Self { kind: FailureKind::Failover, message }
    }

    pub fn fatal(message: String) -> Self {
        Self { kind: FailureKind::Fatal, message }
    }

    fn from_ureq(error: UreqError) -> Self {
        match &error {
            UreqError::Status(code, _) => {
                let kind = match code {
                    401 | 403 => FailureKind::Fatal,
                    _ => FailureKind::Failover,
                };
                AgentError { kind, message: format!("HTTP {code}: {error}") }
            }
            UreqError::Transport(_) => AgentError::failover(error.to_string()),
        }
    }
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AgentError {}

/// A chat provider. [`HttpProvider`] is the only implementation.
pub trait ChatProvider {
    fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, AgentError>;
}

/// One chain entry: the config plus its health. The health is behind a
/// mutex because the chain's `chat` is `&self` — it reports results from
/// callers that only hold a shared reference.
pub struct ProviderEntry {
    pub config: AgentProviderConfig,
    pub health: parking_lot::Mutex<Health>,
}

/// Health of one provider.
#[derive(Clone, Debug)]
pub struct Health {
    pub healthy: bool,
    pub down_until: Option<Instant>,
    pub last_error: Option<AgentError>,
    errors: u32,
}

impl Health {
    pub fn new() -> Self {
        Self { healthy: true, down_until: None, last_error: None, errors: 0 }
    }

    pub fn note_ok(&mut self) {
        self.healthy = true;
        self.down_until = None;
        self.errors = 0;
        self.last_error = None;
    }

    pub fn note_err(&mut self, error: &AgentError) {
        self.errors = self.errors.saturating_add(1);
        match error.kind {
            FailureKind::Fatal => {
                self.healthy = false;
                self.down_until = Some(Instant::now() + Duration::from_secs(90));
            }
            FailureKind::Failover => {
                self.healthy = false;
                let backoff = Duration::from_secs(2 + 10 * self.errors.min(3) as u64);
                self.down_until = Some(Instant::now() + backoff);
            }
        }
        self.last_error = Some(error.clone());
    }

    /// Whether the chain may use this provider right now: healthy and not
    /// in a backoff window.
    pub fn available(&self) -> bool {
        if let Some(until) = self.down_until {
            if Instant::now() < until {
                return false;
            }
        }
        self.healthy
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new()
    }
}

/// The ordered provider chain with failover.
pub struct ProviderChain {
    entries: Vec<ProviderEntry>,
}

impl ProviderChain {
    pub fn new(configs: Vec<AgentProviderConfig>) -> Self {
        Self {
            entries: configs
                .into_iter()
                .map(|config| ProviderEntry {
                    config,
                    health: parking_lot::Mutex::new(Health::new()),
                })
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether any entry in the chain is usable right now.
    pub fn available(&self) -> bool {
        self.entries.iter().any(|entry| entry.health.lock().available())
    }

    /// Whether any entry accepts screenshots as vision input.
    pub fn any_vision(&self) -> bool {
        self.entries.iter().any(|entry| entry.config.vision)
    }

    /// Index of the provider to ask first: the decision model if flagged and
    /// available, else the first available entry.
    pub fn decision_index(&self) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.config.decision_model && entry.health.lock().available())
            .or_else(|| self.entries.iter().position(|entry| entry.health.lock().available()))
    }

    pub fn config(&self, index: usize) -> &AgentProviderConfig {
        &self.entries[index].config
    }

    /// Try each provider in order until one answers. Failover-class errors
    /// mark the provider unhealthy and the next one is tried; a fatal error
    /// stops the chain (the provider is marked down).
    pub fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, AgentError> {
        let mut last: Option<AgentError> = None;
        for entry in &self.entries {
            if !entry.health.lock().available() {
                continue;
            }
            let provider = HttpProvider::new(&entry.config);
            match provider.chat(request) {
                Ok(response) => {
                    entry.health.lock().note_ok();
                    return Ok(response);
                }
                Err(error) => {
                    entry.health.lock().note_err(&error);
                    last = Some(error);
                    if matches!(last.as_ref().unwrap().kind, FailureKind::Fatal) {
                        break;
                    }
                }
            }
        }
        Err(last
            .unwrap_or_else(|| AgentError::failover("no healthy providers".to_owned())))
    }

}

/// An OpenAI-compatible HTTP provider: `<base_url>/chat/completions`.
pub struct HttpProvider {
    config: AgentProviderConfig,
}

impl HttpProvider {
    pub fn new(config: &AgentProviderConfig) -> Self {
        Self { config: config.clone() }
    }
}

impl ChatProvider for HttpProvider {
    fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, AgentError> {
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        let mut body = serde_json::Map::new();
        body.insert("model".into(), Value::String(self.config.model.clone()));
        body.insert(
            "messages".into(),
            serde_json::to_value(messages_to_openai(&request.messages))
                .map_err(|e| AgentError::failover(format!("serialize messages: {e}")))?,
        );
        body.insert("tools".into(), desktop::tool_schemas());
        body.insert("stream".into(), Value::Bool(false));
        let body = Value::Object(body).to_string();

        let mut req = ureq::post(&url).set("Content-Type", "application/json");
        if let Some(key) = self.config.api_key() {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }
        let response = req.send_string(&body).map_err(AgentError::from_ureq)?;
        let text = response.into_string().map_err(|e| AgentError::failover(e.to_string()))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| AgentError::failover(format!("invalid JSON from {url}: {e}")))?;
        parse_chat_response(&value)
    }
}

fn parse_chat_response(value: &Value) -> Result<ChatResponse, AgentError> {
    let message = value
        .pointer("/choices/0/message")
        .ok_or_else(|| AgentError::failover("response has no choices[0].message".to_owned()))?;
    let content = match message.get("content") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Array(parts)) => Some(
            parts
                .iter()
                .filter_map(|p| p.get("text").and_then(Value::as_str).map(String::from))
                .collect(),
        ),
        _ => None,
    };
    let tool_calls = message
        .get("tool_calls")
        .and_then(Value::as_array)
        .map(|calls| {
            calls
                .iter()
                .filter_map(|c| {
                    let function = c.get("function")?;
                    Some(ToolCall {
                        id: c.get("id").and_then(Value::as_str).unwrap_or("").to_owned(),
                        name: function.get("name").and_then(Value::as_str).unwrap_or("").to_owned(),
                        arguments: function
                            .get("arguments")
                            .and_then(Value::as_str)
                            .unwrap_or("{}")
                            .to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(ChatResponse { content, tool_calls })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::config::{AgentProviderConfig, ProviderKind};

    fn chain_of(kinds: &[ProviderKind]) -> ProviderChain {
        let configs = kinds
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                let mut config = AgentProviderConfig::new(format!("p{i}"), *kind, "m");
                config.decision_model = true;
                config
            })
            .collect();
        ProviderChain::new(configs)
    }

    #[test]
    fn health_starts_available() {
        assert!(Health::new().available());
    }

    #[test]
    fn failover_error_marks_unavailable_and_next_chain_member_stays_available() {
        let chain = chain_of(&[ProviderKind::Local, ProviderKind::Groq]);
        chain.entries[0].health.lock().note_err(&AgentError::failover("HTTP 429".into()));
        assert!(!chain.entries[0].health.lock().available());
        assert!(chain.entries[1].health.lock().available());
        assert_eq!(chain.decision_index(), Some(1));
    }

    #[test]
    fn fatal_error_marks_provider_down() {
        let chain = chain_of(&[ProviderKind::Local]);
        chain.entries[0].health.lock().note_err(&AgentError::fatal("HTTP 401".into()));
        assert!(!chain.entries[0].health.lock().available());
        assert!(!chain.available());
        assert_eq!(chain.decision_index(), None);
    }

    #[test]
    fn backoff_expires_and_provider_becomes_available_again() {
        let mut health = Health::new();
        health.note_err(&AgentError::failover("HTTP 500".into()));
        assert!(!health.available());
        health.down_until = Some(Instant::now() - Duration::from_millis(1));
        assert!(health.available());
    }

    #[test]
    fn success_resets_health() {
        let mut health = Health::new();
        health.note_err(&AgentError::failover("HTTP 429".into()));
        assert!(!health.available());
        health.note_ok();
        assert!(health.available());
        assert!(health.last_error.is_none());
    }

    #[test]
    fn decision_index_prefers_flagged_provider() {
        let mut chain = chain_of(&[ProviderKind::Local, ProviderKind::Groq]);
        chain.entries[0].config.decision_model = false;
        chain.entries[1].config.decision_model = true;
        assert_eq!(chain.decision_index(), Some(1));
    }
}
