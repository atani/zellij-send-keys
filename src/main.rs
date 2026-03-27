use std::collections::{BTreeMap, VecDeque};
use zellij_send_keys::{parse_send_keys_message, serialize_panes, PaneInfo, PaneType};
use zellij_tile::prelude::*;

/// Delay in seconds before sending Enter key after text.
/// Allows the terminal to process the text before Enter is pressed.
const ENTER_DELAY_SECS: f64 = 0.2;

/// Plugin state
#[derive(Default)]
struct State {
    /// Last processed message (for debug display)
    last_message: Option<String>,
    /// Cached pane list (includes plugin panes, excludes suppressed)
    panes: Vec<PaneInfo>,
    /// Queue of pending Enter key sends (delayed via timer)
    pending_enter: VecDeque<PaneId>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::WriteToStdin,
            PermissionType::ReadApplicationState,
        ]);

        subscribe(&[EventType::PaneUpdate, EventType::Timer]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(pane_manifest) => {
                // Exclude suppressed panes; keep both plugin and terminal.
                // Zellij assigns globally unique pane IDs across tabs.
                self.panes = pane_manifest
                    .panes
                    .values()
                    .flat_map(|tab_panes| tab_panes.iter())
                    .filter(|pane| !pane.is_suppressed)
                    .map(|pane| PaneInfo {
                        id: pane.id,
                        name: Some(pane.title.clone()),
                        is_focused: pane.is_focused,
                        is_plugin: pane.is_plugin,
                    })
                    .collect();
                true
            }
            Event::Timer(_) => {
                // Delayed Enter key send; verify pane still exists before writing
                if let Some(pane_id) = self.pending_enter.pop_front() {
                    let pane_exists = self.panes.iter().any(|pane| match pane_id {
                        PaneId::Terminal(id) => pane.id == id && !pane.is_plugin,
                        PaneId::Plugin(id) => pane.id == id && pane.is_plugin,
                    });
                    if pane_exists {
                        write_to_pane_id(vec![b'\r'], pane_id);
                    } else {
                        eprintln!(
                            "Timer: target pane {:?} no longer exists, skipping Enter",
                            pane_id
                        );
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        // Dispatch by pipe command name
        match pipe_message.name.as_str() {
            "send_keys" => {
                self.handle_send_keys(&pipe_message.payload);
                true
            }
            "list_panes" => {
                self.handle_list_panes();
                true
            }
            _ => {
                eprintln!("Unknown pipe command: {:?}", pipe_message.name);
                false
            }
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!("=== zellij-send-keys ===");
        println!();

        if let Some(msg) = &self.last_message {
            println!("Last message: {}", msg);
        }

        // Display terminal panes only
        let terminal_panes: Vec<_> = self.panes.iter().filter(|pane| !pane.is_plugin).collect();
        println!();
        println!("Available panes ({}):", terminal_panes.len());
        for pane in &terminal_panes {
            let focused = if pane.is_focused { " [focused]" } else { "" };
            let name = pane.name.as_deref().unwrap_or("(unnamed)");
            println!("  - ID: {}, Name: {}{}", pane.id, name, focused);
        }

        println!();
        println!("Usage:");
        println!("  zellij action pipe --plugin file:zellij-send-keys.wasm \\");
        println!("    --name send_keys -- '{{\"pane_id\": 1, \"text\": \"hello\", \"send_enter\": true, \"pane_type\": \"terminal\"}}'");
    }
}

impl State {
    fn handle_send_keys(&mut self, payload: &Option<String>) {
        let Some(payload) = payload else {
            eprintln!("send_keys: No payload provided");
            return;
        };

        let msg = match parse_send_keys_message(payload) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("send_keys: {}", e);
                return;
            }
        };

        // Verify pane exists (check both ID and type)
        if !self
            .panes
            .iter()
            .any(|pane| pane.matches(msg.pane_id, msg.pane_type))
        {
            eprintln!(
                "send_keys: Pane ID {} (type: {:?}) not found in cached panes",
                msg.pane_id, msg.pane_type
            );
            return;
        }

        self.last_message = Some(format!(
            "Sending to pane {} (type: {:?}, enter: {})",
            msg.pane_id, msg.pane_type, msg.send_enter
        ));

        // Build PaneId based on pane type
        let pane_id = match msg.pane_type {
            PaneType::Plugin => PaneId::Plugin(msg.pane_id),
            PaneType::Terminal => PaneId::Terminal(msg.pane_id),
        };

        // Send text
        write_to_pane_id(msg.text.as_bytes().to_vec(), pane_id);

        // Delay Enter via timer to ensure text is processed first
        if msg.send_enter {
            self.pending_enter.push_back(pane_id);
            set_timeout(ENTER_DELAY_SECS);
        }

        eprintln!(
            "send_keys: Sent to pane {} (type: {:?}, enter: {})",
            msg.pane_id, msg.pane_type, msg.send_enter
        );
    }

    fn handle_list_panes(&self) {
        println!("{}", serialize_panes(&self.panes));
    }
}
