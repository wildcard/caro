# Ultra-fast demo for social media
# 30 second version for Twitter/LinkedIn

Output caro-social.gif

Require ../target/release/cmdai

Set FontSize 24
Set Width 1200
Set Height 700
Set Padding 25
Set Theme "Catppuccin Mocha"
Set WindowBar Rings
Set BorderRadius 10
Set TypingSpeed 40ms
Set PlaybackSpeed 1.5

Hide
Type "clear"
Enter
Show

Sleep 500ms

# Quick title
Type@20ms "# Caro.sh - Your Terminal's AI Companion"
Enter
Sleep 1s

# Rapid fire demos
Type@40ms "./target/release/cmdai "
Type@60ms '"list all files"'
Enter
Sleep 2s

Type@40ms "./target/release/cmdai "
Type@60ms '"show system uptime"'
Enter
Sleep 2s

Type@40ms "./target/release/cmdai "
Type@60ms '"find rust files"'
Enter
Sleep 2s

Type@40ms "./target/release/cmdai "
Type@60ms '"archive directory"'
Enter
Sleep 2s

Type@40ms "./target/release/cmdai "
Type@60ms '"top CPU processes"'
Enter
Sleep 2s

# Quick stats
Sleep 500ms
Type@30ms "# ⚡ <1s • 💾 3.9MB • 🔒 Safe • 🆓 Free"
Enter
Sleep 1s
Type@30ms "# ⭐ github.com/wildcard/caro.sh"
Enter
Sleep 2s
