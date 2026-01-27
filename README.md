# zellij-multiagent

zellijでマルチエージェント環境を構築するためのプラグイン。
tmuxの`send-keys`に相当する機能を提供します。

## 機能

- **send_keys**: 指定したペインにテキストを送信
- **list_panes**: ペイン一覧を取得

## ビルド

```bash
# wasm32-wasip1ターゲットを追加（初回のみ）
rustup target add wasm32-wasip1

# ビルド
cargo build --target wasm32-wasip1 --release
```

## 使用方法

### 1. プラグインをインストール

```bash
# zellijのプラグインディレクトリにコピー
mkdir -p ~/.config/zellij/plugins
cp target/wasm32-wasip1/release/zellij-multiagent.wasm ~/.config/zellij/plugins/
```

### 2. プラグインを起動

```bash
# フローティングペインとして起動
zellij plugin -- file:~/.config/zellij/plugins/zellij-multiagent.wasm
```

### 3. コマンドを送信

```bash
# 指定ペインにテキストを送信（Enter付き）
zellij action pipe \
  --plugin file:~/.config/zellij/plugins/zellij-multiagent.wasm \
  --name send_keys \
  -- '{"pane_id": 1, "text": "echo hello", "send_enter": true}'

# Enterなしでテキストのみ送信
zellij action pipe \
  --plugin file:~/.config/zellij/plugins/zellij-multiagent.wasm \
  --name send_keys \
  -- '{"pane_id": 1, "text": "partial text"}'
```

## マルチエージェント構成例

記事「[Claude Codeでマルチエージェント構築を実現する手法](https://zenn.dev/shio_shoppaize/articles/5fee11d03a11a1)」のzellij版。

### レイアウトファイル（multiagent.kdl）

```kdl
layout {
    tab name="shogun" {
        pane name="将軍" command="claude" {
            args "--profile" "shogun"
        }
    }
    tab name="karo" {
        pane name="家老" command="claude" {
            args "--profile" "karo"
        }
    }
    tab name="ashigaru" {
        pane split_direction="vertical" {
            pane split_direction="horizontal" {
                pane name="足軽1" command="claude"
                pane name="足軽2" command="claude"
            }
            pane split_direction="horizontal" {
                pane name="足軽3" command="claude"
                pane name="足軽4" command="claude"
            }
        }
    }
}
```

### 起動

```bash
zellij -l multiagent.kdl
```

## API

### send_keys

指定したペインにテキストを送信します。

**パラメータ:**
- `pane_id` (u32): 送信先ターミナルペインのID
- `text` (string): 送信するテキスト
- `send_enter` (bool, optional): Enterキーを送信するか（デフォルト: false）

**例:**
```json
{"pane_id": 1, "text": "/task 調査開始", "send_enter": true}
```

### list_panes

現在のペイン一覧を表示します。

```bash
zellij action pipe \
  --plugin file:~/.config/zellij/plugins/zellij-multiagent.wasm \
  --name list_panes
```

## ライセンス

MIT
