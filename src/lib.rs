use serde::{Deserialize, Serialize};

/// Maximum payload size (64KB)
pub const MAX_TEXT_LENGTH: usize = 65536;

/// Pane type
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PaneType {
    #[default]
    Terminal,
    Plugin,
}

/// Pipe message format for send_keys command
#[derive(Deserialize, Debug)]
pub struct SendKeysMessage {
    /// Target pane ID
    pub pane_id: u32,
    /// Text to send
    pub text: String,
    /// Whether to send Enter key after the text
    #[serde(default)]
    pub send_enter: bool,
    /// Pane type (default: terminal)
    #[serde(default)]
    pub pane_type: PaneType,
}

/// Pane information
#[derive(Serialize, Clone, Debug, Default)]
pub struct PaneInfo {
    pub id: u32,
    pub name: Option<String>,
    pub is_focused: bool,
    pub is_plugin: bool,
    pub is_suppressed: bool,
}

impl PaneInfo {
    /// Check if this pane matches the given ID and pane type
    pub fn matches(&self, pane_id: u32, pane_type: PaneType) -> bool {
        self.id == pane_id && self.is_plugin == (pane_type == PaneType::Plugin)
    }
}

/// Parse a JSON string into a SendKeysMessage with validation.
///
/// Text length is limited to `MAX_TEXT_LENGTH` bytes (UTF-8 encoded).
pub fn parse_send_keys_message(payload: &str) -> Result<SendKeysMessage, String> {
    let msg: SendKeysMessage =
        serde_json::from_str(payload).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if msg.text.len() > MAX_TEXT_LENGTH {
        return Err(format!(
            "Text too long: {} bytes (max: {} bytes)",
            msg.text.len(),
            MAX_TEXT_LENGTH
        ));
    }

    Ok(msg)
}

/// Serialize pane list to a JSON string
pub fn serialize_panes(panes: &[PaneInfo]) -> String {
    // PaneInfoは基本型のみで構成されるためシリアライズは失敗しない
    serde_json::to_string_pretty(panes).expect("PaneInfo serialization should never fail")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_message() {
        let payload = r#"{"pane_id": 1, "text": "hello"}"#;
        let msg = parse_send_keys_message(payload).unwrap();
        assert_eq!(msg.pane_id, 1);
        assert_eq!(msg.text, "hello");
        assert!(!msg.send_enter);
        assert_eq!(msg.pane_type, PaneType::Terminal);
    }

    #[test]
    fn parse_message_with_all_fields() {
        let payload =
            r#"{"pane_id": 5, "text": "ls", "send_enter": true, "pane_type": "terminal"}"#;
        let msg = parse_send_keys_message(payload).unwrap();
        assert_eq!(msg.pane_id, 5);
        assert_eq!(msg.text, "ls");
        assert!(msg.send_enter);
        assert_eq!(msg.pane_type, PaneType::Terminal);
    }

    #[test]
    fn parse_message_plugin_pane_type() {
        let payload = r#"{"pane_id": 3, "text": "cmd", "pane_type": "plugin"}"#;
        let msg = parse_send_keys_message(payload).unwrap();
        assert_eq!(msg.pane_type, PaneType::Plugin);
    }

    #[test]
    fn parse_invalid_json() {
        let result = parse_send_keys_message("not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse JSON"));
    }

    #[test]
    fn parse_missing_required_fields() {
        let result = parse_send_keys_message(r#"{"text": "hello"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn parse_text_too_long() {
        let long_text = "a".repeat(MAX_TEXT_LENGTH + 1);
        let payload = format!(r#"{{"pane_id": 1, "text": "{}"}}"#, long_text);
        let result = parse_send_keys_message(&payload);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Text too long"));
    }

    #[test]
    fn parse_invalid_pane_type() {
        let payload = r#"{"pane_id": 1, "text": "hello", "pane_type": "invalid"}"#;
        let result = parse_send_keys_message(payload);
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_text() {
        let payload = r#"{"pane_id": 1, "text": ""}"#;
        let msg = parse_send_keys_message(payload).unwrap();
        assert_eq!(msg.text, "");
    }

    #[test]
    fn parse_text_at_max_length() {
        let max_text = "a".repeat(MAX_TEXT_LENGTH);
        let payload = format!(r#"{{"pane_id": 1, "text": "{}"}}"#, max_text);
        let msg = parse_send_keys_message(&payload).unwrap();
        assert_eq!(msg.text.len(), MAX_TEXT_LENGTH);
    }

    #[test]
    fn serialize_panes_success() {
        let panes = vec![PaneInfo {
            id: 1,
            name: Some("test".to_string()),
            is_focused: true,
            is_plugin: false,
            is_suppressed: false,
        }];
        let json = serialize_panes(&panes);
        assert!(json.contains("\"id\": 1"));
        assert!(json.contains("\"name\": \"test\""));
    }

    #[test]
    fn serialize_empty_panes() {
        let panes: Vec<PaneInfo> = vec![];
        let json = serialize_panes(&panes);
        assert_eq!(json, "[]");
    }

    #[test]
    fn pane_info_matches_terminal() {
        let pane = PaneInfo {
            id: 1,
            is_plugin: false,
            ..Default::default()
        };
        assert!(pane.matches(1, PaneType::Terminal));
        assert!(!pane.matches(1, PaneType::Plugin));
        assert!(!pane.matches(2, PaneType::Terminal));
    }

    #[test]
    fn pane_info_matches_plugin() {
        let pane = PaneInfo {
            id: 5,
            is_plugin: true,
            ..Default::default()
        };
        assert!(pane.matches(5, PaneType::Plugin));
        assert!(!pane.matches(5, PaneType::Terminal));
    }
}
