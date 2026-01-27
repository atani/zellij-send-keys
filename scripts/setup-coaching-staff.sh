#!/bin/bash
# Source this file to set up environment for coaching-staff layout
#
# Usage: source ./setup-coaching-staff.sh [session_name]
#
# Pane IDs (coaching-staff.kdl):
#   0: coach
#   1: assistant
#   2: tactician
#   3: tester
#   4: worker-a
#   5: worker-b

# Auto-detect session if not provided
if [ -n "$1" ]; then
    export ZELLIJ_SESSION="$1"
else
    ZELLIJ_SESSION=$(zellij list-sessions 2>/dev/null | grep -v EXITED | head -1 | awk '{print $1}')
    if [ -z "$ZELLIJ_SESSION" ]; then
        echo "No active zellij session found. Start one first:"
        echo "  zellij -l examples/coaching-staff.kdl"
        return 1
    fi
    export ZELLIJ_SESSION
fi

export ZELLIJ_PLUGIN="file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm"

# Helper function for sending to any pane
_send_to_pane() {
    local pane_id="$1"
    local text="$2"
    ZELLIJ_SESSION_NAME="$ZELLIJ_SESSION" zellij action pipe \
        --plugin "$ZELLIJ_PLUGIN" \
        --name send_keys \
        -- "{\"pane_id\": ${pane_id}, \"text\": \"${text}\", \"send_enter\": true}"
}

# Role-specific functions
send-to-coach() {
    _send_to_pane 0 "$1"
}

send-to-assistant() {
    _send_to_pane 1 "$1"
}

send-to-tactician() {
    _send_to_pane 2 "$1"
}

send-to-tester() {
    _send_to_pane 3 "$1"
}

send-to-worker-a() {
    _send_to_pane 4 "$1"
}

send-to-worker-b() {
    _send_to_pane 5 "$1"
}

# Generic pane function
send-to-pane() {
    _send_to_pane "$1" "$2"
}

echo "Environment set up for session: $ZELLIJ_SESSION"
echo ""
echo "Coaching Staff Commands:"
echo "  send-to-coach     \"msg\"  - Coach (pane 0)"
echo "  send-to-assistant \"msg\"  - Assistant (pane 1)"
echo "  send-to-tactician \"msg\"  - Tactician (pane 2)"
echo "  send-to-tester    \"msg\"  - Tester (pane 3)"
echo "  send-to-worker-a  \"msg\"  - Worker A (pane 4)"
echo "  send-to-worker-b  \"msg\"  - Worker B (pane 5)"
echo "  send-to-pane <id> \"msg\"  - Any pane by ID"
