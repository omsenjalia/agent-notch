//! Approval policy: which tools the brain may run without asking.

use crate::agent::config::ApprovalMode;

/// Whether a tool changes the desktop (mutating) or is a read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    ReadOnly,
    Mutating,
}

/// Tools the brain can run without any approval.
const READ_ONLY: &[&str] = &[
    "screenshot",
    "list_windows",
    "get_foreground",
    "clipboard_read",
    "scroll",
];

pub fn classify(tool: &str) -> ToolKind {
    if READ_ONLY.contains(&tool) {
        ToolKind::ReadOnly
    } else {
        ToolKind::Mutating
    }
}

/// Tools that never run without an explicit approval, even in
/// [`ApprovalMode::Autonomous`] — anything that could destroy or leak.
const IRREVERSIBLE: &[&str] = &["execute_command"];

pub fn irreversible(tool: &str) -> bool {
    IRREVERSIBLE.contains(&tool)
}

/// Whether the brain may run the tool right now: read-only tools always,
/// mutating tools only in autonomous mode, never when irreversible.
pub fn auto_run(mode: ApprovalMode, tool: &str) -> bool {
    if irreversible(tool) {
        return false;
    }
    mode == ApprovalMode::Autonomous || classify(tool) == ToolKind::ReadOnly
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::config::ApprovalMode;

    #[test]
    fn read_only_tools_auto_run_in_both_modes() {
        for mode in [ApprovalMode::Ask, ApprovalMode::Autonomous] {
            assert!(auto_run(mode, "screenshot"));
            assert!(auto_run(mode, "list_windows"));
            assert!(auto_run(mode, "clipboard_read"));
        }
    }

    #[test]
    fn mutating_tools_never_auto_run_in_ask_mode() {
        assert!(!auto_run(ApprovalMode::Ask, "click"));
        assert!(!auto_run(ApprovalMode::Ask, "launch"));
        assert!(!auto_run(ApprovalMode::Ask, "clipboard_write"));
    }

    #[test]
    fn mutating_tools_auto_run_in_autonomous_mode() {
        assert!(auto_run(ApprovalMode::Autonomous, "click"));
        assert!(auto_run(ApprovalMode::Autonomous, "type_text"));
        assert!(auto_run(ApprovalMode::Autonomous, "activate_window"));
    }

    #[test]
    fn irreversible_tools_never_auto_run() {
        assert!(!auto_run(ApprovalMode::Autonomous, "execute_command"));
    }
}
