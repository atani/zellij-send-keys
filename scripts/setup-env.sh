#!/bin/bash
# Source this file to set up environment for zellij-send-keys
#
# Usage: source ./setup-env.sh [session_name]
# Example: source ./setup-env.sh my-session

# Requires jq
command -v jq >/dev/null 2>&1 || { echo "Error: jq is required but not installed."; return 1; }

# Auto-detect session if not provided
if [ -n "$1" ]; then
    export ZELLIJ_SESSION_NAME="$1"
else
    # Get the first active (non-exited) session
    local_session=$(zellij list-sessions 2>/dev/null | grep -v EXITED | head -1 | awk '{print $1}')
    if [ -z "$local_session" ]; then
        echo "No active zellij session found. Start zellij first."
        return 1
    fi
    export ZELLIJ_SESSION_NAME="$local_session"
fi

export ZELLIJ_PLUGIN="file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm"

# Helper function for sending to any pane (using jq for safe JSON encoding)
send-to-pane() {
    if [ $# -lt 2 ]; then
        echo "Usage: send-to-pane <pane_id> <text> [send_enter]"
        return 1
    fi

    local pane_id="$1"
    local text="$2"
    local send_enter="${3:-true}"

    # Validate pane_id is a non-negative integer
    if ! [[ "$pane_id" =~ ^[0-9]+$ ]]; then
        echo "Error: pane_id must be a non-negative integer (got: '$pane_id')"
        return 1
    fi

    # Validate send_enter
    if [ "$send_enter" != "true" ] && [ "$send_enter" != "false" ]; then
        echo "Error: send_enter must be 'true' or 'false' (got: '$send_enter')"
        return 1
    fi

    local json_payload
    json_payload=$(jq -cn --argjson pane_id "$pane_id" --arg text "$text" --argjson send_enter "$send_enter" \
        '{pane_id: $pane_id, text: $text, send_enter: $send_enter}')
    ZELLIJ_SESSION_NAME="$ZELLIJ_SESSION_NAME" zellij action pipe \
        --plugin "$ZELLIJ_PLUGIN" \
        --name send_keys \
        -- "$json_payload"
}

echo "Environment set up for session: $ZELLIJ_SESSION_NAME"
echo ""
echo "Available commands:"
echo "  send-to-pane <id> \"message\"           - Send text to pane and press Enter"
echo "  send-to-pane <id> \"message\" false     - Send text without Enter"
