//! Ollama discovery + model warming.
//!
//! The Settings › Agent page uses this to list the models Ollama already has
//! on disk (no more typing model names by hand) and to load the chosen model
//! into memory with a `keep_alive` window, so the next prompt is instant.
//!
//! Everything talks to the OpenAI-compatible base URL the user configured —
//! `http://127.0.0.1:11434/v1` by default:
//!
//! * `GET  <base>/models` lists the models.
//! * Ollama's native `/api/generate` (the `/v1` suffix removed) loads a model
//!   with no prompt and a `keep_alive` window — no tokens are generated. If
//!   that endpoint is unavailable, `<base>/chat/completions` with `keep_alive`
//!   is used as a fallback for other OpenAI-compatible servers.

use std::time::Duration;

use serde_json::Value;
use ureq::Error as UreqError;

use crate::agent::config::ProviderKind;
use crate::agent::providers::AgentError;

/// How long a selected model stays resident after the last call.
pub const KEEP_ALIVE: &str = "10m";

/// The default OpenAI-compatible endpoint Ollama exposes locally.
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11434/v1";

fn trim(base_url: &str) -> &str {
    base_url.trim().trim_end_matches('/')
}

/// `<base>/models`, the OpenAI-compatible model list.
pub fn models_url(base_url: &str) -> String {
    format!("{}/models", trim(base_url))
}

/// `<base>/chat/completions`, the OpenAI-compatible chat endpoint.
pub fn chat_url(base_url: &str) -> String {
    format!("{}/chat/completions", trim(base_url))
}

/// Ollama's native endpoint, for the no-output load fallback.
/// `http://127.0.0.1:11434/v1` -> `http://127.0.0.1:11434/api/generate`.
fn native_generate_url(base_url: &str) -> Option<String> {
    let base = trim(base_url);
    let host = base.strip_suffix("/v1").unwrap_or(base);
    (host.ends_with("/api/generate"))
        .then(|| host.to_owned())
        .or_else(|| (!host.is_empty()).then(|| format!("{host}/api/generate")))
}

fn describe(error: UreqError, url: &str) -> AgentError {
    match error {
        UreqError::Status(code, _) => AgentError::failover(format!(
            "Ollama returned HTTP {code} for {url}. Is Ollama running with model support?"
        )),
        UreqError::Transport(e) => AgentError::failover(format!(
            "can't reach Ollama at {url}: {e}. Start Ollama, then try again."
        )),
    }
}

/// Read the model ids out of an OpenAI-compatible `/models` response.
fn parse_models(value: &Value) -> Vec<String> {
    let mut ids: Vec<String> = value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    row.get("id")
                        .or_else(|| row.get("name"))
                        .and_then(Value::as_str)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    ids.sort();
    ids.dedup();
    ids
}

/// Fetch the models Ollama currently has available.
pub fn fetch_models(base_url: &str) -> Result<Vec<String>, AgentError> {
    let url = models_url(base_url);
    let agent = ureq::builder().timeout(Duration::from_secs(4)).build();
    let body = agent
        .get(&url)
        .call()
        .map_err(|e| describe(e, &url))?
        .into_string()
        .map_err(|e| AgentError::failover(format!("reading Ollama response: {e}")))?;
    let value: Value = serde_json::from_str(&body)
        .map_err(|e| AgentError::failover(format!("Ollama returned invalid JSON: {e}")))?;
    Ok(parse_models(&value))
}

/// Load `model` and keep it warm for [`KEEP_ALIVE`].
///
/// Ollama's native load-only call is tried first: it keeps the model resident
/// without generating a single token. If the server has no native endpoint,
/// the OpenAI-compatible chat call with `keep_alive` is used instead. Intended
/// to run on a background thread — a cold model can take a while to load.
pub fn keep_loaded(base_url: &str, model: &str) -> Result<(), AgentError> {
    let agent = ureq::builder().timeout(Duration::from_secs(120)).build();
    if let Some(native) = native_generate_url(base_url) {
        let body = serde_json::json!({
            "model": model,
            "keep_alive": KEEP_ALIVE,
        })
        .to_string();
        let load = agent
            .post(&native)
            .set("Content-Type", "application/json")
            .send_string(&body);
        if load.is_ok() {
            return Ok(());
        }
    }

    let url = chat_url(base_url);
    let body = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": "hi" }],
        "max_tokens": 1,
        "stream": false,
        "keep_alive": KEEP_ALIVE,
    })
    .to_string();
    agent
        .post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body)
        .map(|_| ())
        .map_err(|e| describe(e, &url))
}

/// Whether a provider points at a local Ollama-style endpoint. Used by the
/// chat path to attach `keep_alive` to every request, so the warm window is
/// refreshed whenever the brain actually uses the model.
pub fn is_ollama(kind: ProviderKind, base_url: &str) -> bool {
    kind == ProviderKind::Local || base_url.contains("11434") || base_url.contains("ollama")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_normalise_trailing_slashes() {
        assert_eq!(
            models_url("http://127.0.0.1:11434/v1/"),
            "http://127.0.0.1:11434/v1/models"
        );
        assert_eq!(chat_url("http://x/v1///"), "http://x/v1/chat/completions");
    }

    #[test]
    fn native_url_drops_the_v1_suffix() {
        assert_eq!(
            native_generate_url(DEFAULT_BASE_URL).as_deref(),
            Some("http://127.0.0.1:11434/api/generate")
        );
        assert_eq!(
            native_generate_url("http://localhost:11434").as_deref(),
            Some("http://localhost:11434/api/generate")
        );
    }

    #[test]
    fn parse_models_reads_both_openai_and_ollama_shapes() {
        let value: Value = serde_json::from_str(
            r#"{"object":"list","data":[{"id":"qwen2.5:latest"},{"name":"llama3.1:latest"},{"id":"qwen2.5:latest"}]}"#,
        )
        .unwrap();
        assert_eq!(
            parse_models(&value),
            vec!["llama3.1:latest".to_owned(), "qwen2.5:latest".to_owned()]
        );
    }

    #[test]
    fn is_ollama_detects_local_endpoints() {
        assert!(is_ollama(ProviderKind::Local, ""));
        assert!(is_ollama(
            ProviderKind::OpenAICompatible,
            "http://127.0.0.1:11434/v1"
        ));
        assert!(!is_ollama(
            ProviderKind::OpenRouter,
            "https://openrouter.ai/api/v1"
        ));
    }
}
