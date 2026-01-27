#!/bin/bash
# Send a task to an agent via YAML file and zellij-send-keys plugin
#
# Usage: ./send-task.sh <session> <pane_id> <task_description>
# Example: ./send-task.sh profound-clarinet 0 "Analyze the codebase structure"

set -e

SESSION="${1:?Usage: $0 <session> <pane_id> <task_description>}"
PANE_ID="${2:?Usage: $0 <session> <pane_id> <task_description>}"
TASK="${3:?Usage: $0 <session> <pane_id> <task_description>}"

PLUGIN="file:$HOME/.config/zellij/plugins/zellij-send-keys.wasm"
TASKS_DIR="${TASKS_DIR:-$HOME/.local/share/zellij-multiagent/tasks}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TASK_FILE="$TASKS_DIR/task_${TIMESTAMP}_pane${PANE_ID}.yaml"

# Create tasks directory if not exists
mkdir -p "$TASKS_DIR"

# Write task to YAML file
cat > "$TASK_FILE" << EOF
id: ${TIMESTAMP}_pane${PANE_ID}
created_at: $(date -Iseconds)
status: pending
from_pane: external
to_pane: ${PANE_ID}
task: |
  ${TASK}
EOF

echo "Task written to: $TASK_FILE"

# Send task to the agent via plugin
ZELLIJ_SESSION_NAME="$SESSION" zellij action pipe \
  --plugin "$PLUGIN" \
  --name send_keys \
  -- "{\"pane_id\": ${PANE_ID}, \"text\": \"${TASK}\", \"send_enter\": true}"

echo "Task sent to pane ${PANE_ID} in session ${SESSION}"
