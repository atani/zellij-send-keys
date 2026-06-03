use serde::{Deserialize, Serialize};

/// Maximum payload size in bytes (64KB)
pub const MAX_TEXT_BYTES: usize = 65536;

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
}

impl PaneInfo {
    /// Check if this pane matches the given ID and pane type.
    /// is_plugin == true corresponds to PaneType::Plugin; otherwise Terminal.
    pub fn matches(&self, pane_id: u32, pane_type: PaneType) -> bool {
        self.id == pane_id && self.is_plugin == (pane_type == PaneType::Plugin)
    }
}

/// A target pane identifier that mirrors `zellij_tile`'s `PaneId`.
///
/// Defined here (instead of reusing `zellij_tile::PaneId`) so the resolution
/// logic can be unit-tested on the host target without linking the WASM-only
/// `zellij_tile` host functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneTarget {
    Terminal(u32),
    Plugin(u32),
}

impl PaneTarget {
    /// Build a `PaneTarget` from an id and the parsed `PaneType`.
    pub fn from_type(pane_id: u32, pane_type: PaneType) -> Self {
        match pane_type {
            PaneType::Plugin => PaneTarget::Plugin(pane_id),
            PaneType::Terminal => PaneTarget::Terminal(pane_id),
        }
    }

    /// Does the given cached pane correspond to this target?
    pub fn matches_pane(&self, id: u32, is_plugin: bool) -> bool {
        match self {
            PaneTarget::Terminal(t) => *t == id && !is_plugin,
            PaneTarget::Plugin(p) => *p == id && is_plugin,
        }
    }
}

/// The pipe commands this plugin understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipeCommand {
    SendKeys,
    ListPanes,
    Unknown,
}

/// Map a raw pipe command name to a known `PipeCommand`.
pub fn dispatch_pipe_command(name: &str) -> PipeCommand {
    match name {
        "send_keys" => PipeCommand::SendKeys,
        "list_panes" => PipeCommand::ListPanes,
        _ => PipeCommand::Unknown,
    }
}

/// The concrete side effects to perform for a validated `send_keys` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendKeysPlan {
    /// Bytes to write to the target pane's stdin.
    pub text_bytes: Vec<u8>,
    /// The resolved target pane.
    pub target: PaneTarget,
    /// Whether a delayed Enter key should be scheduled for this target.
    pub schedule_enter: bool,
    /// Human-readable summary for the debug display / logs.
    pub summary: String,
}

/// Resolve a parsed `send_keys` message against the cached pane list into a
/// concrete plan of side effects, or an error describing why it cannot run.
///
/// This is the pure core of `handle_send_keys`: the caller is responsible for
/// actually writing to the pane and scheduling the timer.
pub fn resolve_send_keys(
    msg: &SendKeysMessage,
    panes: &[PaneInfo],
) -> Result<SendKeysPlan, String> {
    if !panes
        .iter()
        .any(|pane| pane.matches(msg.pane_id, msg.pane_type))
    {
        return Err(format!(
            "Pane ID {} (type: {:?}) not found in cached panes",
            msg.pane_id, msg.pane_type
        ));
    }

    let target = PaneTarget::from_type(msg.pane_id, msg.pane_type);
    let summary = format!(
        "Sending to pane {} (type: {:?}, enter: {})",
        msg.pane_id, msg.pane_type, msg.send_enter
    );

    Ok(SendKeysPlan {
        text_bytes: msg.text.as_bytes().to_vec(),
        target,
        schedule_enter: msg.send_enter,
        summary,
    })
}

/// Decide whether a queued Enter should still be delivered to `target`,
/// given the currently cached panes. Used by the Timer handler to avoid
/// writing to a pane that has since closed.
pub fn pending_enter_target_exists(target: PaneTarget, panes: &[PaneInfo]) -> bool {
    panes
        .iter()
        .any(|pane| target.matches_pane(pane.id, pane.is_plugin))
}

/// Parse a JSON string into a SendKeysMessage with validation.
///
/// Text length is limited to `MAX_TEXT_BYTES` bytes (UTF-8 encoded).
pub fn parse_send_keys_message(payload: &str) -> Result<SendKeysMessage, String> {
    let msg: SendKeysMessage =
        serde_json::from_str(payload).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if msg.text.len() > MAX_TEXT_BYTES {
        return Err(format!(
            "Text too long: {} bytes (max: {} bytes)",
            msg.text.len(),
            MAX_TEXT_BYTES
        ));
    }

    Ok(msg)
}

