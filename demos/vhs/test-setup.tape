# Quick test tape to verify setup works
# Should run in ~10 seconds

Output test-setup.gif

Set Shell zsh
Set FontSize 20
Set Width 1400
Set Height 600
Set Padding 20
Set Theme "Dracula"
Set TypingSpeed 10ms
Set PlaybackSpeed 2.0

Hide
Type "clear"

Enter
Type "cd ./sysadmin-playground"
Enter
Show

Sleep 500ms

Type@10ms "# Testing Caro setup..."
Enter
Sleep 500ms

# Test 1: Simple command
Type@20ms "caro "
Type@30ms '"list files"'
Enter
Sleep 2s

# Test 2: Show output
Type "ls -la"
Enter
Sleep 1s

Enter
Type@10ms "# ✅ Setup working!"
Enter
Sleep 1s
