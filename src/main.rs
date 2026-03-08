use std::collections::{BTreeMap, VecDeque};
use zellij_send_keys::{parse_send_keys_message, serialize_panes, PaneInfo, PaneType};
use zellij_tile::prelude::*;

/// プラグインの状態
#[derive(Default)]
struct State {
    /// 最後に処理したメッセージ（デバッグ用）
    last_message: Option<String>,
    /// ペイン一覧のキャッシュ（plugin/suppressedを含む全ペイン）
    panes: Vec<PaneInfo>,
    /// Enter送信待ちキュー（pane_id）
    pending_enter: VecDeque<PaneId>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        // 必要なパーミッションを要求
        request_permission(&[
            PermissionType::WriteToStdin,         // ペインへの書き込み
            PermissionType::ReadApplicationState, // ペイン一覧の取得
        ]);

        // イベントを購読
        subscribe(&[
            EventType::PaneUpdate, // ペイン情報の更新
            EventType::Timer,      // 遅延Enter送信用
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(pane_manifest) => {
                // suppressedペインは除外、plugin/terminalは両方保持
                self.panes = pane_manifest
                    .panes
                    .values()
                    .flat_map(|tab_panes| tab_panes.iter())
                    .filter(|p| !p.is_suppressed)
                    .map(|p| PaneInfo {
                        id: p.id,
                        name: Some(p.title.clone()),
                        is_focused: p.is_focused,
                        is_plugin: p.is_plugin,
                        is_suppressed: p.is_suppressed,
                    })
                    .collect();
                true
            }
            Event::Timer(_elapsed) => {
                // 遅延Enter送信
                if let Some(pane_id) = self.pending_enter.pop_front() {
                    write_to_pane_id(vec![b'\r'], pane_id);
                }
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

        // terminalペインのみ表示
        let terminal_panes: Vec<_> = self.panes.iter().filter(|p| !p.is_plugin).collect();
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
        println!("    --name send_keys -- '{{\"pane_id\": 1, \"text\": \"hello\", \"send_enter\": true}}'");
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

        // ペインの存在チェック（IDとタイプの両方を確認）
        if !self
            .panes
            .iter()
            .any(|p| p.matches(msg.pane_id, msg.pane_type))
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

        // ペインタイプに応じてPaneIdを構築
        let pane_id = match msg.pane_type {
            PaneType::Plugin => PaneId::Plugin(msg.pane_id),
            PaneType::Terminal => PaneId::Terminal(msg.pane_id),
        };

        // テキストを送信
        write_to_pane_id(msg.text.as_bytes().to_vec(), pane_id);

        // Enterはタイマーで遅延送信（テキストが確実に処理された後に送る）
        if msg.send_enter {
            self.pending_enter.push_back(pane_id);
            set_timeout(0.2); // 200ms後にEnter送信
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
