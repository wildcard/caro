# Tutorial: System Operations

Learn to monitor, manage, and troubleshoot your system using cmdai.

## What You'll Learn

In this tutorial, you'll learn to:
- 📊 Monitor system resources (CPU, memory, disk)
- 🔍 Inspect processes and services
- 🌐 Check network connectivity
- 🐛 Debug common issues
- 🔧 Perform system maintenance

**Time to complete:** ~15 minutes
**Prerequisites:** [Working with Files](./working-with-files.md)
**Related:** [Performance Optimization](../technical/performance.md) | [Safety & Security](../user-guide/safety.md)

## Quick Reference

| Category | Example Prompt | Common Commands |
|----------|----------------|-----------------|
| **System Info** | "show system information" | `uname`, `sw_vers`, `lsb_release` |
| **Disk** | "show disk usage" | `df`, `du` |
| **Processes** | "show running processes" | `ps`, `top`, `htop` |
| **Network** | "test internet connection" | `ping`, `ifconfig`, `netstat` |
| **Performance** | "show CPU usage" | `top`, `ps`, `vmstat` |

---

## Example 1: System Information

### Scenario: What System Am I On?

Quick system overview:

```bash
cmdai "show system information"
```

**Generated:**
```bash
uname -a
```

**Execute it:**
```
Darwin MacBook-Pro.local 23.0.0 Darwin Kernel Version 23.0.0
arm64
```

**What you learned:**
- OS: macOS (Darwin)
- Architecture: ARM64 (Apple Silicon)
- Kernel version: 23.0.0

---

### Scenario: Check macOS Version

```bash
cmdai "show macOS version"
```

**Generated:**
```bash
sw_vers
```

**Execute it:**
```
ProductName:        macOS
ProductVersion:     14.1
BuildVersion:       23B74
```

---

## Example 2: Disk and Storage

### Scenario: Check Disk Space

Is your disk full?

```bash
cmdai "show disk usage in human readable format"
```

**Generated:**
```bash
df -h
```

**Execute it:**
```
Filesystem      Size   Used  Avail Capacity  Mounted on
/dev/disk1s1   500Gi  350Gi  148Gi    71%    /
/dev/disk1s2   500Gi  1.0Gi  148Gi     1%    /System/Volumes/Data
```

**Understanding the output:**
- **Size:** Total disk capacity
- **Used:** Space currently in use
- **Avail:** Free space remaining
- **Capacity:** Percentage used
- **Mounted on:** Where the disk is accessible

<div class="info">
<strong>💡 Rule of thumb:</strong> Keep at least 10-20% free space for optimal performance.
</div>

---

### Scenario: What's Using My Disk?

Find space hogs:

```bash
cmdai "show top 10 largest directories"
```

**Generated:**
```bash
du -sh */ 2>/dev/null | sort -rh | head -10
```

**Execute it:**
```
 45G    Library/
 12G    Applications/
 8.5G   Downloads/
 3.2G   Documents/
 1.5G   Pictures/
 890M   Desktop/
 450M   Music/
 125M   Movies/
  85M   .cache/
  42M   .npm/
```

**What's happening:**
- `du -sh */` - Disk usage of directories
- `2>/dev/null` - Hide permission errors
- `sort -rh` - Sort by size (largest first)
- `head -10` - Show top 10

---

## Example 3: Process Management

### Scenario: What's Running?

See all processes:

```bash
cmdai "show all running processes"
```

**Generated:**
```bash
ps aux
```

**Execute it (truncated):**
```
USER    PID  %CPU %MEM    VSZ   RSS TTY   STAT START TIME COMMAND
user   1234  15.2  2.1 8192000 524288 ??   S   9:00AM 1:23.45 /Applications/Chrome
user   5678   5.3  1.8 6144000 458752 ??   S   9:15AM 0:45.12 /Applications/Slack
user   9012   0.5  0.3 2048000  98304 ??   S  10:00AM 0:05.23 /usr/bin/terminal
```

**Understanding columns:**
- **PID:** Process ID
- **%CPU:** CPU usage
- **%MEM:** Memory usage
- **COMMAND:** What's running

---

### Scenario: Find Memory Hogs

Which app is using all your RAM?

```bash
cmdai "show processes using most memory"
```

**Generated:**
```bash
ps aux | sort -nrk 4 | head -10
```

**Execute it:**
```
USER    PID  %CPU %MEM COMMAND
user   1234  15.2  8.5 /Applications/Chrome
user   2345   3.1  4.2 /Applications/Slack
user   3456   1.8  3.1 /Applications/VSCode
```

**Sort keys:**
- `-nrk 4` - Numerically, reverse, by column 4 (%MEM)

---

### Scenario: Kill a Frozen App

Find the process ID:

```bash
cmdai "find process ID for Chrome"
```

**Generated:**
```bash
pgrep -l Chrome
```

**Execute it:**
```
1234 Google Chrome
1235 Google Chrome Helper
```

Then (if needed):

