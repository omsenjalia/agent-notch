//! Message types and decision parsing for the agent brain.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One message in the brain's conversation. Mirrors the OpenAI chat format:
/// a system or user message, assistant messages (with tool calls), and tool
/// results.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    /// Data URIs of screenshots attached to a user message (vision models).
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: String) -> Self {
        Self {
            role: "system".into(),
            content: Some(content),
            ..Default::default()
        }
    }

    pub fn user(content: String, images: Vec<String>) -> Self {
        Self {
            role: "user".into(),
            content: Some(content),
            images,
            ..Default::default()
        }
    }

    /// An assistant message carrying tool calls (sent back to the provider
    /// before the tool results).
    pub fn assistant(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".into(),
            content: None,
            tool_calls: Some(tool_calls),
            ..Default::default()
        }
    }

    pub fn assistant_text(text: String) -> Self {
        Self {
            role: "assistant".into(),
            content: Some(text),
            ..Default::default()
        }
    }

    pub fn tool_result(tool_call_id: String, output: String) -> Self {
        Self {
            role: "tool".into(),
            content: Some(output),
            tool_call_id: Some(tool_call_id),
            ..Default::default()
        }
    }
}

/// A tool the brain asks a provider to run: the id it echoes back, the tool
/// name, and the arguments as a JSON string.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// What the brain sends to a provider: the model and the messages so far.
#[derive(Clone, Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

/// What the provider answers: text the brain can parse, or tool calls.
#[derive(Clone, Debug, Default)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

/// The brain's next move: answer the user, or run a tool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Answer(String),
    Tool { tool: String, args: Value },
}

impl Decision {
    /// Parse the model's text into a decision. The brain is asked to answer
    /// with `{"type":"answer","text":...}` or
    /// `{"type":"tool","tool":"...","args":{...}}`; when it replies without a
    /// JSON object, the text itself is the answer.
    pub fn from_text(text: &str) -> Decision {
        let trimmed = text.trim();
        if let Some(obj) = find_json_object(trimmed) {
            if let Ok(value) = serde_json::from_str::<Value>(obj) {
                match value.get("type").and_then(Value::as_str) {
                    Some("answer") => {
                        let text = value
                            .get("text")
                            .or_else(|| value.get("answer"))
                            .and_then(Value::as_str)
                            .unwrap_or(trimmed)
                            .to_owned();
                        return Decision::Answer(text);
                    }
                    Some("tool") => {
                        let tool = value
                            .get("tool")
                            .or_else(|| value.get("name"))
                            .and_then(Value::as_str)
                            .map(String::from)
                            .unwrap_or_default();
                        let args = value
                            .get("args")
                            .or_else(|| value.get("parameters"))
                            .cloned()
                            .unwrap_or(Value::Object(Default::default()));
                        return Decision::Tool { tool, args };
                    }
                    _ => {}
                }
            }
        }
        Decision::Answer(trimmed.to_owned())
    }
}

/// The first balanced `{...}` object in `text`, as a `&str` slice.
pub fn find_json_object(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let start = bytes.iter().position(|&b| b == b'{')?;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for i in start..bytes.len() {
        let b = bytes[i];
        if in_string {
            match b {
                b'\\' if !escaped => escaped = true,
                b'"' if !escaped => in_string = false,
                _ => escaped = false,
            }
        } else {
            match b {
                b'"' => in_string = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&text[start..=i]);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Serialize messages to the OpenAI chat format.
pub fn messages_to_openai(messages: &[Message]) -> Vec<Value> {
    messages.iter().map(message_to_openai).collect()
}

fn message_to_openai(m: &Message) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("role".into(), Value::String(m.role.clone()));
    if !m.images.is_empty() {
        // Vision input: content as an array of text + image_url parts.
        let mut parts: Vec<Value> = vec![serde_json::json!({
            "type": "text",
            "text": m.content.clone().unwrap_or_default(),
        })];
        for url in &m.images {
            parts.push(serde_json::json!({
                "type": "image_url",
                "image_url": { "url": url },
            }));
        }
        obj.insert("content".into(), Value::Array(parts));
    } else {
        obj.insert(
            "content".into(),
            m.content.clone().map(Value::String).unwrap_or(Value::Null),
        );
    }
    if let Some(calls) = &m.tool_calls {
        obj.insert(
            "tool_calls".into(),
            Value::Array(calls.iter().map(tool_call_to_openai).collect()),
        );
    }
    if let Some(id) = &m.tool_call_id {
        obj.insert("tool_call_id".into(), Value::String(id.clone()));
    }
    Value::Object(obj)
}

fn tool_call_to_openai(c: &ToolCall) -> Value {
    serde_json::json!({
        "id": c.id,
        "type": "function",
        "function": { "name": c.name, "arguments": c.arguments },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_parses_answer_json() {
        assert_eq!(
            Decision::from_text("Here you go: {\"type\":\"answer\",\"text\":\"done\"}"),
            Decision::Answer("done".to_owned())
        );
    }

    #[test]
    fn decision_parses_tool_json() {
        assert_eq!(
            Decision::from_text("{\"type\":\"tool\",\"tool\":\"click\",\"args\":{\"x\":10,\"y\":20}}"),
            Decision::Tool {
                tool: "click".to_owned(),
                args: serde_json::json!({"x": 10, "y": 20}),
            }
        );
    }

    #[test]
    fn decision_falls_back_to_raw_text() {
        assert_eq!(Decision::from_text("nothing here"), Decision::Answer("nothing here".to_owned()));
    }

    #[test]
    fn decision_defaults_tool_args() {
        assert_eq!(
            Decision::from_text("{\"type\":\"tool\",\"tool\":\"screenshot\"}"),
            Decision::Tool {
                tool: "screenshot".to_owned(),
                args: serde_json::Value::Object(Default::default()),
            }
        );
    }
}
