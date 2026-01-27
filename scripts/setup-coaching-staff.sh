#!/bin/bash
# Source this file to set up environment for coaching-staff layout
#
# Usage: source ./setup-coaching-staff.sh [session_name]
#
# Pane IDs (coaching-staff.kdl):
#   0: coach       - チーム統括のスペシャリスト
#   1: reviewer    - コードレビューのエキスパート
#   2: tactician   - ソフトウェアアーキテクトの達人
#   3: qa-lead     - 品質保証のプロフェッショナル
#   4: coordinator - プロジェクト調整の専門家
#   5: tester      - テスト実装のスペシャリスト
#   6: worker-a    - 実装のエキスパート
#   7: worker-b    - 実装のエキスパート

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

# Helper function for sending to any pane (using jq for safe JSON encoding)
_send_to_pane() {
    local pane_id="$1"
    local text="$2"
    local json_payload
    json_payload=$(jq -cn --argjson pane_id "$pane_id" --arg text "$text" \
        '{pane_id: $pane_id, text: $text, send_enter: true}')
    ZELLIJ_SESSION_NAME="$ZELLIJ_SESSION" zellij action pipe \
        --plugin "$ZELLIJ_PLUGIN" \
        --name send_keys \
        -- "$json_payload"
}

# Role-specific functions
send-to-coach() {
    _send_to_pane 0 "$1"
}

send-to-reviewer() {
    _send_to_pane 1 "$1"
}

send-to-tactician() {
    _send_to_pane 2 "$1"
}

send-to-qa-lead() {
    _send_to_pane 3 "$1"
}

send-to-coordinator() {
    _send_to_pane 4 "$1"
}

send-to-tester() {
    _send_to_pane 5 "$1"
}

send-to-worker-a() {
    _send_to_pane 6 "$1"
}

send-to-worker-b() {
    _send_to_pane 7 "$1"
}

# Generic pane function
send-to-pane() {
    _send_to_pane "$1" "$2"
}

echo "Environment set up for session: $ZELLIJ_SESSION"
echo ""
echo "Coaching Staff Commands (8-person team):"
echo "  send-to-coach       \"msg\"  - Coach (pane 0)"
echo "  send-to-reviewer    \"msg\"  - Reviewer (pane 1)"
echo "  send-to-tactician   \"msg\"  - Tactician (pane 2)"
echo "  send-to-qa-lead     \"msg\"  - QA Lead (pane 3)"
echo "  send-to-coordinator \"msg\"  - Coordinator (pane 4)"
echo "  send-to-tester      \"msg\"  - Tester (pane 5)"
echo "  send-to-worker-a    \"msg\"  - Worker A (pane 6)"
echo "  send-to-worker-b    \"msg\"  - Worker B (pane 7)"
echo "  send-to-pane <id>   \"msg\"  - Any pane by ID"
