# Before/After Comparison Demo
# Showing the productivity difference

Output demos/caro-before-after.gif

Require ./target/release/cmdai

Set FontSize 20
Set Width 1600
Set Height 900
Set Padding 30
Set Theme "Nord"
Set WindowBar ColorfulRight
Set BorderRadius 12
Set TypingSpeed 50ms
Set PlaybackSpeed 1.2

Hide
Type "clear"
Enter
Show

Sleep 1s

# Title
Type@30ms "╔══════════════════════════════════════════════════════════════╗"
Enter
Type@30ms "║                                                              ║"
Enter
Type@30ms "║          Caro.sh: Before vs After                           ║"
Enter
Type@30ms "║                                                              ║"
Enter
Type@30ms "╚══════════════════════════════════════════════════════════════╝"
Enter
Sleep 2s

# BEFORE scenario
Enter
Type@40ms "# 😰 BEFORE Caro: The Old Way"
Enter
Sleep 1s
Type@60ms "# 1. Need to find large files..."
Enter
Sleep 500ms
Type@60ms "# 2. Open browser"
Enter
Sleep 500ms
Type@60ms "# 3. Google 'find files larger than 10MB unix'"
Enter
Sleep 500ms
Type@60ms "# 4. Click Stack Overflow"
Enter
Sleep 500ms
Type@60ms "# 5. Copy command"
Enter
Sleep 500ms
Type@60ms "# 6. Paste and hope it works on macOS"
Enter
Sleep 500ms
Type@60ms "# 7. Get error because it's Linux syntax"
Enter
Sleep 500ms
Type@60ms "# 8. Google again 'find files macos'"
Enter
Sleep 500ms
Type@60ms "# 9. Finally find the right command"
Enter
Sleep 1s
Type@40ms "#"
Enter
Type@40ms "# ⏱️  Time wasted: 5-10 minutes"
Enter
Sleep 3s

# Clear
Hide
Type "clear"
Enter
Show

# AFTER scenario
Type@30ms "╔══════════════════════════════════════════════════════════════╗"
Enter
Type@30ms "║                                                              ║"
Enter
Type@30ms "║          Caro.sh: Before vs After                           ║"
Enter
Type@30ms "║                                                              ║"
Enter
Type@30ms "╚══════════════════════════════════════════════════════════════╝"
Enter
Sleep 1s

Enter
Type@40ms "# 😎 AFTER Caro: The New Way"
Enter
Sleep 1s
Type@80ms "./target/release/cmdai "
Type@120ms '"find large files in current directory"'
Enter
Sleep 3s
Type@40ms "# → find . -type f -size +10M"
Enter
Sleep 1s
Type@40ms "#"
Enter
Type@40ms "# ⏱️  Time: 3 seconds"
Enter
Sleep 2s

# Comparison table
Enter
Type@30ms "╔═══════════════════════════════════════════════════════════════╗"
Enter
Type@30ms "║                                                               ║"
Enter
Type@30ms "║  Before:  5-10 minutes   🐌                                  ║"
Enter
Type@30ms "║  After:   3 seconds      ⚡                                  ║"
Enter
Type@30ms "║                                                               ║"
Enter
Type@30ms "║  Productivity gain: 99% faster                               ║"
Enter
Type@30ms "║                                                               ║"
Enter
Type@30ms "╚═══════════════════════════════════════════════════════════════╝"
Enter
Sleep 3s

# More examples rapid fire
Enter
Type@40ms "# More examples in seconds:"
Enter
Sleep 1s

Type@60ms "./target/release/cmdai "
Type@100ms '"show system uptime"'
Enter
Sleep 2s

Type@60ms "./target/release/cmdai "
Type@100ms '"archive current directory"'
Enter
Sleep 2s

Type@60ms "./target/release/cmdai "
Type@100ms '"find rust files modified today"'
Enter
Sleep 2s

Type@60ms "./target/release/cmdai "
Type@100ms '"show top CPU processes"'
Enter
Sleep 2s

# Final stats
Enter
Type@30ms "╔═══════════════════════════════════════════════════════════════╗"
Enter
Type@30ms "║                                                               ║"
Enter
Type@30ms "║  💰 ROI for a 100-engineer team:                            ║"
Enter
Type@30ms "║                                                               ║"
Enter
Type@30ms "║  • 10-20 commands per day                                    ║"
Enter
Type@30ms "║  • 5 min → 5 sec per command                                ║"
Enter
Type@30ms "║  • = 50-100 min saved per engineer daily                    ║"
Enter
Type@30ms "║  • = $1.5M - $3M annual productivity gain                   ║"
Enter
Type@30ms "║                                                               ║"
Enter
Type@30ms "╚═══════════════════════════════════════════════════════════════╝"
Enter
Sleep 3s

# CTA
Enter
Type@40ms "# Stop googling. Start shipping."
Enter
Sleep 1s
Type@40ms "# github.com/wildcard/caro.sh"
Enter
Sleep 3s