```bash
cmdai "kill process 1234"
```

**Generated:**
```bash
kill 1234
```

**Safety check:** ⚠️ Moderate (terminates a process)

<div class="warning">
<strong>⚠️ Warning:</strong> Killing processes can cause unsaved data loss. Try closing the app normally first!
</div>

---

## Example 4: Network Operations

### Scenario: Am I Connected to the Internet?

Test connectivity:

```bash
cmdai "test internet connection"
```

**Generated:**
```bash
ping -c 4 8.8.8.8
```

**Execute it:**
```
PING 8.8.8.8 (8.8.8.8): 56 data bytes
64 bytes from 8.8.8.8: icmp_seq=0 ttl=117 time=12.3 ms
64 bytes from 8.8.8.8: icmp_seq=1 ttl=117 time=11.8 ms
64 bytes from 8.8.8.8: icmp_seq=2 ttl=117 time=12.1 ms
64 bytes from 8.8.8.8: icmp_seq=3 ttl=117 time=12.5 ms

--- 8.8.8.8 ping statistics ---
4 packets transmitted, 4 packets received, 0.0% packet loss
round-trip min/avg/max/stddev = 11.8/12.2/12.5/0.3 ms
```

**What it means:**
- ✅ 0% packet loss = Good connection
- ⚠️ >5% packet loss = Connection issues
- ❌ 100% packet loss = No internet

---

### Scenario: Check Your IP Address

What's my local IP?

```bash
cmdai "show my IP address"
```

**Generated:**
```bash
ifconfig | grep "inet " | grep -v 127.0.0.1
```

**Execute it:**
```
inet 192.168.1.42 netmask 0xffffff00 broadcast 192.168.1.255
```

Your local IP: **192.168.1.42**

---

### Scenario: What Ports Are Open?

See listening ports:

```bash
cmdai "show all open ports"
```

**Generated:**
```bash
lsof -i -P | grep LISTEN
```

**Execute it:**
```
node     12345 user   TCP *:3000 (LISTEN)
postgres 23456 user   TCP *:5432 (LISTEN)
docker   34567 user   TCP *:80 (LISTEN)
```

**What's running:**
- Node.js app on port 3000
- PostgreSQL on port 5432
- Docker container on port 80

---

## Example 5: Performance Monitoring

### Scenario: Real-Time CPU Usage

Watch CPU usage live:

```bash
cmdai "show real time CPU usage"
```

**Generated:**
```bash
top -o cpu
```

**What you'll see:**
- Live updating list of processes
- Sorted by CPU usage
- Press `q` to quit

---

### Scenario: Memory Status

How much RAM is free?

```bash
cmdai "show memory usage"
```

**Generated:**
```bash
vm_stat | perl -ne '/page size of (\d+)/ and $size=$1; /Pages\s+([^:]+)[^\d]+(\d+)/ and printf("%-16s % 16.2f Mi\n", "$1:", $2 * $size / 1048576);'
```

**Execute it:**
```
free:                    2048.00 Mi
active:                  4096.50 Mi
inactive:                2048.25 Mi
speculative:              512.75 Mi
wired down:              3072.00 Mi
```

**Memory types:**
- **free:** Unused RAM
- **active:** Actively used
- **inactive:** Recently used, can be freed
- **wired:** Kernel memory, cannot be freed

---

## Example 6: Troubleshooting

### Scenario: Check If a Service Is Running

Is the web server running?

```bash
cmdai "check if nginx is running"
```

**Generated:**
```bash
pgrep -l nginx
```

**If running:**
```
12345 nginx: master process
12346 nginx: worker process
```

**If not running:**
```
(no output)
```

---

### Scenario: View Recent System Logs

What happened recently?

```bash
cmdai "show last 50 system log messages"
```

**Generated (macOS):**
```bash
log show --predicate 'processID == 0' --last 1h | tail -50
```

**Generated (Linux):**
```bash
journalctl -n 50
```

---

### Scenario: Check System Uptime

How long has the system been running?

```bash
cmdai "show system uptime"
```

**Generated:**
```bash
uptime
```

**Execute it:**
```
10:42  up 7 days, 14:23, 3 users, load averages: 2.15 1.98 1.87
```

**What it means:**
- System has been up for 7 days, 14 hours, 23 minutes
- 3 users logged in
- Load averages: 1-min, 5-min, 15-min (1.0 = full CPU core usage)

---

## Example 7: System Maintenance

### Scenario: Find Crashed Applications

Check for crash reports:

```bash
cmdai "list recent crash reports"
```

**Generated (macOS):**
```bash
ls -lt ~/Library/Logs/DiagnosticReports/ | head -10
```

**Execute it:**
```
-rw-r--r--  Chrome_2024-11-19_093045_crash.txt
-rw-r--r--  Slack_2024-11-18_154523_crash.txt
```

---

### Scenario: Clear System Caches

Free up space by clearing caches:

```bash
cmdai "show size of cache directories"
```

**Generated:**
```bash
du -sh ~/Library/Caches/* | sort -rh | head -10
```

