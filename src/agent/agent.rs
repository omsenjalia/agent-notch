//! The agent engine: the turn loop — screen context, decide, answer or
//! execute a tool, repeat.

use serde_json::Value;

use crate::agent::config::{AgentConfig, ApprovalMode};
use crate::agent::desktop;
use crate::agent::guard;
use crate::agent::message::{ChatRequest, Decision, Message, ToolCall};
use crate::agent::providers::ProviderChain;
use crate::agent::storage;
use crate::agent::vision::ScreenContext;

/// Maximum tool rounds per turn before the agent stops and answers.
const MAX_ROUNDS: u32 = 8;

/// One event the brain publishes for the UI: what the user said, what the
/// agent decided and did.
#[derive(Clone, Debug)]
pub enum AgentEvent {
    UserMessage(String),
    /// The agent asked to run a tool.
    ToolCall {
        tool: String,
        description: String,
        id: String,
    },
    /// The tool ran.
    ToolResult {
        tool: String,
        ok: bool,
        output: String,
    },
    /// The user denied the pending tool.
    ToolSkipped {
        tool: String,
    },
    /// The agent needs approval before running `tool`.
    ApprovalRequired {
        tool: String,
        id: String,
        description: String,
    },
    /// The agent's final answer for the user.
    Answer(String),
    /// A provider or tool call failed.
    Error(String),
}

/// A tool the agent wants to run and is waiting to be approved.
#[derive(Clone, Debug)]
pub struct PendingApproval {
    pub tool: String,
    pub id: String,
    pub args: Value,
    pub description: String,
}

pub struct AgentEngine {
    chain: ProviderChain,
    approval_mode: ApprovalMode,
    history: Vec<Message>,
    ctx: Option<ScreenContext>,
    pending: Option<PendingApproval>,
}

impl AgentEngine {
    pub fn new(cfg: &AgentConfig, history: Vec<Message>) -> Self {
        Self {
            chain: ProviderChain::new(cfg.chain.clone()),
            approval_mode: cfg.approval_mode,
            history,
            ctx: None,
            pending: None,
        }
    }

    pub fn history(&self) -> &[Message] {
        &self.history
    }

    pub fn pending(&self) -> Option<PendingApproval> {
        self.pending.clone()
    }

    pub fn pending_tool(&self) -> Option<String> {
        self.pending.as_ref().map(|p| p.tool.clone())
    }

    /// Run one turn: gather the screen context, ask the decision model, run
    /// tools until the agent answers or waits for approval.
    pub fn process(&mut self, text: &str) -> Vec<AgentEvent> {
        let mut events = vec![AgentEvent::UserMessage(text.to_owned())];
        self.ctx = Some(ScreenContext::gather(&self.chain));
        let ctx = self.ctx.clone();
        self.history
            .push(Message::system(self.system_prompt(ctx.as_ref())));
        let images = ctx.as_ref().map(|c| c.screens.clone()).unwrap_or_default();
        self.history.push(Message::user(text.to_owned(), images));
        events.extend(self.run_rounds());
        storage::save_history(&self.history);
        events
    }

    /// Continue after the user approved or denied the pending tool.
    pub fn resolve_pending(&mut self, approved: bool) -> Vec<AgentEvent> {
        let Some(p) = self.pending.take() else {
            return Vec::new();
        };
        let mut events = Vec::new();
        if approved {
            let (ok, output) = desktop::run_tool(&p.tool, &p.args);
            events.push(AgentEvent::ToolResult {
                tool: p.tool.clone(),
                ok,
                output: output.clone(),
            });
            self.history
                .push(Message::tool_result(p.id.clone(), output));
        } else {
            events.push(AgentEvent::ToolSkipped {
                tool: p.tool.clone(),
            });
            self.history.push(Message::tool_result(
                p.id.clone(),
                "cancelled by the user".to_owned(),
            ));
        }
        events.extend(self.run_rounds());
        storage::save_history(&self.history);
        events
    }

