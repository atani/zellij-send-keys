# zellij-send-keys

A zellij plugin that provides `send-keys` functionality similar to tmux.
Send text/commands to specific panes from outside or other panes.

## Features

- **send_keys**: Send text to a specific pane by ID
- **list_panes**: Get a list of all panes with their IDs

## Installation

### From Release

Download the `.wasm` file from [Releases](https://github.com/atani/zellij-send-keys/releases) and copy it to your zellij plugins directory:

```bash
mkdir -p ~/.config/zellij/plugins
cp zellij-send-keys.wasm ~/.config/zellij/plugins/
```

### From Source

```bash
# Add wasm target (first time only)
rustup target add wasm32-wasip1

# Build
cargo build --target wasm32-wasip1 --release

# Install
cp target/wasm32-wasip1/release/zellij-send-keys.wasm ~/.config/zellij/plugins/
```

## First Time Setup: Grant Permissions

The plugin requires permissions to write to pane STDIN. On first use, you need to grant permissions:

```bash
# Inside a zellij session, run:
zellij plugin -- file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm
```

A permission dialog will appear. Click **Grant** to allow the plugin to:
- Write to pane STDIN
- Read application state (for listing panes)

> **Note**: Use `$HOME` instead of `~` in plugin paths. The tilde is not expanded in all contexts.

## Usage

### Send text to a pane

```bash
# Send "echo hello" to pane 0 and press Enter
ZELLIJ_SESSION_NAME=<session_name> zellij action pipe \
  --plugin file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm \
  --name send_keys \
  -- '{"pane_id": 0, "text": "echo hello", "send_enter": true}'

# Send text without Enter
ZELLIJ_SESSION_NAME=<session_name> zellij action pipe \
  --plugin file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm \
  --name send_keys \
  -- '{"pane_id": 0, "text": "partial text"}'
```

> **Note**: Use `ZELLIJ_SESSION_NAME` environment variable instead of `-s` flag for session targeting.

### Using Helper Scripts

Source the setup script for convenient commands:

```bash
# Set up environment (auto-detects active session)
source scripts/setup-env.sh

# Or specify session name
source scripts/setup-env.sh my-session

# Now use simple commands
send-to-coach "Analyze the codebase"
send-to-captain "Run the tests"
send-to-pane 5 "echo hello"
```

### Comparison with tmux

| tmux | zellij-send-keys |
|------|------------------|
| `tmux send-keys -t 0 "echo hello" Enter` | `send-to-pane 0 "echo hello"` |

## API

### send_keys

Send text to the STDIN of a specific pane.

**Parameters:**
- `pane_id` (u32): Target terminal pane ID
- `text` (string): Text to send
- `send_enter` (bool, optional): Whether to send Enter key after text (default: false)

**Example:**
```json
{"pane_id": 0, "text": "npm run build", "send_enter": true}
```

### list_panes

Display a list of available panes with their IDs. Launch the plugin to see the pane list:

```bash
zellij plugin -- file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm
```

## Use Case: Multi-Agent AI System

This plugin enables building a hierarchical AI agent system in zellij, similar to what's possible with tmux.

### Example Layout (Coaching Staff - Recommended)

A practical 6-person team for real development work:

```
┌─────────────────────────────────────┐
│ Coach (総監督・統合・最終判断)       │
├─────────────────────────────────────┤
│ Assistant (調整・レビュー・進捗管理) │
├──────────────────┬──────────────────┤
│ Tactician        │ Tester           │
│ (設計・戦術)      │ (テスト・QA)     │
├──────────────────┼──────────────────┤
│ Worker A         │ Worker B         │
│ (実装)           │ (実装)           │
└──────────────────┴──────────────────┘
```

```bash
# 1. Start the layout
zellij -l examples/coaching-staff.kdl

# 2. Set up environment
source scripts/setup-coaching-staff.sh

# 3. Send commands
send-to-coach "PRをレビューして"
send-to-tactician "APIの設計を考えて"
send-to-tester "テストを書いて"
send-to-worker-a "認証機能を実装して"
```

| Pane ID | Role | 役割 |
|---------|------|------|
| 0 | Coach | 総監督・最終判断 |
| 1 | Assistant | 調整・レビュー |
| 2 | Tactician | 設計・アーキテクチャ |
| 3 | Tester | テスト・QA |
| 4 | Worker A | 実装 |
| 5 | Worker B | 実装 |

### Example Layout (Soccer Team Structure)

```
┌─────────────────────────────────────────────┐
│ [Coach]      │ Tactical planning            │
├──────────────┼──────────────────────────────┤
│ [Captain]    │ Relay instructions to team   │
├──────────────┴──────────────────────────────┤
│ [Players]    ┌────┬────┬────┐  FW           │
│              ├────┼────┼────┤  MF           │
│              ├────┼────┼────┤  DF           │
│              └────┴────┴────┘  GK           │
├─────────────────────────────────────────────┤
│ [VAR]        │ Monitoring & control         │
└─────────────────────────────────────────────┘
```

See `examples/soccer-team.kdl` for a ready-to-use layout.

### Quick Start

```bash
# 1. Start the layout
zellij -l examples/soccer-team.kdl

# 2. In another terminal, set up the environment
source scripts/setup-env.sh

# 3. Send commands to agents
send-to-coach "Analyze the opponent's formation"
send-to-captain "Tell the midfielders to press higher"
```

### Pane IDs in Soccer Team Layout

| Pane ID | Role |
|---------|------|
| 0 | Coach |
| 1 | Captain |
| 2-4 | Forwards (FW) |
| 5-8 | Midfielders (MF) |
| 9-12 | Defenders (DF) |
| 13 | Goalkeeper (GK) |

## License

MIT
