# Before/After Comparison Demo
# Showing the productivity difference

Output caro-before-after.gif

Set FontSize 20
Set Width 1600
Set Height 900
Set Padding 30
Set Theme "Nord"
Set WindowBar ColorfulRight
Set BorderRadius 12
Set TypingSpeed 15ms
Set PlaybackSpeed 2.0

Hide
Type "clear"
Enter
Type "source ~/.zshrc"
Enter
Type "cd demos/playground"
Enter
Show

Sleep 500ms

# Title
Type@10ms "╔══════════════════════════════════════════════════════════════╗"
Enter
Type@10ms "║                                                              ║"
Enter
Type@10ms "║          Caro.sh: Before vs After                           ║"
Enter
Type@10ms "║                                                              ║"
Enter
Type@10ms "╚══════════════════════════════════════════════════════════════╝"
Enter
Sleep 1s

# BEFORE scenario
Enter
Type@20ms "# 😰 BEFORE Caro: The Old Way"
Enter
Sleep 500ms
Type@30ms "# 1. Need to find large files..."
Enter
Sleep 200ms
Type@30ms "# 2. Open browser"
Enter
Sleep 200ms
Type@30ms "# 3. Google 'find files larger than 10MB unix'"
Enter
Sleep 200ms
Type@30ms "# 4. Click Stack Overflow"
Enter
Sleep 200ms
Type@30ms "# 5. Copy command"
Enter
Sleep 200ms
Type@30ms "# 6. Paste and hope it works on macOS"
Enter
Sleep 200ms
Type@30ms "# 7. Get error because it's Linux syntax"
Enter
Sleep 200ms
Type@30ms "# 8. Google again 'find files macos'"
Enter
Sleep 200ms
Type@30ms "# 9. Finally find the right command"
Enter
Sleep 500ms
Type@20ms "#"
Enter
Type@20ms "# ⏱️  Time wasted: 5-10 minutes"
Enter
Sleep 1.5s

# Clear
Hide
Type "clear"
Enter
Show

# AFTER scenario
Type@10ms "╔══════════════════════════════════════════════════════════════╗"
Enter
Type@10ms "║                                                              ║"
Enter
Type@10ms "║          Caro.sh: Before vs After                           ║"
Enter
Type@10ms "║                                                              ║"
Enter
Type@10ms "╚══════════════════════════════════════════════════════════════╝"
Enter
Sleep 500ms

Enter
Type@20ms "# 😎 AFTER Caro: The New Way"
Enter
Sleep 500ms
Type@30ms "caro "
Type@40ms '"find large files in current directory"'
Enter
Sleep 2s
Type@20ms "# → find . -type f -size +10M"
Enter
Sleep 500ms
Type@20ms "#"
Enter
Type@20ms "# ⏱️  Time: 3 seconds"
Enter
Sleep 1s

# Comparison table
Enter
Type@10ms "╔═══════════════════════════════════════════════════════════════╗"
Enter
Type@10ms "║                                                               ║"
Enter
Type@10ms "║  Before:  5-10 minutes   🐌                                  ║"
Enter
Type@10ms "║  After:   3 seconds      ⚡                                  ║"
Enter
Type@10ms "║                                                               ║"
Enter
Type@10ms "║  Productivity gain: 99% faster                               ║"
Enter
Type@10ms "║                                                               ║"
Enter
Type@10ms "╚═══════════════════════════════════════════════════════════════╝"
Enter
Sleep 1.5s

# More examples rapid fire
Enter
Type@20ms "# More examples in seconds:"
Enter
Sleep 500ms

Type@30ms "caro "
Type@40ms '"show system uptime"'
Enter
Sleep 1.5s

Type@30ms "caro "
Type@40ms '"find JavaScript files"'
Enter
Sleep 1.5s

Type@30ms "caro "
Type@40ms '"count lines in source"'
Enter
Sleep 1.5s

# Final stats
Enter
Type@10ms "╔═══════════════════════════════════════════════════════════════╗"
Enter
Type@10ms "║                                                               ║"
Enter
Type@10ms "║  💰 ROI for a 100-engineer team:                            ║"
Enter
Type@10ms "║                                                               ║"
Enter
Type@10ms "║  • 10-20 commands per day                                    ║"
Enter
Type@10ms "║  • 5 min → 5 sec per command                                ║"
Enter
Type@10ms "║  • = 50-100 min saved per engineer daily                    ║"
Enter
Type@10ms "║  • = $1.5M - $3M annual productivity gain                   ║"
Enter
Type@10ms "║                                                               ║"
Enter
Type@10ms "╚═══════════════════════════════════════════════════════════════╝"
Enter
Sleep 1.5s

# CTA
Enter
Type@20ms "# Stop googling. Start shipping."
Enter
Sleep 500ms
Type@20ms "# github.com/wildcard/caro.sh"
Enter
Sleep 2s
