use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use zellij_tile::prelude::*;

/// プラグインの状態
#[derive(Default)]
struct State {
    /// 最後に処理したメッセージ（デバッグ用）
    last_message: Option<String>,
    /// ペイン一覧のキャッシュ
    panes: Vec<PaneInfo>,
}

/// pipeメッセージの形式
#[derive(Deserialize, Debug)]
struct SendKeysMessage {
    /// 送信先ペインID（Terminal ID）
    pane_id: u32,
    /// 送信するテキスト
    text: String,
    /// Enterキーを送信するか
    #[serde(default)]
    send_enter: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
struct PaneInfo {
    id: u32,
    name: Option<String>,
    is_focused: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        // 必要なパーミッションを要求
        request_permission(&[
            PermissionType::WriteToStdin,  // ペインへの書き込み
            PermissionType::ReadApplicationState, // ペイン一覧の取得
        ]);

        // イベントを購読
        subscribe(&[
            EventType::PaneUpdate,  // ペイン情報の更新
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(pane_manifest) => {
                // ペイン一覧を更新
                self.panes = pane_manifest
                    .panes
                    .values()
                    .flat_map(|tab_panes| tab_panes.iter())
                    .map(|p| PaneInfo {
                        id: p.id,
                        name: Some(p.title.clone()),
                        is_focused: p.is_focused,
                    })
                    .collect();
                true
            }
            _ => false,
        }
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        // pipe名で処理を分岐
        match pipe_message.name.as_str() {
            "send_keys" => {
                self.handle_send_keys(&pipe_message.payload);
            }
            "list_panes" => {
                self.handle_list_panes();
            }
            _ => {
                eprintln!("Unknown pipe command: {}", pipe_message.name);
            }
        }
        true
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!("=== Zellij MultiAgent Plugin ===");
        println!();

        if let Some(msg) = &self.last_message {
            println!("Last message: {}", msg);
        }

        println!();
        println!("Available panes ({}):", self.panes.len());
        for pane in &self.panes {
            let focused = if pane.is_focused { " [focused]" } else { "" };
            let name = pane.name.as_deref().unwrap_or("(unnamed)");
            println!("  - ID: {}, Name: {}{}", pane.id, name, focused);
        }

        println!();
        println!("Usage:");
        println!("  zellij action pipe --plugin file:zellij-multiagent.wasm \\");
        println!("    --name send_keys -- '{{\"pane_id\": 1, \"text\": \"hello\", \"send_enter\": true}}'");
    }
}

impl State {
    fn handle_send_keys(&mut self, payload: &Option<String>) {
        let Some(payload) = payload else {
            eprintln!("send_keys: No payload provided");
            return;
        };

        let msg: SendKeysMessage = match serde_json::from_str(payload) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("send_keys: Failed to parse JSON: {}", e);
                return;
            }
        };

        self.last_message = Some(format!(
            "Sending '{}' to pane {} (enter: {})",
            msg.text, msg.pane_id, msg.send_enter
        ));

        // 指定ペインにテキストを送信
        let pane_id = PaneId::Terminal(msg.pane_id);
        write_chars_to_pane_id(&msg.text, pane_id);

        // Enterキーを送信
        if msg.send_enter {
            // Enter = '\r' (0x0D)
            write_to_pane_id(vec![0x0D], pane_id);
        }

        eprintln!(
            "send_keys: Sent '{}' to pane {} (enter: {})",
            msg.text, msg.pane_id, msg.send_enter
        );
    }

    fn handle_list_panes(&self) {
        // ペイン一覧をJSON形式で出力
        let panes_json = serde_json::to_string_pretty(&self.panes).unwrap_or_default();
        println!("{}", panes_json);
    }
}
