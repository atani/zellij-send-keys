#!/bin/bash
# Source this file to set up environment for zellij-send-keys
#
# Usage: source ./setup-env.sh [session_name]
# Example: source ./setup-env.sh my-session

# Auto-detect session if not provided
if [ -n "$1" ]; then
    export ZELLIJ_SESSION="$1"
else
    # Get the first active (non-exited) session
    ZELLIJ_SESSION=$(zellij list-sessions 2>/dev/null | grep -v EXITED | head -1 | awk '{print $1}')
    if [ -z "$ZELLIJ_SESSION" ]; then
        echo "No active zellij session found. Start zellij first."
        return 1
    fi
    export ZELLIJ_SESSION
fi

export ZELLIJ_PLUGIN="file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm"

# Helper function for sending to any pane (using jq for safe JSON encoding)
send-to-pane() {
    local pane_id="$1"
    local text="$2"
    local send_enter="${3:-true}"
    local json_payload
    json_payload=$(jq -cn --argjson pane_id "$pane_id" --arg text "$text" --argjson send_enter "$send_enter" \
        '{pane_id: $pane_id, text: $text, send_enter: $send_enter}')
    ZELLIJ_SESSION_NAME="$ZELLIJ_SESSION" zellij action pipe \
        --plugin "$ZELLIJ_PLUGIN" \
        --name send_keys \
        -- "$json_payload"
}

echo "Environment set up for session: $ZELLIJ_SESSION"
echo ""
echo "Available commands:"
echo "  send-to-pane <id> \"message\"           - Send text to pane and press Enter"
echo "  send-to-pane <id> \"message\" false     - Send text without Enter"
