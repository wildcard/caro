# SysAdmin/DevOps/SRE Demo
# Realistic operations and log analysis scenarios

Output sysadmin-demo.gif

Set FontSize 22
Set Width 1800
Set Height 1000
Set Padding 40
Set Theme "TokyoNight"
Set WindowBar Rings
Set BorderRadius 15
Set TypingSpeed 20ms
Set PlaybackSpeed 2.0

Hide
Type "clear"
Enter
Type "cd demos/sysadmin-playground"
Enter
Type "source ~/.zshrc"
Enter
Show

Sleep 500ms

# Simple banner
Type@10ms "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type@10ms "  Caro.sh - For SysAdmins, DevOps, and SREs"
Enter
Type@10ms "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Enter
Sleep 800ms

# Demo 1: Find errors in logs
Type@20ms "caro "
Type@30ms '"find all errors in application logs"'
Enter
Sleep 2s
Type "grep ERROR logs/app.log"
Enter
Sleep 1s
Enter

# Demo 2: Count failed login attempts
Type@20ms "caro "
Type@30ms '"count failed login attempts in syslog"'
Enter
Sleep 2s
Type "grep 'Failed login' logs/syslog | wc -l"
Enter
Sleep 1s
Enter

# Demo 3: Show most recent log entries
Type@20ms "caro "
Type@30ms '"show last 5 lines of nginx access log"'
Enter
Sleep 2s
Type "tail -5 logs/nginx-access.log"
Enter
Sleep 1s
Enter

# Demo 4: Find large backup files
Type@20ms "caro "
Type@30ms '"list backup files sorted by size"'
Enter
Sleep 2s
Type "ls -lhS backups/"
Enter
Sleep 1s
Enter

# Key messaging
Type@10ms "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type@10ms "  From Natural Language to Shell Commands"
Enter
Type@10ms "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 500ms
Type@20ms "# 🔍 Log analysis made simple"
Enter
Sleep 300ms
Type@20ms "# 🛡️  Built-in safety patterns"
Enter
Sleep 300ms
Type@20ms "# ⚡ Fast local inference"
Enter
Sleep 300ms
Type@20ms "# 💜 Open source (AGPL-3.0)"
Enter
Sleep 1s

Enter
Type@10ms "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Type@20ms "# ⭐ github.com/wildcard/caro.sh"
Enter
Type@10ms "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Enter
Sleep 1.5s
