# zellij-send-keys

[![CI](https://github.com/atani/zellij-send-keys/actions/workflows/ci.yml/badge.svg)](https://github.com/atani/zellij-send-keys/actions/workflows/ci.yml)
[![Release](https://github.com/atani/zellij-send-keys/actions/workflows/release.yml/badge.svg)](https://github.com/atani/zellij-send-keys/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/atani/zellij-send-keys)](https://github.com/atani/zellij-send-keys/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Bring tmux's `send-keys` to Zellij.** Send text and commands to any pane, from anywhere.

## Why

Zellij doesn't have a built-in `send-keys` equivalent.
If you're migrating from tmux and rely on scripted pane control, this plugin fills that gap.
CI runners, dev environment setup, AI agent orchestration -- all covered.

```bash
# tmux
tmux send-keys -t 0 "echo hello" Enter

# zellij-send-keys
send-to-pane 0 "echo hello"
```

## Features

- **send_keys** - Send text to a specific pane by ID (terminal or plugin pane)
- **list_panes** - Get a list of all panes with their IDs
- Works from outside a zellij session via `ZELLIJ_SESSION_NAME`
- Helper script for a tmux-like `send-to-pane` command

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

### Using Helper Script

Source the setup script for a convenient `send-to-pane` command:

```bash
# Set up environment (auto-detects active session)
source scripts/setup-env.sh

# Or specify session name
source scripts/setup-env.sh my-session

# Send text to pane
send-to-pane 0 "echo hello"
send-to-pane 1 "npm run build"

# Send without pressing Enter
send-to-pane 0 "partial text" false
```

## API

### send_keys

Send text to the STDIN of a specific pane.

**Parameters:**
- `pane_id` (u32): Target pane ID
- `text` (string): Text to send (max 64KB)
- `send_enter` (bool, optional): Whether to send Enter key after text (default: false)
- `pane_type` (string, optional): `"terminal"` or `"plugin"` (default: `"terminal"`)

**Example:**
```json
{"pane_id": 0, "text": "npm run build", "send_enter": true}
```

**Send to a plugin pane:**
```json
{"pane_id": 3, "text": "cmd", "pane_type": "plugin"}
```

### list_panes

Get a JSON list of available panes with their IDs via the pipe API:

```bash
ZELLIJ_SESSION_NAME=<session_name> zellij action pipe \
  --plugin file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm \
  --name list_panes
```

Or launch the plugin UI to see panes interactively:

```bash
zellij plugin -- file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm
```

## Use Cases

- **Dev environment automation** - Start servers, watchers, and build tools across panes
- **CI/CD integration** - Drive zellij sessions from external scripts
- **AI agent orchestration** - Let AI tools send commands to terminal panes
- **Pair programming** - Send commands to a colleague's pane in a shared session

## License

MIT
