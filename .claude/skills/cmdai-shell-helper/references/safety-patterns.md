# cmdai Safety Patterns Reference

This document catalogs the dangerous command patterns that cmdai detects and blocks or warns about.

## Risk Level Matrix

| Risk Level | Color | Confirmation | Examples |
|------------|-------|--------------|----------|
| **Safe** | 🟢 Green | Not required | `ls`, `find`, `grep`, `cat` |
| **Moderate** | 🟡 Yellow | In strict mode | File edits, package updates |
| **High** | 🟠 Orange | Always required | `rm -rf`, mass operations |
| **Critical** | 🔴 Red | Blocked (override needed) | System destruction, fork bombs |

## Critical (Red) - Always Blocked

### System Destruction Patterns

```bash
# Root filesystem deletion
rm -rf /
rm -rf /*
rm -rf /bin
rm -rf /usr
rm -rf /etc
rm -rf /lib
rm -rf /lib64
rm -rf /boot

# Home directory destruction
rm -rf ~
rm -rf ~/
rm -rf $HOME

# Current directory mass deletion (dangerous in wrong location)
rm -rf .
rm -rf ./
rm -rf *
```

**Why dangerous:**
- Irreversible data loss
- System becomes unbootable
- Recovery requires reinstallation

**Safe alternatives:**
```bash
# Delete specific subdirectory
rm -rf ./specific-folder

# Interactive deletion
rm -ri ./folder

# Preview before deleting
find ./folder -type f -ls
```

### Fork Bombs

```bash
# Classic bash fork bomb
:(){ :|:& };:

# Variations
.(){{.|.&};};.
$0 & $0 &
```

**Why dangerous:**
- Creates infinite processes
- Exhausts system resources
- Causes system freeze/crash
- Requires hard reboot

**No safe alternative:** These are never legitimate commands.

### Disk Operations

```bash
# Filesystem creation (destroys existing data)
mkfs /dev/sda
mkfs.ext4 /dev/sda1
mke2fs /dev/sda

# Disk overwrite
dd if=/dev/zero of=/dev/sda
dd if=/dev/urandom of=/dev/sda

# Partition table destruction
fdisk /dev/sda
parted /dev/sda

# Block device direct write
cat /dev/zero > /dev/sda
```

**Why dangerous:**
- Complete data loss on entire disk
- Affects all partitions
- Recovery extremely difficult/impossible

**Safe alternatives:**
```bash
# Create filesystem on mounted image file
dd if=/dev/zero of=disk.img bs=1M count=100
mkfs.ext4 disk.img

# Use test/development disks only
# Always verify device name before operations
lsblk  # List all block devices first
```

### Privilege Escalation

```bash
# Gain root without audit trail
sudo su
sudo su -

# Make system world-writable
chmod 777 /
chmod -R 777 /usr

# Dangerous SUID modifications
chmod u+s /bin/bash
chmod 4755 /usr/bin/evil-script

# Disable security features
setenforce 0
systemctl stop firewalld
```

**Why dangerous:**
- Bypasses security controls
- Creates security vulnerabilities
- Leaves system open to attack
- Violates least-privilege principle

**Safe alternatives:**
```bash
# Use sudo for specific commands only
sudo specific-command

# Proper permission setting
chmod 755 user-script.sh
chmod 644 data-file.txt

# Temporary privilege elevation with audit
sudo -i  # Audited root shell
```

## High (Orange) - Requires Explicit Confirmation

### Recursive Deletions

```bash
# Recursive delete with wildcard
rm -rf ./*/
rm -rf ./*

# Delete all of specific type
rm -rf *.log
rm -rf **/*.tmp

# Delete hidden files
rm -rf .*
```

**Why concerning:**
- Can delete more than intended
- Irreversible
- Easy to mistype path

**Safe alternatives:**
```bash
# Preview first
find . -name "*.log" -type f -ls

# Interactive deletion
rm -ri ./folder

# Delete with confirmation per file
find . -name "*.log" -exec rm -i {} \;

# Move to trash instead
trash ./folder  # If trash-cli installed
```

### Mass File Operations

```bash
# Change permissions on everything
chmod -R 777 .
chown -R user:group /

# Mass file modifications
sed -i 's/old/new/g' *
find . -type f -exec sed -i 's/old/new/g' {} \;

# Mass moves/renames
mv * ../other-dir/
rename 's/old/new/' *
```

**Why concerning:**
- Affects many files at once
- May have unintended consequences
- Difficult to undo

**Safe alternatives:**
```bash
# Test on single file first
chmod 755 test-file.sh

# Use find with -print to preview
find . -type f -name "*.sh" -print

# Limit scope explicitly
find ./specific-dir -name "*.log" -exec operation {} \;

# Create backups before mass operations
tar czf backup.tar.gz directory/
```

### System Configuration Changes

```bash
# Firewall modifications
iptables -F  # Flush all rules
ufw disable

# Service modifications
systemctl disable important-service
systemctl stop critical-daemon

# Network configuration
ifconfig eth0 down
ip link set eth0 down

# Cron/scheduled task modifications
crontab -r  # Remove all cron jobs
```

