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

## License

MIT