    fn run_rounds(&mut self) -> Vec<AgentEvent> {
        let mut events = Vec::new();
        for _round in 0..MAX_ROUNDS {
            let Some(idx) = self.chain.decision_index() else {
                events.push(AgentEvent::Error(
                    "no provider is reachable \u{2014} check Settings \u{203a} Agent".to_owned(),
                ));
                return events;
            };
            let model = self.chain.config(idx).model.clone();
            let request = ChatRequest {
                model,
                messages: self.history.clone(),
            };
            let response = match self.chain.chat(&request) {
                Ok(response) => response,
                Err(error) => {
                    events.push(AgentEvent::Error(error.message));
                    return events;
                }
            };
            let decision = if !response.tool_calls.is_empty() {
                let call = &response.tool_calls[0];
                Decision::Tool {
                    tool: call.name.clone(),
                    args: serde_json::from_str::<Value>(&call.arguments)
                        .unwrap_or(Value::Object(Default::default())),
                }
            } else {
                Decision::from_text(&response.content.unwrap_or_default())
            };
            match decision {
                Decision::Answer(text) => {
                    self.history.push(Message::assistant_text(text.clone()));
                    events.push(AgentEvent::Answer(text));
                    return events;
                }
                Decision::Tool { tool, args } => {
                    let description = self.describe_tool(&tool, &args);
                    let id = format!("call_{}", self.history.len());
                    events.push(AgentEvent::ToolCall {
                        tool: tool.clone(),
                        description: description.clone(),
                        id: id.clone(),
                    });
                    let call = ToolCall {
                        id: id.clone(),
                        name: tool.clone(),
                        arguments: serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_owned()),
                    };
                    self.history.push(Message::assistant(vec![call]));
                    if guard::auto_run(self.approval_mode, &tool) {
                        let (ok, output) = desktop::run_tool(&tool, &args);
                        events.push(AgentEvent::ToolResult {
                            tool: tool.clone(),
                            ok,
                            output: output.clone(),
                        });
                        self.history.push(Message::tool_result(id, output));
                    } else {
                        self.pending = Some(PendingApproval {
                            tool: tool.clone(),
                            id: id.clone(),
                            args: args.clone(),
                            description: description.clone(),
                        });
                        events.push(AgentEvent::ApprovalRequired {
                            tool,
                            id,
                            description,
                        });
                        return events;
                    }
                }
            }
        }
        events.push(AgentEvent::Error(
            "agent reached the maximum number of tool rounds".to_owned(),
        ));
        events
    }

    fn system_prompt(&self, ctx: Option<&ScreenContext>) -> String {
        let summary = ctx
            .map(ScreenContext::summary)
            .unwrap_or_else(|| "No foreground window visible".to_owned());
        format!(
            "You are agent-notch's agent brain. Reply ONLY with a JSON object: \
             {{\"type\":\"answer\",\"text\":\"...\"}} or \
             {{\"type\":\"tool\",\"tool\":\"NAME\",\"args\":{{...}}}}. \
             Available tools: click, right_click, double_click, drag, type_text, press_chord, \
             scroll, activate_window, launch, execute_command, clipboard_read, clipboard_write, \
             screenshot, list_windows, get_foreground. \
             Current screen: {summary}"
        )
    }

    fn describe_tool(&self, tool: &str, args: &Value) -> String {
        match tool {
            "click" => format!("click ({}, {})", arg(args, "x"), arg(args, "y")),
            "type_text" => format!("type: {}", arg(args, "text")),
            "launch" => format!("launch {}", arg(args, "path")),
            "execute_command" => format!("run: {}", arg(args, "command")),
            "activate_window" => format!("activate: {}", arg(args, "title")),
            other => format!("run {other}"),
        }
    }
}

fn arg(args: &Value, name: &str) -> String {
    args.get(name)
        .and_then(Value::as_str)
        .unwrap_or("")
        .chars()
        .take(60)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    use super::*;
    use crate::agent::config::{AgentProviderConfig, ProviderKind};

    const TOOL_RESPONSE: &str = r#"{"choices":[{"message":{"content":null,"tool_calls":[{"id":"c1","type":"function","function":{"name":"click","arguments":"{\"x\":100,\"y\":200}"}}]}}]}"#;
    const ANSWER_RESPONSE: &str = r#"{"choices":[{"message":{"content":"{\"type\":\"answer\",\"text\":\"clicked (100, 200)\"}"}}]}"#;

    fn find_sub(hay: &[u8], needle: &[u8]) -> Option<usize> {
        hay.windows(needle.len()).position(|w| w == needle)
    }

    /// Read a full HTTP request (headers + body by Content-Length).
    fn read_request(stream: &mut TcpStream) -> String {
        let mut buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = stream.read(&mut chunk).unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(pos) = find_sub(&buf, b"\r\n\r\n") {
                let header_len = pos + 4;
                let header = String::from_utf8_lossy(&buf[..header_len]).to_string();
                let content_length = header
                    .lines()
                    .find_map(|l| {
                        l.to_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if buf.len() - header_len >= content_length {
                    return header;
                }
            }
        }
        String::from_utf8_lossy(&buf).to_string()
    }

    fn respond(stream: &mut TcpStream, body: &str) {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
    }

    fn mock_config(addr: std::net::SocketAddr) -> AgentConfig {
        let mut config = AgentConfig::default();
        config.chain = vec![AgentProviderConfig {
            name: "mock".into(),
            kind: ProviderKind::Local,
            base_url: format!("http://{addr}/v1"),
            api_key_env: None,
            model: "mock-model".into(),
            decision_model: true,
            vision: false,
        }];
        config
    }

    #[test]
    fn process_asks_for_approval_then_denies() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            for response in [TOOL_RESPONSE, ANSWER_RESPONSE] {
                let (mut stream, _) = listener.accept().unwrap();
                let _ = read_request(&mut stream);
                respond(&mut stream, response);
            }
        });
        let config = mock_config(addr);
        let mut engine = AgentEngine::new(&config, Vec::new());

        let events = engine.process("click around");
        assert!(events
            .iter()
            .any(|e| matches!(e, AgentEvent::ApprovalRequired { tool, .. } if tool == "click")));
        assert!(engine.pending_tool().as_deref() == Some("click"));

        // Denying skips the tool, then the second mock response is the answer.
        let events = engine.resolve_pending(false);
        assert!(events
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolSkipped { tool } if tool == "click")));
        assert!(events
            .iter()
            .any(|e| matches!(e, AgentEvent::Answer(t) if t.contains("clicked"))));
        server.join().unwrap();
    }

    #[test]
    fn process_answers_without_tools() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            for response in [ANSWER_RESPONSE] {
                let (mut stream, _) = listener.accept().unwrap();
                let _ = read_request(&mut stream);
                respond(&mut stream, response);
            }
        });
        let config = mock_config(addr);
        let mut engine = AgentEngine::new(&config, Vec::new());
        let events = engine.process("hello");
        assert!(events
            .iter()
            .any(|e| matches!(e, AgentEvent::Answer(t) if t == "clicked (100, 200)")));
        assert!(!events
            .iter()
            .any(|e| matches!(e, AgentEvent::ApprovalRequired { .. })));
        server.join().unwrap();
    }
}
