//! Host: owns the engine and the worker thread; status + events for the UI.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use parking_lot::Mutex;

use crate::agent::agent::{AgentEngine, AgentEvent, PendingApproval};
use crate::agent::config::AgentConfig;
use crate::agent::storage;

/// How much of the conversation the UIs keep on screen. Old events are
/// dropped so a long session cannot grow the log without bound.
const TRANSCRIPT_LIMIT: usize = 120;

/// What the Agent slide shows about the brain.
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Idle,
    Thinking,
    /// Waiting for the user to approve the tool with the given name.
    Approval(String),
}

/// Append to the transcript, keeping it bounded.
fn push_events(events: &Arc<Mutex<Vec<AgentEvent>>>, mut new: Vec<AgentEvent>) {
    let mut log = events.lock();
    log.append(&mut new);
    if log.len() > TRANSCRIPT_LIMIT {
        let drop = log.len() - TRANSCRIPT_LIMIT;
        log.drain(..drop);
    }
}

/// The global agent host, installed by the notch at startup.
static HOST: OnceLock<Arc<Host>> = OnceLock::new();

/// Install (once) or return the global host.
pub fn install(cfg: &AgentConfig) -> Arc<Host> {
    HOST.get_or_init(|| Arc::new(Host::new(cfg))).clone()
}

/// The global host, if the notch has installed one.
pub fn global() -> Option<Arc<Host>> {
    HOST.get().cloned()
}

/// The host: engine + events + status, shared with the worker thread.
pub struct Host {
    engine: Arc<Mutex<AgentEngine>>,
    events: Arc<Mutex<Vec<AgentEvent>>>,
    status: Arc<Mutex<Status>>,
    approval: Arc<AtomicU8>,
    enabled: Arc<AtomicBool>,
    tx: mpsc::Sender<String>,
}

impl Host {
    pub fn new(cfg: &AgentConfig) -> Self {
        let engine = Arc::new(Mutex::new(AgentEngine::new(cfg, storage::load_history())));
        let events = Arc::new(Mutex::new(Vec::new()));
        let status = Arc::new(Mutex::new(Status::Idle));
        let approval = Arc::new(AtomicU8::new(0));
        let enabled = Arc::new(AtomicBool::new(cfg.enabled));
        let (tx, rx) = mpsc::channel::<String>();
        {
            let engine = Arc::clone(&engine);
            let events = Arc::clone(&events);
            let status = Arc::clone(&status);
            let approval = Arc::clone(&approval);
            std::thread::Builder::new()
                .name("agent-notch host".to_owned())
                .spawn(move || {
                    while let Ok(text) = rx.recv() {
                        *status.lock() = Status::Thinking;
                        let new = engine.lock().process(&text);
                        push_events(&events, new);
                        while engine.lock().pending().is_some() {
                            if let Some(tool) = engine.lock().pending_tool() {
                                *status.lock() = Status::Approval(tool);
                            }
                            let decision = match approval.swap(0, Ordering::SeqCst) {
                                1 => Some(true),
                                2 => Some(false),
                                _ => None,
                            };
                            match decision {
                                Some(approved) => {
                                    *status.lock() = Status::Thinking;
                                    let new = engine.lock().resolve_pending(approved);
                                    push_events(&events, new);
                                }
                                None => {
                                    std::thread::sleep(Duration::from_millis(20));
                                }
                            }
                        }
                        *status.lock() = Status::Idle;
                    }
                })
                .ok();
        }
        Self {
            engine,
            events,
            status,
            approval,
            enabled,
            tx,
        }
    }

    /// Current status for the Agent slide.
    pub fn status(&self) -> Status {
        self.status.lock().clone()
    }

    /// The whole conversation so far. Both the Settings page and the notch
    /// read this: draining would race, so whichever UI drew first would steal
    /// the reply from the other.
    pub fn transcript(&self) -> Vec<AgentEvent> {
        self.events.lock().clone()
    }

    /// Backwards-compatible name for [`Host::transcript`].
    pub fn drain_events(&self) -> Vec<AgentEvent> {
        self.transcript()
    }

    /// Apply edited settings without restarting the app. The chat page calls
    /// this as soon as a provider or the approval mode changes.
    pub fn reconfigure(&self, cfg: &AgentConfig) {
        self.enabled.store(cfg.enabled, Ordering::SeqCst);
        let mut engine = self.engine.lock();
        engine.reconfigure(cfg);
    }

    /// Whether the brain is switched on in Settings.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Send a message to the agent. When it is switched off, say so in the
    /// conversation instead of silently dropping the message.
    pub fn send(&self, text: String) {
        if !self.is_enabled() {
            push_events(
                &self.events,
                vec![AgentEvent::Error(
                    "Agent is off. Turn on “Agent enabled” in Settings › Agent.".to_owned(),
                )],
            );
            return;
        }
        let _ = self.tx.send(text);
    }

    /// Resolve the pending approval: `true` = approve, `false` = deny.
    pub fn set_approval(&self, approved: bool) {
        self.approval
            .store(if approved { 1 } else { 2 }, Ordering::SeqCst);
    }

    /// The tool awaiting approval, if any.
    pub fn pending_approval(&self) -> Option<PendingApproval> {
        self.engine.lock().pending()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_starts_idle() {
        // The global host is only installed by the notch; unit tests construct
        // a standalone host to check the status plumbing.
        let host = Host::new(&AgentConfig::default());
        assert_eq!(host.status(), Status::Idle);
        assert!(host.pending_approval().is_none());
        assert!(host.drain_events().is_empty());
        // A message goes nowhere harmful: the worker thread is alive and the
        // engine is idle, so no events are published yet.
        host.send("hello".to_owned());
    }
}
