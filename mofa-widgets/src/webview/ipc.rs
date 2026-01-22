//! IPC (Inter-Process Communication) between JavaScript and Rust
//!
//! This module provides a simple message-passing system for bidirectional
//! communication between the WebView's JavaScript context and Rust code.

use std::collections::HashMap;

/// A message from JavaScript to Rust
#[derive(Debug, Clone)]
pub struct IpcMessage {
    /// Channel/topic name
    pub channel: String,
    /// JSON data
    pub data: String,
}

impl IpcMessage {
    /// Parse a message from JavaScript JSON
    pub fn from_js(json: &str) -> Self {
        // Try to parse as JSON
        if let Ok(value) = serde_json_minimal_parse(json) {
            if let (Some(channel), Some(data)) = (
                value.get("channel").and_then(|v| v.as_str()),
                value.get("data"),
            ) {
                return Self {
                    channel: channel.to_string(),
                    data: data.to_string(),
                };
            }
        }

        // Fallback: treat entire message as data on "default" channel
        Self {
            channel: "default".to_string(),
            data: json.to_string(),
        }
    }
}

/// Simple JSON value type (to avoid serde_json dependency)
#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if let JsonValue::Object(map) = self {
            map.get(key)
        } else {
            None
        }
    }
}

impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Number(n) => write!(f, "{}", n),
            JsonValue::String(s) => write!(f, "\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, "\"{}\":{}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Minimal JSON parser (avoids serde_json dependency)
fn serde_json_minimal_parse(json: &str) -> Result<JsonValue, ()> {
    let json = json.trim();
    parse_value(&mut json.chars().peekable())
}

fn parse_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, ()> {
    skip_whitespace(chars);

    match chars.peek() {
        Some('"') => parse_string(chars),
        Some('{') => parse_object(chars),
        Some('[') => parse_array(chars),
        Some('t') | Some('f') => parse_bool(chars),
        Some('n') => parse_null(chars),
        Some(c) if c.is_ascii_digit() || *c == '-' => parse_number(chars),
        _ => Err(()),
    }
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, ()> {
    chars.next(); // consume opening "
    let mut s = String::new();

    loop {
        match chars.next() {
            Some('"') => return Ok(JsonValue::String(s)),
            Some('\\') => {
                match chars.next() {
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some(c) => s.push(c),
                    None => return Err(()),
                }
            }
            Some(c) => s.push(c),
            None => return Err(()),
        }
    }
}

fn parse_object(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, ()> {
    chars.next(); // consume {
    let mut map = HashMap::new();

    skip_whitespace(chars);
    if chars.peek() == Some(&'}') {
        chars.next();
        return Ok(JsonValue::Object(map));
    }

    loop {
        skip_whitespace(chars);

        // Parse key
        let key = if let JsonValue::String(k) = parse_string(chars)? {
            k
        } else {
            return Err(());
        };

        skip_whitespace(chars);
        if chars.next() != Some(':') {
            return Err(());
        }

        // Parse value
        let value = parse_value(chars)?;
        map.insert(key, value);

        skip_whitespace(chars);
        match chars.peek() {
            Some(',') => { chars.next(); }
            Some('}') => { chars.next(); return Ok(JsonValue::Object(map)); }
            _ => return Err(()),
        }
    }
}

fn parse_array(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, ()> {
    chars.next(); // consume [
    let mut arr = Vec::new();

    skip_whitespace(chars);
    if chars.peek() == Some(&']') {
        chars.next();
        return Ok(JsonValue::Array(arr));
    }

    loop {
        let value = parse_value(chars)?;
        arr.push(value);

        skip_whitespace(chars);
        match chars.peek() {
            Some(',') => { chars.next(); }
            Some(']') => { chars.next(); return Ok(JsonValue::Array(arr)); }
            _ => return Err(()),
        }
    }
}

fn parse_bool(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, ()> {
    let s: String = chars.by_ref().take(4).collect();
    if s == "true" {
        Ok(JsonValue::Bool(true))
    } else if s.starts_with("fals") {
        chars.next(); // consume 'e'
        Ok(JsonValue::Bool(false))
    } else {
        Err(())
    }
}

fn parse_null(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, ()> {
    let s: String = chars.by_ref().take(4).collect();
    if s == "null" {
        Ok(JsonValue::Null)
    } else {
        Err(())
    }
}

fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, ()> {
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+' {
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    s.parse::<f64>().map(JsonValue::Number).map_err(|_| ())
}

/// Callback type for IPC message handlers
pub type IpcCallback = Box<dyn Fn(&IpcMessage) + Send + Sync>;

/// Handler for IPC messages from JavaScript
pub struct IpcHandler {
    callbacks: HashMap<String, Vec<IpcCallback>>,
    pending_messages: Vec<IpcMessage>,
}

impl IpcHandler {
    /// Create a new IPC handler
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
            pending_messages: Vec::new(),
        }
    }

    /// Register a callback for a specific channel
    pub fn on<F>(&mut self, channel: &str, callback: F)
    where
        F: Fn(&IpcMessage) + Send + Sync + 'static,
    {
        self.callbacks
            .entry(channel.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    /// Handle an incoming message
    pub fn handle_message(&mut self, message: IpcMessage) {
        // Try to call registered callbacks
        if let Some(callbacks) = self.callbacks.get(&message.channel) {
            for cb in callbacks {
                cb(&message);
            }
        }

        // Store in pending for polling
        self.pending_messages.push(message);
    }

    /// Poll for pending messages (clears the queue)
    pub fn poll_messages(&mut self) -> Vec<IpcMessage> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Check if there are pending messages
    pub fn has_pending(&self) -> bool {
        !self.pending_messages.is_empty()
    }
}

impl Default for IpcHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipc_message() {
        let json = r#"{"channel":"test","data":{"foo":"bar"}}"#;
        let msg = IpcMessage::from_js(json);
        assert_eq!(msg.channel, "test");
    }

    #[test]
    fn test_json_parse() {
        let json = r#"{"name":"hello","value":42}"#;
        let parsed = serde_json_minimal_parse(json).unwrap();
        assert_eq!(parsed.get("name").unwrap().as_str(), Some("hello"));
    }
}
