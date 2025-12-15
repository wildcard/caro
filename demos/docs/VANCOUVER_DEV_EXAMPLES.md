# Vancouver.dev Demo Examples - December 16, 2024

Spicy, real-world command examples that showcase Caro's capabilities for developers and sysadmins.

## 🔍 Git & Version Control

### Git Archaeology
```bash
# Find that breaking commit
caro "show git commits from last 2 weeks with author names"
# Output: git log --since='2 weeks ago' --pretty=format:'%h %an %s'

caro "show all commits by author containing 'fix' in message"
# Output: git log --author='author' --grep='fix' --oneline

caro "find largest files in git history"
# Output: git rev-list --objects --all | git cat-file --batch-check='%(objectsize) %(objectname) %(rest)' | sort -nr | head -20

caro "show files changed in last commit with stats"
# Output: git show --stat HEAD

caro "find commits that touched authentication code"
# Output: git log --all --oneline -- '*auth*'
```

### Branch & Remote Operations
```bash
caro "list all branches sorted by last commit date"
# Output: git for-each-ref --sort=-committerdate refs/heads/ --format='%(committerdate:short) %(refname:short)'

caro "show commits ahead of main branch"
# Output: git log main..HEAD --oneline

caro "find branches not merged to main"
# Output: git branch --no-merged main
```

## 💻 Development Workflow

### Code Search & Analysis
```bash
caro "find all Rust files modified in the last 7 days"
# Output: find . -name '*.rs' -type f -mtime -7

caro "count total lines of TypeScript code"
# Output: find . -name '*.ts' -not -path '*/node_modules/*' | xargs wc -l

caro "find files with TODO comments"
# Output: grep -r "TODO" --include="*.rs" --include="*.ts" .

caro "show files larger than 10MB in this directory"
# Output: find . -type f -size +10M -exec ls -lh {} \;

caro "find all test files modified today"
# Output: find . -name '*test*.rs' -type f -mtime -1
```

### Dependencies & Package Management
```bash
caro "list all npm dependencies with their versions"
# Output: npm list --depth=0

caro "find outdated npm packages"
# Output: npm outdated

caro "show disk space used by node_modules"
# Output: du -sh node_modules

caro "find all package.json files in subdirectories"
# Output: find . -name 'package.json' -type f
```

## 🖥️ System Monitoring & Performance

### Process Management
```bash
caro "show top 5 processes by CPU usage"
# Output: ps aux --sort=-%cpu | head -6

caro "show top 10 memory-consuming processes"
# Output: ps aux --sort=-%mem | head -11

caro "find all Node.js processes"
# Output: ps aux | grep -i node

caro "show processes listening on port 3000"
# Output: lsof -i :3000

caro "kill all processes matching pattern nginx"
# Output: pkill nginx  # (requires confirmation due to safety)
```

### System Health
```bash
caro "show system uptime and load average"
# Output: uptime

caro "show disk usage sorted by size"
# Output: df -h | sort -k5 -r

caro "check memory usage in gigabytes"
# Output: free -g

caro "show kernel version and OS details"
# Output: uname -a && cat /etc/os-release

caro "show temperature of CPU cores"
# Output: sensors  # (if lm-sensors installed)
```

## 🌐 Network & Services

### Network Debugging
```bash
caro "show all listening TCP ports"
# Output: lsof -nP -iTCP -sTCP:LISTEN

caro "test DNS resolution for api.github.com"
# Output: dig +short api.github.com

caro "show all established connections to port 443"
# Output: ss -tn state established '( dport = :443 or sport = :443 )'

caro "find which process is using port 8080"
# Output: lsof -i :8080

caro "show network interface statistics"
# Output: ip -s link

caro "ping host 5 times and show round-trip time"
# Output: ping -c 5 example.com
```

### Service Management
```bash
caro "check if nginx is running"
# Output: systemctl is-active nginx

caro "show all failed systemd services"
# Output: systemctl --failed

caro "show nginx error log last 20 lines"
# Output: tail -20 /var/log/nginx/error.log

caro "restart docker service and check status"
# Output: sudo systemctl restart docker && systemctl status docker
```

## 📊 Log Analysis

### Application Logs
```bash
caro "find all error messages in application logs"
# Output: grep -i error /var/log/app/*.log

caro "count 500 errors in nginx access log"
# Output: grep '500' /var/log/nginx/access.log | wc -l

caro "show unique IP addresses from access log"
# Output: awk '{print $1}' /var/log/nginx/access.log | sort | uniq

caro "find all log files and count total lines"
# Output: find /var/log -name '*.log' -type f -exec wc -l {} + | tail -1

caro "tail application log and filter for errors"
# Output: tail -f /var/log/app.log | grep --color=auto ERROR
```

### System Logs
```bash
caro "show failed SSH login attempts"
# Output: grep 'Failed password' /var/log/auth.log

caro "show last 50 kernel messages"
# Output: dmesg | tail -50

caro "find out of memory events in logs"
# Output: grep -i 'out of memory' /var/log/syslog
```

