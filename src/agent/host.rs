//! Host: owns the engine and the worker thread; status + events for the UI.
//!
//! All mutations of the engine go through the worker's channel. The UI thread
//! never locks the engine: a chat call can block on network I/O for a while,
//! and holding that lock from a Settings draw would freeze the window.

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

/// Work the single agent worker performs, in order.
enum Command {
    Message(String),
    /// Settings were edited; swap the chain between turns, never mid-request.
    Reconfigure(AgentConfig),
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
    /// Mirror of the engine's pending approval. The UI reads this instead of
    /// locking the engine, which is held for the duration of a network call.
    pending: Arc<Mutex<Option<PendingApproval>>>,
    events: Arc<Mutex<Vec<AgentEvent>>>,
    status: Arc<Mutex<Status>>,
    approval: Arc<AtomicU8>,
    enabled: Arc<AtomicBool>,
    tx: mpsc::Sender<Command>,
}

impl Host {
    pub fn new(cfg: &AgentConfig) -> Self {
        let engine = Arc::new(Mutex::new(AgentEngine::new(cfg, storage::load_history())));
        let pending = Arc::new(Mutex::new(None));
        let events = Arc::new(Mutex::new(Vec::new()));
        let status = Arc::new(Mutex::new(Status::Idle));
        let approval = Arc::new(AtomicU8::new(0));
        let enabled = Arc::new(AtomicBool::new(cfg.enabled));
        let (tx, rx) = mpsc::channel::<Command>();
        {
            let engine = Arc::clone(&engine);
            let pending = Arc::clone(&pending);
            let events = Arc::clone(&events);
            let status = Arc::clone(&status);
            let approval = Arc::clone(&approval);
            let enabled = Arc::clone(&enabled);
            std::thread::Builder::new()
                .name("agent-notch host".to_owned())
                .spawn(move || {
                    while let Ok(command) = rx.recv() {
                        match command {
                            Command::Reconfigure(cfg) => {
                                // A click on Approve that raced with a settings
                                // edit must never approve a *later* tool.
                                approval.store(0, Ordering::SeqCst);
                                let new = engine.lock().reconfigure(&cfg);
                                *pending.lock() = None;
                                push_events(&events, new);
                                enabled.store(cfg.enabled, Ordering::SeqCst);
                                *status.lock() = Status::Idle;
                            }
                            Command::Message(text) => {
                                if !enabled.load(Ordering::SeqCst) {
                                    push_events(
                                        &events,
                                        vec![AgentEvent::Error(
                                            "Agent is off. Turn on “Agent enabled” in Settings › Agent."
                                                .to_owned(),
                                        )],
                                    );
                                    continue;
                                }
                                // A decision left over from a previous turn is
                                // never valid for this one.
                                approval.store(0, Ordering::SeqCst);
                                *status.lock() = Status::Thinking;
                                let new = engine.lock().process(&text);
                                *pending.lock() = engine.lock().pending();
                                push_events(&events, new);
                                Self::resolve_approvals(
                                    &engine, &pending, &events, &status, &approval, &enabled,
                                );
                                *status.lock() = Status::Idle;
                            }
                        }
                    }
                })
                .ok();
        }
        Self {
            engine,
            pending,
            events,
            status,
            approval,
            enabled,
            tx,
        }
    }

    /// Wait for the brain's decision, applying Approve/Deny as they arrive.
    /// An approval token is read only while a tool is actually pending, so a
    /// stale click cannot leak into the next tool.
    fn resolve_approvals(
        engine: &Arc<Mutex<AgentEngine>>,
        pending: &Arc<Mutex<Option<PendingApproval>>>,
        events: &Arc<Mutex<Vec<AgentEvent>>>,
        status: &Arc<Mutex<Status>>,
        approval: &Arc<AtomicU8>,
        enabled: &Arc<AtomicBool>,
    ) {
        loop {
            let tool = {
                let guard = engine.lock();
                if guard.pending().is_none() {
                    *pending.lock() = None;
                    return;
                }
                guard.pending_tool()
            };
            if let Some(tool) = tool {
                *status.lock() = Status::Approval(tool);
            }
            // Switching the brain off while it waits cancels the action.
            let decision = if !enabled.load(Ordering::SeqCst) {
                Some(false)
            } else {
                match approval.swap(0, Ordering::SeqCst) {
                    1 => Some(true),
                    2 => Some(false),
                    _ => None,
                }
            };
            match decision {
                Some(approved) => {
                    *status.lock() = Status::Thinking;
                    let new = engine.lock().resolve_pending(approved);
                    *pending.lock() = engine.lock().pending();
                    push_events(events, new);
                }
                None => std::thread::sleep(Duration::from_millis(20)),
            }
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

    /// Queue an edited settings change. Returns immediately even if a chat
    /// request is in flight; the worker applies it between turns.
    pub fn reconfigure(&self, cfg: &AgentConfig) {
        // Gating the UI is immediate; the chain swap itself is ordered.
        self.enabled.store(cfg.enabled, Ordering::SeqCst);
        let _ = self.tx.send(Command::Reconfigure(cfg.clone()));
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
        let _ = self.tx.send(Command::Message(text));
    }

    /// Resolve the pending approval: `true` = approve, `false` = deny.
    /// A click with nothing pending is ignored so it cannot pre-authorise a
    /// tool the agent has not asked for yet.
    pub fn set_approval(&self, approved: bool) {
        if self.pending_approval().is_none() {
            return;
        }
        self.approval
            .store(if approved { 1 } else { 2 }, Ordering::SeqCst);
    }

    /// The tool awaiting approval, if any.
    pub fn pending_approval(&self) -> Option<PendingApproval> {
        self.pending.lock().clone()
    }

    /// Test-only view of the raw approval token.
    #[cfg(test)]
    fn approval_token(&self) -> u8 {
        self.approval.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wait_for(predicate: impl Fn() -> bool) -> bool {
        for _ in 0..100 {
            if predicate() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }

    #[test]
    fn status_starts_idle() {
        // The global host is only installed by the notch; unit tests construct
        // a standalone host to check the status plumbing.
        let host = Host::new(&AgentConfig::default());
        assert_eq!(host.status(), Status::Idle);
        assert!(host.pending_approval().is_none());
        assert!(host.drain_events().is_empty());
        host.send("hello".to_owned());
    }

    #[test]
    fn reconfigure_clears_a_stale_approval_token() {
        let host = Host::new(&AgentConfig::default());
        // Simulate a click that landed just after the tool disappeared.
        host.approval.store(1, Ordering::SeqCst);
        assert_eq!(host.approval_token(), 1);
        host.reconfigure(&AgentConfig::default());
        assert!(wait_for(|| host.approval_token() == 0));
    }

    #[test]
    fn send_while_disabled_reports_why_instead_of_dropping() {
        let mut cfg = AgentConfig::default();
        cfg.enabled = false;
        let host = Host::new(&cfg);
        host.send("hello".to_owned());
        assert!(wait_for(|| host.transcript().iter().any(
            |e| matches!(e, AgentEvent::Error(m) if m.contains("Agent is off"))
        )));
    }

    #[test]
    fn approval_click_without_a_pending_tool_is_ignored() {
        let host = Host::new(&AgentConfig::default());
        host.set_approval(true);
        assert_eq!(host.approval_token(), 0);
    }
}
