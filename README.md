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
source scripts/setup-coaching-staff.sh

# Or specify session name
source scripts/setup-coaching-staff.sh my-session

# Now use simple commands
send-to-coach "Analyze the codebase"
send-to-reviewer "Review the diff"
send-to-tactician "Draft an API plan"
send-to-tester "Run the tests"
send-to-pane 6 "echo hello"
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

An 8-person team of specialists, organized into tabs:

```
Tabs:
- Coach
- Reviewer
- Strategy (Tactician | QA Lead)
- Operations (Coordinator | Tester)
- Workers (Worker A | Worker B)
- VAR (plugin)
```

```bash
# 1. Start the layout
zellij -l examples/coaching-staff.kdl

# 2. Set up environment
source scripts/setup-coaching-staff.sh

# 3. Send commands
send-to-coach "Review the PR"
send-to-reviewer "Scan for risky changes"
send-to-tactician "Design the API"
send-to-qa-lead "Define test strategy"
send-to-tester "Write tests"
send-to-worker-a "Implement authentication"
```

| Pane ID | Role | Responsibility |
|---------|------|----------------|
| 0 | Coach | Lead specialist, final decision |
| 1 | Reviewer | Code review specialist, design guardrails |
| 2 | Tactician | Architecture specialist, design strategy |
| 3 | QA Lead | Quality specialist, release criteria |
| 4 | Coordinator | Coordination specialist, blockers |
| 5 | Tester | Testing specialist, QA execution |
| 6 | Worker A | Implementation specialist |
| 7 | Worker B | Implementation specialist |

Balance rationale: one lead specialist, two implementation specialists, and dedicated specialists for review, architecture, quality, coordination, and testing. This keeps decision flow clear while preserving parallel execution and quality control.

### Quick Start (Coaching Staff)

```bash
# 1. Start the layout
zellij -l examples/coaching-staff.kdl

# 2. In another terminal, set up the environment
source scripts/setup-coaching-staff.sh

# 3. Send commands to agents
send-to-coach "Summarize the codebase"
send-to-tactician "Draft an API plan"
send-to-worker-a "Start the implementation"
```

## License

MIT