## 🗄️ Database Operations

### Database Debugging
```bash
caro "show PostgreSQL active connections"
# Output: psql -c "SELECT * FROM pg_stat_activity WHERE state = 'active';"

caro "export database to SQL file"
# Output: pg_dump dbname > backup.sql

caro "show MySQL slow queries"
# Output: mysqldumpslow -t 10 /var/log/mysql/slow.log

caro "check Redis memory usage"
# Output: redis-cli INFO memory
```

## 🔒 Security & Permissions

### Security Audit
```bash
caro "find files with world-writable permissions"
# Output: find . -type f -perm 0002

caro "find files owned by root with setuid bit"
# Output: find / -user root -perm -4000 2>/dev/null

caro "show recent sudo commands"
# Output: grep sudo /var/log/auth.log | tail -20

caro "list users who can sudo"
# Output: getent group sudo

caro "find SSH keys in home directories"
# Output: find /home -name 'id_rsa*' -o -name 'id_ed25519*'
```

### File Permissions
```bash
caro "make all shell scripts executable"
# Output: find . -name '*.sh' -type f -exec chmod +x {} \;

caro "show files modified in last hour"
# Output: find . -type f -mmin -60

caro "find broken symbolic links"
# Output: find . -xtype l
```

## 📦 Docker & Containers

### Container Management
```bash
caro "show running docker containers with their ports"
# Output: docker ps --format 'table {{.Names}}\t{{.Ports}}'

caro "show disk space used by docker"
# Output: docker system df

caro "remove stopped containers older than 24 hours"
# Output: docker container prune -f --filter 'until=24h'

caro "show logs of container named api"
# Output: docker logs api

caro "list all docker images sorted by size"
# Output: docker images --format '{{.Size}}\t{{.Repository}}:{{.Tag}}' | sort -hr
```

## 🎯 Productivity Hacks

### Compression & Archives
```bash
caro "create compressed archive of current directory"
# Output: tar czf archive-$(date +%Y%m%d).tar.gz .

caro "extract tar.gz file to specific directory"
# Output: tar xzf archive.tar.gz -C /target/directory

caro "compress all log files excluding today"
# Output: find /var/log -name '*.log' -mtime +1 -exec gzip {} \;
```

### Batch Operations
```bash
caro "rename all txt files to have .backup extension"
# Output: for f in *.txt; do mv "$f" "$f.backup"; done

caro "convert all JPEG images to PNG"
# Output: for img in *.jpg; do convert "$img" "${img%.jpg}.png"; done

caro "remove all node_modules directories"
# Output: find . -name 'node_modules' -type d -prune -exec rm -rf {} +
```

### Text Processing
```bash
caro "count unique lines in file"
# Output: sort file.txt | uniq | wc -l

caro "show duplicate lines in file"
# Output: sort file.txt | uniq -d

caro "remove empty lines from file"
# Output: sed '/^$/d' file.txt

caro "replace all occurrences of foo with bar"
# Output: sed -i 's/foo/bar/g' file.txt
```

## 🚨 Incident Response

### Quick Diagnostics
```bash
caro "show what's filling up disk space"
# Output: du -h --max-depth=1 / 2>/dev/null | sort -hr | head -20

caro "find recently modified files in /etc"
# Output: find /etc -type f -mtime -7 -ls

caro "show active network connections by process"
# Output: lsof -i -n -P

caro "check if system is under high load"
# Output: uptime && top -bn1 | head -20
```

### Performance Investigation
```bash
caro "show IO statistics for all disks"
# Output: iostat -x 1 3

caro "find which process is using most disk IO"
# Output: iotop -oP

caro "show TCP connection states summary"
# Output: ss -s

caro "check DNS resolution time"
# Output: time dig +short google.com
```

---

## 💡 Tips for Vancouver.dev Presentation

### What Makes These Examples "Spicy"
1. **Real scenarios**: Git archaeology, production debugging, incident response
2. **Developer pain points**: Finding breaking commits, memory leaks, port conflicts
3. **Time-savers**: Complex command patterns simplified to natural language
4. **Safety first**: Dangerous commands require confirmation (won't rm -rf / by accident)

### Demo Flow Suggestions
1. **Start with Git** - everyone relates to finding commits
2. **Show system monitoring** - real production scenarios
3. **Network debugging** - common dev problem
4. **End with productivity** - show time savings

### Key Messages
- "Stop memorizing flags and options"
- "Natural language → correct command"
- "100% local, privacy-first"
- "Safety built-in, won't destroy your system"
- "Optimized for Apple Silicon developers"

### Audience Engagement
- Ask: "Who's ever typed `man ps` and immediately regretted it?"
- Show: Before/after comparison (traditional vs caro)
- Highlight: Commands you'd normally have to Google
- Demonstrate: Real-time generation speed (<2s)

---

**Ready to demo:** December 16, 2024 🚀
