# Caro.sh Feature Showcase Demo
# Inspired by Atuin and Crush demos

Output caro-features.gif

Require ../target/release/cmdai

Set FontSize 20
Set Width 1600
Set Height 900
Set Padding 30
Set Theme "Catppuccin-mocha"
Set WindowBar ColorfulRight
Set BorderRadius 12
Set TypingSpeed 60ms
Set PlaybackSpeed 1.0

Hide
Type "clear"
Enter
Show

Sleep 500ms

# Intro banner
Type@40ms "╔═══════════════════════════════════════════════════════════╗"
Enter
Type@40ms "║                                                           ║"
Enter
Type@40ms "║        Caro.sh - Your Terminal's AI Companion            ║"
Enter
Type@40ms "║                                                           ║"
Enter
Type@40ms "║    Local • Private • Fast • Community-Driven             ║"
Enter
Type@40ms "║                                                           ║"
Enter
Type@40ms "╚═══════════════════════════════════════════════════════════╝"
Enter
Enter
Sleep 2s

# Feature 1: System Monitoring
Type@50ms "# 🔍 System Monitoring"
Enter
Sleep 500ms
Type@80ms "caro "
Type@120ms '"show system uptime and load average"'
Enter
Sleep 3s
Type "uptime"
Enter
Sleep 2s
Enter

# Feature 2: Process Management
Type@50ms "# 🏃 Process Management"
Enter
Sleep 500ms
Type@80ms "caro "
Type@120ms '"show top 10 processes by CPU usage"'
Enter
Sleep 3s
Type "# Generated: ps aux | sort -nr -k 3 | head -n 10"
Enter
Sleep 2s
Enter

# Feature 3: File Operations
Type@50ms "# 📁 File Operations"
Enter
Sleep 500ms
Type@80ms "caro "
Type@120ms '"find all rust files modified in the last 7 days"'
Enter
Sleep 3s
Type "# Perfect for daily development workflow"
Enter
Sleep 2s
Enter

# Feature 4: Security Auditing
Type@50ms "# 🔒 Security Auditing"
Enter
Sleep 500ms
Type@80ms "caro "
Type@120ms '"find files with setuid bit enabled"'
Enter
Sleep 3s
Type "find /usr/bin -type f -perm -u+s 2>/dev/null | head -3"
Enter
Sleep 2s
Enter

# Feature 5: Network Diagnostics
Type@50ms "# 🌐 Network Diagnostics"
Enter
Sleep 500ms
Type@80ms "caro "
Type@120ms '"check DNS resolution for api.example.com"'
Enter
Sleep 3s
Type "# dig api.example.com +short"
Enter
Sleep 2s
Enter

# Feature 6: Archive Operations
Type@50ms "# 📦 Archive Operations"
Enter
Sleep 500ms
Type@80ms "caro "
Type@120ms '"archive current directory"'
Enter
Sleep 2s
Type "# tar czvf archive.tar.gz ./"
Enter
Sleep 2s
Enter

# Stats banner
Type@40ms "╔═══════════════════════════════════════════════════════════╗"
Enter
Type@40ms "║                                                           ║"
Enter
Type@40ms "║  ✅ 87% success rate • 🚀 Sub-second inference          ║"
Enter
Type@40ms "║  💾 3.9MB binary • 🔒 52 safety patterns               ║"
Enter
Type@40ms "║  🏠 Local-first • 🆓 Free forever                      ║"
Enter
Type@40ms "║                                                           ║"
Enter
Type@40ms "╚═══════════════════════════════════════════════════════════╝"
Enter
Sleep 2s

# Call to action
Enter
Type@50ms "# 🌟 Get Started:"
Enter
Sleep 500ms
Type@50ms "# git clone https://github.com/wildcard/caro.sh"
Enter
Sleep 500ms
Type@50ms "# cargo build --release --features embedded-mlx"
Enter
Sleep 500ms
Type@50ms "# caro "
Type@100ms '"your command here"'
Enter
Sleep 3s
