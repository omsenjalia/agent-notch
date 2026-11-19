//! Host: owns the engine and the worker thread; status + events for the UI.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use parking_lot::Mutex;

use crate::agent::agent::{AgentEngine, AgentEvent, PendingApproval};
use crate::agent::config::AgentConfig;
use crate::agent::storage;

/// What the Agent slide shows about the brain.
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Idle,
    Thinking,
    /// Waiting for the user to approve the tool with the given name.
    Approval(String),
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
    tx: mpsc::Sender<String>,
}

impl Host {
    pub fn new(cfg: &AgentConfig) -> Self {
        let engine = Arc::new(Mutex::new(AgentEngine::new(cfg, storage::load_history())));
        let events = Arc::new(Mutex::new(Vec::new()));
        let status = Arc::new(Mutex::new(Status::Idle));
        let approval = Arc::new(AtomicU8::new(0));
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
                        let mut new = engine.lock().process(&text);
                        events.lock().append(&mut new);
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
                                    let mut new = engine.lock().resolve_pending(approved);
                                    events.lock().append(&mut new);
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
        Self { engine, events, status, approval, tx }
    }

    /// Current status for the Agent slide.
    pub fn status(&self) -> Status {
        self.status.lock().clone()
    }

    /// Events published since the last drain.
    pub fn drain_events(&self) -> Vec<AgentEvent> {
        std::mem::take(&mut *self.events.lock())
    }

    /// Send a message to the agent.
    pub fn send(&self, text: String) {
        let _ = self.tx.send(text);
    }

    /// Resolve the pending approval: `true` = approve, `false` = deny.
    pub fn set_approval(&self, approved: bool) {
        self.approval.store(if approved { 1 } else { 2 }, Ordering::SeqCst);
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
