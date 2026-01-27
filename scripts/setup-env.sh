#!/bin/bash
# Source this file to set up environment for zellij-send-keys
#
# Usage: source ./setup-env.sh [session_name]
# Example: source ./setup-env.sh profound-clarinet

# Auto-detect session if not provided
if [ -n "$1" ]; then
    export ZELLIJ_SESSION="$1"
else
    # Get the first active (non-exited) session
    ZELLIJ_SESSION=$(zellij list-sessions 2>/dev/null | grep -v EXITED | head -1 | awk '{print $1}')
    if [ -z "$ZELLIJ_SESSION" ]; then
        echo "No active zellij session found. Start one first:"
        echo "  zellij -l soccer-team.kdl"
        return 1
    fi
    export ZELLIJ_SESSION
fi

export ZELLIJ_PLUGIN="file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm"

# Helper functions
send-to-coach() {
    zellij -s "$ZELLIJ_SESSION" action pipe \
        --plugin "$ZELLIJ_PLUGIN" \
        --name send_keys \
        -- "{\"pane_id\": 0, \"text\": \"$1\", \"send_enter\": true}"
}

send-to-captain() {
    zellij -s "$ZELLIJ_SESSION" action pipe \
        --plugin "$ZELLIJ_PLUGIN" \
        --name send_keys \
        -- "{\"pane_id\": 1, \"text\": \"$1\", \"send_enter\": true}"
}

send-to-pane() {
    local pane_id="$1"
    local text="$2"
    zellij -s "$ZELLIJ_SESSION" action pipe \
        --plugin "$ZELLIJ_PLUGIN" \
        --name send_keys \
        -- "{\"pane_id\": ${pane_id}, \"text\": \"${text}\", \"send_enter\": true}"
}

echo "Environment set up for session: $ZELLIJ_SESSION"
echo ""
echo "Available commands:"
echo "  send-to-coach \"your message\"    - Send to Coach (pane 0)"
echo "  send-to-captain \"your message\"  - Send to Captain (pane 1)"
echo "  send-to-pane <id> \"message\"     - Send to specific pane"