**Execute it:**
```
  1.5G  com.google.Chrome
  890M  com.apple.Safari
  450M  com.docker.docker
  125M  com.microsoft.VSCode
```

To clear (carefully!):

```bash
cmdai "clear Chrome cache"
```

**Safety check:** ⚠️ Moderate (deletes cache files)

<div class="info">
<strong>💡 Note:</strong> Clearing caches is safe but apps will be slower on next launch as they rebuild caches.
</div>

---

## Real-World Example: System Health Check

Combine commands for a complete health check:

### Step 1: Resource Check

```bash
cmdai "show disk usage"
# Check: Do I have enough space?

cmdai "show memory usage"
# Check: Is RAM available?
```

### Step 2: Performance Check

```bash
cmdai "show processes using most CPU"
# Check: What's slowing down my computer?

cmdai "show processes using most memory"
# Check: What's using all my RAM?
```

### Step 3: Network Check

```bash
cmdai "test internet connection"
# Check: Am I online?

cmdai "show my IP address"
# Check: Am I on the right network?
```

### Step 4: Cleanup

```bash
cmdai "find large files taking up space"
# Action: Review and delete if needed

cmdai "show size of cache directories"
# Action: Clear unnecessary caches
```

---

## Platform-Specific Commands

### macOS

```bash
# Battery status
cmdai "show battery percentage"
# → pmset -g batt

# Connected displays
cmdai "show connected monitors"
# → system_profiler SPDisplaysDataType

# Running services
cmdai "list all running services"
# → launchctl list
```

### Linux

```bash
# System info
cmdai "show detailed system information"
# → lsb_release -a

# Hardware info
cmdai "show CPU information"
# → lscpu

# Service status
cmdai "check status of nginx service"
# → systemctl status nginx
```

---

## Safety Best Practices

### ✅ DO: Monitor Before Acting

```bash
# First: Check what's using resources
cmdai "show processes using most memory"

# Then: Decide what to kill
cmdai "kill process 1234"
```

### ✅ DO: Check Before Deleting

```bash
# First: See what will be affected
cmdai "show size of cache directories"

# Then: Clear specific caches
cmdai "clear specific app cache"
```

### ❌ DON'T: Kill System Processes

```bash
# Dangerous! Don't kill PID 1 or system processes
cmdai "kill all processes"  # Never do this!
```

### ❌ DON'T: Delete System Files

```bash
# Very dangerous!
cmdai "delete all logs"  # Too broad!
```

---

## Practice Challenges

### Challenge 1: Performance Investigation
"Your computer is slow. Use cmdai to find what's causing it."

<details>
<summary>Solution Steps</summary>

1. `cmdai "show processes using most CPU"`
2. `cmdai "show processes using most memory"`
3. `cmdai "show disk usage"`
4. Identify the culprit and take action
</details>

### Challenge 2: Network Troubleshooting
"You can't access a website. Debug the network."

<details>
<summary>Solution Steps</summary>

1. `cmdai "test internet connection"`
2. `cmdai "show my IP address"`
3. `cmdai "ping google.com"`
4. Check DNS: `cmdai "test DNS resolution"`
</details>

### Challenge 3: Disk Space Recovery
"You have 5GB free but need 20GB. Find what to delete."

<details>
<summary>Solution Steps</summary>

1. `cmdai "show top 10 largest directories"`
2. `cmdai "find files larger than 1GB"`
3. `cmdai "show size of cache directories"`
4. Review and safely delete unnecessary files
</details>

---

## What You Learned

You now know how to:

✅ Monitor system resources (disk, CPU, memory)
✅ Manage running processes
✅ Check network connectivity
✅ Troubleshoot common issues
✅ Perform system maintenance
✅ Debug performance problems

---

## Next Steps

You've completed all tutorials! 🎉

**Practice more:**
- **[Try It Online](./playground.md)** - Interactive playground
- **[Quick Start](../user-guide/quick-start.md)** - More examples

**Go deeper:**
- **[Safety & Security](../user-guide/safety.md)** - Advanced safety
- **[Configuration](../user-guide/configuration.md)** - Customize cmdai
- **[Architecture](../dev-guide/architecture.md)** - How it works

**Contribute:**
- **[Contributing Guide](../community/contributing.md)** - Join the project

---

## See Also

**Related Tutorials:**
- [Your First Command](./first-command.md) - Basic cmdai usage
- [Working with Files](./working-with-files.md) - File operations

**User Guides:**
- [Safety & Security](../user-guide/safety.md) - Understand risk levels for system commands
- [Configuration](../user-guide/configuration.md) - Configure safety for system operations

**Technical Details:**
- [Safety Validation](../technical/safety-validation.md) - Protection against dangerous system commands
- [Performance Optimization](../technical/performance.md) - cmdai performance characteristics
- [Architecture](../dev-guide/architecture.md) - How command generation works

---

**Want to practice interactively?** Try the [Online Playground](./playground.md) →
