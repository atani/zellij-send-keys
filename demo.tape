Output demo.gif

Set Shell "bash"
Set FontSize 18
Set Width 900
Set Height 500
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms

# Compare tmux vs zellij-send-keys
Sleep 0.5s
Type "# tmux"
Enter
Sleep 0.5s
Type `tmux send-keys -t 0 "echo hello" Enter`
Enter
Sleep 1.5s

Type "# zellij-send-keys"
Enter
Sleep 0.5s
Type `send-to-pane 0 "echo hello"`
Enter
Sleep 2s

# Show setup
Type "# Setup"
Enter
Sleep 0.5s
Type "source scripts/setup-env.sh"
Enter
Sleep 1.5s

# Demo: send commands to different panes
Type "# Send to pane 1"
Enter
Sleep 0.5s
Type `send-to-pane 1 "npm run build"`
Enter
Sleep 1.5s

Type "# Send to pane 2"
Enter
Sleep 0.5s
Type `send-to-pane 2 "npm run test"`
Enter
Sleep 2s

Type "# Works from outside zellij too"
Enter
Sleep 0.5s
Type `ZELLIJ_SESSION_NAME=dev send-to-pane 0 "deploy.sh"`
Enter
Sleep 3s