/// Serialize pane list to a JSON string
pub fn serialize_panes(panes: &[PaneInfo]) -> String {
    // PaneInfo contains only primitive types, so serialization cannot fail
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
        let long_text = "a".repeat(MAX_TEXT_BYTES + 1);
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
        let max_text = "a".repeat(MAX_TEXT_BYTES);
        let payload = format!(r#"{{"pane_id": 1, "text": "{}"}}"#, max_text);
        let msg = parse_send_keys_message(&payload).unwrap();
        assert_eq!(msg.text.len(), MAX_TEXT_BYTES);
    }

    #[test]
    fn serialize_panes_success() {
        let panes = vec![PaneInfo {
            id: 1,
            name: Some("test".to_string()),
            is_focused: true,
            is_plugin: false,
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
    fn serialize_panes_with_none_name_and_plugin() {
        let panes = vec![
            PaneInfo {
                id: 1,
                name: None,
                is_focused: false,
                is_plugin: true,
            },
            PaneInfo {
                id: 2,
                name: Some("shell".to_string()),
                is_focused: true,
                is_plugin: false,
            },
        ];
        let json = serialize_panes(&panes);
        assert!(json.contains("\"name\": null"));
        assert!(json.contains("\"is_plugin\": true"));
        assert!(json.contains("\"name\": \"shell\""));
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

    #[test]
    fn pane_info_matches_double_mismatch() {
        let pane = PaneInfo {
            id: 1,
            is_plugin: false,
            ..Default::default()
        };
        assert!(!pane.matches(999, PaneType::Plugin));
    }

    #[test]
    fn parse_empty_string_input() {
        let result = parse_send_keys_message("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse JSON"));
    }

    #[test]
    fn parse_extra_unknown_fields_accepted() {
        let payload = r#"{"pane_id": 1, "text": "hi", "unknown_field": true}"#;
        let msg = parse_send_keys_message(payload).unwrap();
        assert_eq!(msg.pane_id, 1);
        assert_eq!(msg.text, "hi");
    }

    #[test]
    fn parse_missing_required_field_reports_pane_id() {
        let result = parse_send_keys_message(r#"{"text": "hello"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("pane_id"));
    }

    // --- PipeCommand dispatch ---

    #[test]
    fn dispatch_send_keys_command() {
        assert_eq!(dispatch_pipe_command("send_keys"), PipeCommand::SendKeys);
    }

    #[test]
    fn dispatch_list_panes_command() {
        assert_eq!(dispatch_pipe_command("list_panes"), PipeCommand::ListPanes);
    }

    #[test]
    fn dispatch_unknown_command() {
        assert_eq!(dispatch_pipe_command("frobnicate"), PipeCommand::Unknown);
        assert_eq!(dispatch_pipe_command(""), PipeCommand::Unknown);
        // Case sensitivity: the dispatcher matches exact lowercase names only.
        assert_eq!(dispatch_pipe_command("SEND_KEYS"), PipeCommand::Unknown);
    }

    // --- PaneTarget construction & matching ---

    #[test]
    fn pane_target_from_type() {
        assert_eq!(
            PaneTarget::from_type(7, PaneType::Terminal),
            PaneTarget::Terminal(7)
        );
        assert_eq!(
            PaneTarget::from_type(7, PaneType::Plugin),
            PaneTarget::Plugin(7)
        );
    }

    #[test]
    fn pane_target_matches_pane_respects_type() {
        let term = PaneTarget::Terminal(3);
        assert!(term.matches_pane(3, false));
        assert!(!term.matches_pane(3, true)); // same id, but plugin
        assert!(!term.matches_pane(4, false)); // different id

        let plugin = PaneTarget::Plugin(3);
        assert!(plugin.matches_pane(3, true));
        assert!(!plugin.matches_pane(3, false)); // same id, but terminal
    }

    // --- resolve_send_keys ---

    fn pane(id: u32, is_plugin: bool) -> PaneInfo {
        PaneInfo {
            id,
            name: None,
            is_focused: false,
            is_plugin,
        }
    }

    #[test]
    fn resolve_send_keys_terminal_pane() {
        let msg =
            parse_send_keys_message(r#"{"pane_id": 2, "text": "echo hi", "send_enter": true}"#)
                .unwrap();
        let panes = vec![pane(1, false), pane(2, false)];

        let plan = resolve_send_keys(&msg, &panes).unwrap();
        assert_eq!(plan.target, PaneTarget::Terminal(2));
        assert_eq!(plan.text_bytes, b"echo hi".to_vec());
        assert!(plan.schedule_enter);
        assert!(plan.summary.contains("pane 2"));
        assert!(plan.summary.contains("enter: true"));
    }

    #[test]
    fn resolve_send_keys_plugin_pane() {
        let msg =
            parse_send_keys_message(r#"{"pane_id": 9, "text": "cmd", "pane_type": "plugin"}"#)
                .unwrap();
        let panes = vec![pane(9, true)];

        let plan = resolve_send_keys(&msg, &panes).unwrap();
        assert_eq!(plan.target, PaneTarget::Plugin(9));
        assert!(!plan.schedule_enter); // send_enter defaults to false
    }

    #[test]
    fn resolve_send_keys_pane_not_found() {
        let msg = parse_send_keys_message(r#"{"pane_id": 42, "text": "x"}"#).unwrap();
        let panes = vec![pane(1, false)];

        let err = resolve_send_keys(&msg, &panes).unwrap_err();
        assert!(err.contains("Pane ID 42"));
        assert!(err.contains("not found"));
    }

    #[test]
    fn resolve_send_keys_type_mismatch_is_not_found() {
        // A terminal pane with id 5 exists, but the request targets a plugin pane id 5.
        let msg = parse_send_keys_message(r#"{"pane_id": 5, "text": "x", "pane_type": "plugin"}"#)
            .unwrap();
        let panes = vec![pane(5, false)];

        let err = resolve_send_keys(&msg, &panes).unwrap_err();
        assert!(err.contains("Pane ID 5"));
        assert!(err.contains("Plugin"));
    }

    #[test]
    fn resolve_send_keys_preserves_multibyte_bytes() {
        let msg = parse_send_keys_message(r#"{"pane_id": 1, "text": "あ"}"#).unwrap();
        let panes = vec![pane(1, false)];
        let plan = resolve_send_keys(&msg, &panes).unwrap();
        // "あ" is 3 bytes in UTF-8.
        assert_eq!(plan.text_bytes, "あ".as_bytes().to_vec());
        assert_eq!(plan.text_bytes.len(), 3);
    }

    // --- pending_enter_target_exists ---

    #[test]
    fn pending_enter_exists_when_pane_present() {
        let panes = vec![pane(1, false), pane(2, true)];
        assert!(pending_enter_target_exists(PaneTarget::Terminal(1), &panes));
        assert!(pending_enter_target_exists(PaneTarget::Plugin(2), &panes));
    }

    #[test]
    fn pending_enter_absent_when_pane_closed() {
        let panes = vec![pane(1, false)];
        // pane closed
        assert!(!pending_enter_target_exists(
            PaneTarget::Terminal(99),
            &panes
        ));
        // wrong type for an existing id
        assert!(!pending_enter_target_exists(PaneTarget::Plugin(1), &panes));
    }

    #[test]
    fn pending_enter_absent_for_empty_panes() {
        let panes: Vec<PaneInfo> = vec![];
        assert!(!pending_enter_target_exists(
            PaneTarget::Terminal(1),
            &panes
        ));
    }

    #[test]
    fn parse_multibyte_text_limit_is_byte_based() {
        // 4-byte UTF-8 character (emoji)
        let ch = "\u{1F600}"; // 😀 = 4 bytes
        assert_eq!(ch.len(), 4);

        let count = MAX_TEXT_BYTES / 4;
        let text = ch.repeat(count);
        assert_eq!(text.len(), MAX_TEXT_BYTES);

        let payload = format!(r#"{{"pane_id": 1, "text": "{}"}}"#, text);
        let msg = parse_send_keys_message(&payload).unwrap();
        assert_eq!(msg.text.len(), MAX_TEXT_BYTES);

        // One more character exceeds the byte limit
        let text_over = ch.repeat(count + 1);
        let payload_over = format!(r#"{{"pane_id": 1, "text": "{}"}}"#, text_over);
        let result = parse_send_keys_message(&payload_over);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Text too long"));
    }
}
