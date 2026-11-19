//! Agent brain configuration: the provider chain, keys and policies.
//!
//! API keys are read from environment variables at call time and are never
//! stored in config.json — `api_key_env` names the variable to read.

use serde::{Deserialize, Serialize};

/// A chat provider the agent brain can run on. `Local` talks to an
/// OpenAI-compatible server on localhost (Ollama by default); the others are
/// hosted OpenAI-compatible services.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    /// Any OpenAI-compatible server: Ollama by default.
    #[default]
    Local,
    OpenRouter,
    Groq,
    /// Any other OpenAI-compatible server (OpenAI, LM Studio, llama.cpp...).
    OpenAICompatible,
}

impl ProviderKind {
    pub fn default_base_url(self) -> String {
        match self {
            ProviderKind::Local => "http://127.0.0.1:11434/v1".to_owned(),
            ProviderKind::OpenRouter => "https://openrouter.ai/api/v1".to_owned(),
            ProviderKind::Groq => "https://api.groq.com/openai/v1".to_owned(),
            ProviderKind::OpenAICompatible => String::new(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProviderKind::Local => "Ollama (local)",
            ProviderKind::OpenRouter => "OpenRouter",
            ProviderKind::Groq => "Groq",
            ProviderKind::OpenAICompatible => "OpenAI-compatible",
        }
    }
}

/// One entry in the provider chain: an OpenAI-compatible server the agent
/// brain can talk to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProviderConfig {
    pub name: String,
    pub kind: ProviderKind,
    /// OpenAI-compatible base URL, e.g. `http://127.0.0.1:11434/v1`.
    pub base_url: String,
    /// Environment variable holding the API key, read per call.
    pub api_key_env: Option<String>,
    pub model: String,
    /// Flags the model that parses user requests into actions.
    #[serde(default)]
    pub decision_model: bool,
    /// The model accepts screenshots as vision input.
    #[serde(default)]
    pub vision: bool,
}

impl AgentProviderConfig {
    pub fn new(name: impl Into<String>, kind: ProviderKind, model: impl Into<String>) -> Self {
        let base_url = if kind == ProviderKind::OpenAICompatible {
            String::new()
        } else {
            kind.default_base_url()
        };
        Self {
            name: name.into(),
            kind,
            base_url,
            api_key_env: None,
            model: model.into(),
            decision_model: false,
            vision: false,
        }
    }

    /// The API key, if one is configured. Always read from the environment,
    /// so keys are never persisted in config.json.
    pub fn api_key(&self) -> Option<String> {
        self.api_key_env
            .as_deref()
            .and_then(|name| std::env::var(name).ok())
    }
}

impl Default for AgentProviderConfig {
    fn default() -> Self {
        Self::new("ollama", ProviderKind::Local, "llama3.1")
    }
}

/// What the brain asks before running a mutating tool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalMode {
    /// Mutations ask first; read-only tools run freely.
    #[default]
    Ask,
    /// Mutations run without asking — except irreversible ones.
    Autonomous,
}

impl ApprovalMode {
    pub fn label(self) -> &'static str {
        match self {
            ApprovalMode::Ask => "Ask before changes",
            ApprovalMode::Autonomous => "Autonomous (irreversible still asks)",
        }
    }
}

/// The agent brain: enabled, the ordered provider chain, and approval policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_chain")]
    pub chain: Vec<AgentProviderConfig>,
    #[serde(default)]
    pub approval_mode: ApprovalMode,
}

fn default_chain() -> Vec<AgentProviderConfig> {
    vec![AgentProviderConfig::default()]
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            chain: default_chain(),
            approval_mode: ApprovalMode::Ask,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_provider_defaults() {
        let p = AgentProviderConfig::default();
        assert_eq!(p.kind, ProviderKind::Local);
        assert_eq!(p.base_url, "http://127.0.0.1:11434/v1");
        assert!(p.api_key().is_none());
    }

    #[test]
    fn hosted_providers_default_base_urls() {
        assert_eq!(
            ProviderKind::OpenRouter.default_base_url(),
            "https://openrouter.ai/api/v1"
        );
        assert_eq!(
            ProviderKind::Groq.default_base_url(),
            "https://api.groq.com/openai/v1"
        );
    }
}