**Why concerning:**
- Affects system behavior
- May break services
- Could lock you out
- Hard to diagnose if something breaks

**Safe alternatives:**
```bash
# Save current config first
iptables-save > iptables.backup
crontab -l > crontab.backup

# Test in non-production first
# Make incremental changes
# Document each change
```

## Moderate (Yellow) - Confirmation in Strict Mode

### File Modifications in System Directories

```bash
# Edit system files
vim /etc/hosts
nano /etc/fstab
echo "line" >> /etc/profile

# Create files in system paths
touch /usr/local/bin/script.sh
cp file /opt/
```

**Why concerning:**
- May affect all users
- Could break system
- Requires elevated privileges

**Safe alternatives:**
```bash
# Edit user-specific configs instead
vim ~/.bashrc
vim ~/.ssh/config

# Use user directories
mkdir -p ~/bin/
cp script.sh ~/bin/
```

### Package Manager Operations

```bash
# System package changes
apt-get install package
yum remove package
brew install package

# Package updates
apt-get upgrade
dnf update
pacman -Syu
```

**Why concerning:**
- Changes system state
- May introduce incompatibilities
- Can break dependencies

**Safe alternatives:**
```bash
# Preview changes first
apt-get --dry-run install package
apt-get --simulate upgrade

# Install to user directory when possible
pip install --user package
npm install --prefix ~/.local package
```

### Network Operations Affecting Firewall

```bash
# Port forwarding
iptables -A FORWARD -p tcp --dport 80 -j ACCEPT
ufw allow 22

# NAT configuration
iptables -t nat -A POSTROUTING -j MASQUERADE
```

**Why concerning:**
- Changes network security posture
- May expose services
- Could allow unwanted access

**Safe alternatives:**
```bash
# Use specific source IPs
ufw allow from 192.168.1.0/24 to any port 22

# Temporary rules for testing
iptables -I INPUT 1 -p tcp --dport 8080 -j ACCEPT
# Then review and make permanent if safe
```

## Safe (Green) - No Confirmation Required

### Read-Only Operations

```bash
# File listings
ls -lah
find . -name "*.txt"
tree

# File viewing
cat file.txt
less file.log
head -n 10 file.txt
tail -f application.log

# Content searching
grep -r "pattern" .
ag "search-term"
rg "pattern"

# System information
df -h
du -sh *
free -h
ps aux
top
htop
uname -a
```

**Why safe:**
- No modifications
- No side effects
- Cannot damage system
- Reversible (just close terminal)

### Safe Data Transformations

```bash
# Output to new file (not modifying original)
cat file1.txt file2.txt > combined.txt
grep "pattern" input.txt > output.txt
sort < unsorted.txt > sorted.txt

# Compression (original preserved)
tar czf backup.tar.gz directory/
zip -r archive.zip folder/
gzip -k file.txt  # -k keeps original

# Format conversion
convert image.png image.jpg
ffmpeg -i video.mp4 video.webm
pandoc input.md -o output.pdf
```

**Why safe:**
- Creates new files
- Originals unchanged
- Easy to verify results
- Simple to undo (delete output)

## Command Validation Rules

cmdai applies these rules automatically:

### 1. Path Quoting

```bash
# Automatically quotes paths with spaces
find ~/My Documents -name "*.pdf"
# Becomes:
find "~/My Documents" -name "*.pdf"
```

### 2. POSIX Compliance

```bash
# Rejects bash-specific syntax
[[ $var == "value" ]]
# Suggests POSIX alternative:
[ "$var" = "value" ]
```

### 3. Pipe Safety

```bash
# Validates pipe chains
find . -name "*.log" | xargs rm
# Warns: Use -print0 and xargs -0 for files with spaces
find . -name "*.log" -print0 | xargs -0 rm
```

### 4. Wildcard Expansion

```bash
# Warns about dangerous wildcards
rm -rf *
# Suggests safer alternative:
rm -rf ./specific-directory/*
```

## Custom Safety Patterns

Users can add custom patterns in `~/.config/cmdai/config.toml`:

```toml
[safety]
custom_patterns = [
    "rm -rf /important/project",  # Protect specific paths
    "DROP DATABASE production",   # SQL protection
    "kubectl delete namespace",   # Kubernetes protection
]
```

## Safety Override

For advanced users who understand the risks:

```bash
# Use --allow-dangerous flag (not recommended)
caro --allow-dangerous "risky operation"

# Or adjust safety level in config
[safety]
level = "permissive"  # strict, moderate, or permissive
```

⚠️ **Warning:** Overriding safety features requires understanding the risks.

## Learning Resources

- **OWASP Command Injection**: https://owasp.org/www-community/attacks/Command_Injection
- **ShellCheck**: https://www.shellcheck.net/ - Validates shell scripts
- **explainshell.com**: Explains what shell commands do
- **POSIX Specification**: https://pubs.opengroup.org/onlinepubs/9699919799/

## Summary

cmdai's safety system is designed to:
- ✅ Prevent catastrophic mistakes
- ✅ Educate users about command risks
- ✅ Promote POSIX compliance
- ✅ Encourage safe scripting practices
- ✅ Provide escape hatches for advanced users

Always err on the side of caution. When in doubt, preview before executing.
