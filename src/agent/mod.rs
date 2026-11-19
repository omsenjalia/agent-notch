//! The agent brain: the AI worker behind the Agent slide — provider chain,
//! conversation, screen awareness, desktop control.

pub mod agent;
pub mod config;
pub mod desktop;
pub mod guard;
pub mod host;
pub mod message;
pub mod ollama;
pub mod providers;
pub mod storage;
pub mod vision;

pub use agent::{AgentEngine, AgentEvent, PendingApproval};
pub use host::{global, install, Host, Status};
pub use message::Message;
pub use providers::{AgentError, ChatProvider, ProviderChain};
