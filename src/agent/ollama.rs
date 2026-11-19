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
//!   with an empty prompt, `stream: false` and a `keep_alive` window — no
//!   tokens are generated. If that endpoint is unavailable, a minimal
//!   `<base>/chat/completions` call (no `keep_alive`) is used as a fallback.

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

/// Whether `base` is a bare Ollama loopback root: host `127.0.0.1`,
/// `localhost` or `::1`, port `11434`, and no path/query/fragment.
fn is_ollama_loopback_root(base: &str) -> bool {
    let after_scheme = base
        .strip_prefix("http://")
        .or_else(|| base.strip_prefix("https://"))
        .unwrap_or(base);
    // A bare root has no path, query, or fragment after the authority.
    if after_scheme
        .find(['/', '?', '#'])
        .is_some_and(|i| i < after_scheme.len())
    {
        return false;
    }
    let (host, port) = after_scheme.rsplit_once(':').unwrap_or((after_scheme, ""));
    let host = host.trim_start_matches('[').trim_end_matches(']');
    port == "11434" && matches!(host, "127.0.0.1" | "localhost" | "::1")
}

/// Normalise a user-supplied Ollama base URL.
///
/// Trims surrounding whitespace and trailing slashes. A bare Ollama loopback
/// root gets the OpenAI-compatible `/v1` suffix appended; every other URL is
/// returned unchanged apart from the trimming. The Settings page calls this
/// before storing a provider's base URL.
pub fn normalize_base_url(url: &str) -> String {
    let base = trim(url);
    if is_ollama_loopback_root(base) {
        format!("{base}/v1")
    } else {
        base.to_owned()
    }
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

/// Body for Ollama's native load-only call: an empty prompt and a closed
/// stream load the model and refresh its warm window without generating any
/// tokens.
fn native_load_body(model: &str) -> Value {
    serde_json::json!({
        "model": model,
        "prompt": "",
        "stream": false,
        "keep_alive": KEEP_ALIVE,
    })
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
/// a minimal OpenAI-compatible chat call is used instead — that body contains
/// no Ollama-specific fields, so strict servers accept it. Intended to run on
/// a background thread — a cold model can take a while to load.
pub fn keep_loaded(base_url: &str, model: &str) -> Result<(), AgentError> {
    let agent = ureq::builder().timeout(Duration::from_secs(120)).build();
    if let Some(native) = native_generate_url(base_url) {
        let body = native_load_body(model).to_string();
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
    })
    .to_string();
    agent
        .post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body)
        .map(|_| ())
        .map_err(|e| describe(e, &url))
}

/// Whether a provider points at a local Ollama-style endpoint.
///
/// Used only by the warm-model UI path — chat bodies are always strict
/// OpenAI-compatible JSON. Requires a `Local` provider whose URL is either a
/// loopback Ollama endpoint on port 11434 or has a hostname mentioning
/// "ollama". Hosted gateways are never considered Ollama.
pub fn is_ollama(kind: ProviderKind, base_url: &str) -> bool {
    if kind != ProviderKind::Local {
        return false;
    }
    let base = trim(base_url);
    let after_scheme = base
        .strip_prefix("http://")
        .or_else(|| base.strip_prefix("https://"))
        .unwrap_or(base);
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
    let (host, port) = authority.rsplit_once(':').unwrap_or((authority, ""));
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let loopback = port == "11434" && matches!(host, "127.0.0.1" | "localhost" | "::1");
    loopback || host.to_lowercase().contains("ollama")
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
        assert!(is_ollama(ProviderKind::Local, "http://127.0.0.1:11434/v1"));
        assert!(is_ollama(ProviderKind::Local, "https://localhost:11434"));
        assert!(is_ollama(ProviderKind::Local, "http://[::1]:11434/v1"));
        assert!(is_ollama(
            ProviderKind::Local,
            "http://my-ollama-host:8080/v1"
        ));
        assert!(!is_ollama(ProviderKind::Local, ""));
        assert!(!is_ollama(ProviderKind::Local, "http://127.0.0.1:8080/v1"));
    }

    #[test]
    fn is_ollama_does_not_match_hosted_gateways() {
        assert!(!is_ollama(
            ProviderKind::OpenRouter,
            "https://openrouter.ai/api/v1"
        ));
        assert!(!is_ollama(
            ProviderKind::Groq,
            "https://api.groq.com/openai/v1"
        ));
        // A hosted gateway is never Ollama even if its hostname mentions it,
        // and a non-Local provider pointing at a loopback URL is not either.
        assert!(!is_ollama(
            ProviderKind::OpenAICompatible,
            "https://ollama.example.com/v1"
        ));
        assert!(!is_ollama(
            ProviderKind::OpenAICompatible,
            "http://127.0.0.1:11434/v1"
        ));
    }

    #[test]
    fn native_warm_body_loads_without_generating() {
        let body = native_load_body("llama3.1");
        assert_eq!(body.get("prompt").and_then(Value::as_str), Some(""));
        assert_eq!(body.get("stream").and_then(Value::as_bool), Some(false));
        assert_eq!(
            body.get("keep_alive").and_then(Value::as_str),
            Some(KEEP_ALIVE)
        );
        assert_eq!(body.get("model").and_then(Value::as_str), Some("llama3.1"));
    }

    #[test]
    fn normalize_base_url_adds_v1_for_ollama_root() {
        assert_eq!(
            normalize_base_url("http://127.0.0.1:11434"),
            "http://127.0.0.1:11434/v1"
        );
        assert_eq!(
            normalize_base_url(" https://localhost:11434/ "),
            "https://localhost:11434/v1"
        );
        assert_eq!(
            normalize_base_url("http://[::1]:11434"),
            "http://[::1]:11434/v1"
        );
    }

    #[test]
    fn normalize_base_url_leaves_other_urls_alone() {
        assert_eq!(
            normalize_base_url("http://127.0.0.1:11434/v1"),
            "http://127.0.0.1:11434/v1"
        );
        assert_eq!(
            normalize_base_url("https://openrouter.ai/api/v1"),
            "https://openrouter.ai/api/v1"
        );
        assert_eq!(
            normalize_base_url("http://127.0.0.1:8080"),
            "http://127.0.0.1:8080"
        );
    }
}
