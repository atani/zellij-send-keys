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

## Usage

### Send text to a pane

```bash
# Send "echo hello" to pane 1 and press Enter
zellij action pipe \
  --plugin file:~/.config/zellij/plugins/zellij-send-keys.wasm \
  --name send_keys \
  -- '{"pane_id": 1, "text": "echo hello", "send_enter": true}'

# Send text without Enter
zellij action pipe \
  --plugin file:~/.config/zellij/plugins/zellij-send-keys.wasm \
  --name send_keys \
  -- '{"pane_id": 1, "text": "partial text"}'
```

### Comparison with tmux

| tmux | zellij-send-keys |
|------|------------------|
| `tmux send-keys -t 1 "echo hello" Enter` | `zellij action pipe --plugin file:...wasm --name send_keys -- '{"pane_id":1,"text":"echo hello","send_enter":true}'` |

## API

### send_keys

Send text to the STDIN of a specific pane.

**Parameters:**
- `pane_id` (u32): Target terminal pane ID
- `text` (string): Text to send
- `send_enter` (bool, optional): Whether to send Enter key after text (default: false)

**Example:**
```json
{"pane_id": 1, "text": "npm run build", "send_enter": true}
```

### list_panes

Display a list of available panes with their IDs.

```bash
zellij action pipe \
  --plugin file:~/.config/zellij/plugins/zellij-send-keys.wasm \
  --name list_panes
```

## Use Case: Multi-Agent AI System

This plugin enables building a hierarchical AI agent system in zellij, similar to what's possible with tmux.

### Example Layout (Corporate Structure)

```
┌─────────────────────────────────────────────┐
│ [CEO]        │ Strategic planning           │
├──────────────┼──────────────────────────────┤
│ [Manager]    │ Task breakdown & delegation  │
├──────────────┴──────────────────────────────┤
│ [Workers]    ┌────────┬────────┬────────┐   │
│              │Worker 1│Worker 2│Worker 3│   │
│              └────────┴────────┴────────┘   │
└─────────────────────────────────────────────┘
```

See `examples/corporate.kdl` for a ready-to-use layout.

### Start the layout

```bash
zellij -l ~/.config/zellij/layouts/corporate.kdl
```

### Send commands between agents

```bash
# CEO sends task to Manager
zellij action pipe \
  --plugin file:~/.config/zellij/plugins/zellij-send-keys.wasm \
  --name send_keys \
  -- '{"pane_id": 2, "text": "/task Analyze the codebase", "send_enter": true}'
```

## License

MIT
