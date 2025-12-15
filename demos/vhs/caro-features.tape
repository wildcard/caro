# Caro.sh Feature Showcase Demo
# Inspired by Atuin and Crush demos

Output caro-features.gif

Set FontSize 20
Set Width 1600
Set Height 900
Set Padding 30
Set Theme "Catppuccin-mocha"
Set WindowBar ColorfulRight
Set BorderRadius 12
Set TypingSpeed 20ms
Set PlaybackSpeed 2.0

Hide
Type "clear"
Enter
Type "source ~/.zshrc"
Enter
Type "cd demos/sysadmin-playground"
Enter
Show

Sleep 500ms

# Intro banner
Type@10ms "╔═══════════════════════════════════════════════════════════╗"
Enter
Type@10ms "║                                                           ║"
Enter
Type@10ms "║        Caro.sh - Your Terminal's AI Companion            ║"
Enter
Type@10ms "║                                                           ║"
Enter
Type@10ms "║    Local • Private • Fast • Community-Driven             ║"
Enter
Type@10ms "║                                                           ║"
Enter
Type@10ms "╚═══════════════════════════════════════════════════════════╝"
Enter
Enter
Sleep 1s

# Feature 1: Log Analysis
Type@20ms "# 🔍 Log Analysis"
Enter
Sleep 300ms
Type@30ms "caro "
Type@40ms '"find all errors in application logs"'
Enter
Sleep 2s
Type "grep ERROR logs/app.log"
Enter
Sleep 1s
Enter

# Feature 2: System Monitoring
Type@20ms "# 🏃 System Monitoring"
Enter
Sleep 300ms
Type@30ms "caro "
Type@40ms '"show disk usage"'
Enter
Sleep 2s
Type "df -h"
Enter
Sleep 1s
Enter

# Feature 3: File Operations
Type@20ms "# 📁 File Operations"
Enter
Sleep 300ms
Type@30ms "caro "
Type@40ms '"list backup files sorted by size"'
Enter
Sleep 2s
Type "ls -lhS backups/"
Enter
Sleep 1s
Enter

# Stats banner
Type@10ms "╔═══════════════════════════════════════════════════════════╗"
Enter
Type@10ms "║                                                           ║"
Enter
Type@10ms "║  ✅ 87% success rate • 🚀 Sub-second inference          ║"
Enter
Type@10ms "║  💾 3.9MB binary • 🔒 52 safety patterns               ║"
Enter
Type@10ms "║  🏠 Local-first • 🆓 Free forever                      ║"
Enter
Type@10ms "║                                                           ║"
Enter
Type@10ms "╚═══════════════════════════════════════════════════════════╝"
Enter
Sleep 1s

# Call to action
Enter
Type@20ms "# 🌟 Get Started:"
Enter
Sleep 300ms
Type@20ms "# git clone https://github.com/wildcard/caro.sh"
Enter
Sleep 300ms
Type@20ms "# cargo build --release --features embedded-mlx"
Enter
Sleep 300ms
Type@20ms "# caro "
Type@30ms '"your command here"'
Enter
Sleep 2s
