//! Chat history persistence: OpenAI-style messages as JSON at
//! `agent_history.json` in the agent-notch config directory.

use std::fs;
use std::path::PathBuf;

use crate::agent::message::Message;
use crate::config::AppConfig;

/// `<config dir>/agent-notch/agent_history.json`.
pub fn history_path() -> PathBuf {
    let mut path = AppConfig::config_path();
    path.pop(); // drop config.json
    path.push("agent_history.json");
    path
}

pub fn load_history() -> Vec<Message> {
    let path = history_path();
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str::<Vec<Message>>(&text).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_history(history: &[Message]) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(history) {
        let _ = fs::write(&path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_path_lives_in_agent_notch_config_dir() {
        let path = history_path();
        assert!(path.ends_with("agent_history.json"));
        assert!(path.parent().unwrap().ends_with("agent-notch"));
    }
}
