use std::collections::{BTreeMap, VecDeque};
use zellij_send_keys::{
    dispatch_pipe_command, parse_send_keys_message, pending_enter_target_exists, resolve_send_keys,
    serialize_panes, PaneInfo, PaneTarget, PipeCommand,
};
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
                    let target = pane_id_to_target(pane_id);
                    if pending_enter_target_exists(target, &self.panes) {
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
        match dispatch_pipe_command(&pipe_message.name) {
            PipeCommand::SendKeys => {
                self.handle_send_keys(&pipe_message.payload);
                true
            }
            PipeCommand::ListPanes => {
                self.handle_list_panes();
                true
            }
            PipeCommand::Unknown => {
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

        // Resolve the message into a concrete plan (validates pane existence,
        // selects the target pane). Pure logic lives in the library.
        let plan = match resolve_send_keys(&msg, &self.panes) {
            Ok(plan) => plan,
            Err(e) => {
                eprintln!("send_keys: {}", e);
                return;
            }
        };

        self.last_message = Some(plan.summary);

        let pane_id = target_to_pane_id(plan.target);

        // Send text
        write_to_pane_id(plan.text_bytes, pane_id);

        // Delay Enter via timer to ensure text is processed first
        if plan.schedule_enter {
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

/// Convert the library's host-testable `PaneTarget` into zellij's `PaneId`.
fn target_to_pane_id(target: PaneTarget) -> PaneId {
    match target {
        PaneTarget::Terminal(id) => PaneId::Terminal(id),
        PaneTarget::Plugin(id) => PaneId::Plugin(id),
    }
}

/// Convert zellij's `PaneId` into the library's host-testable `PaneTarget`.
fn pane_id_to_target(pane_id: PaneId) -> PaneTarget {
    match pane_id {
        PaneId::Terminal(id) => PaneTarget::Terminal(id),
        PaneId::Plugin(id) => PaneTarget::Plugin(id),
    }
}
