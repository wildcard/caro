# cmdai Demo Cheatsheet for DevOps/SRE/SysAdmin

## 🎯 Pitch: AI-Powered Command Generation for Infrastructure Teams

**cmdai** converts natural language to safe, production-ready shell commands using local LLMs with Apple Silicon optimization. Built for DevOps engineers, SREs, and system administrators who need fast, reliable command generation without leaving the terminal.

---

## 🚀 Performance Highlights

- **Single binary**: 3.9MB (no dependencies)
- **Local inference**: No API keys, no cloud dependencies
- **Apple Silicon optimized**: MLX backend with Metal acceleration
- **Safety-first**: Validates dangerous operations before execution
- **Instant cache**: Model loaded once, sub-second responses after

---

## 💼 Daily Operations - System Monitoring

### Process & Resource Management
```bash
cmdai "find processes using more than 1GB of memory"
cmdai "show top 10 CPU consuming processes"
cmdai "kill all zombie processes"
cmdai "find process listening on port 8080"
cmdai "show all processes owned by user nginx"
```

### Disk & Storage Operations
```bash
cmdai "find directories larger than 10GB"
cmdai "show disk usage by directory sorted by size"
cmdai "find files modified in the last 24 hours"
cmdai "delete log files older than 30 days"
cmdai "find duplicate files in current directory"
```

### System Health Checks
```bash
cmdai "check if system needs reboot"
cmdai "show system uptime and load average"
cmdai "monitor memory usage every 5 seconds"
cmdai "check filesystem usage above 80%"
cmdai "show kernel version and OS details"
```

---

## 🔥 Incident Response & Troubleshooting

### Log Analysis
```bash
cmdai "find all ERROR entries in syslog from last hour"
cmdai "show nginx error log entries with 500 status"
cmdai "count unique IP addresses in access log"
cmdai "extract stack traces from application logs"
cmdai "find log files containing 'connection timeout'"
```

### Network Diagnostics
```bash
cmdai "show all established TCP connections"
cmdai "find which process is using port 443"
cmdai "test connectivity to database server on port 5432"
cmdai "show network interface statistics"
cmdai "display routing table with gateway info"
cmdai "check DNS resolution for api.example.com"
```

### Service Management
```bash
cmdai "restart nginx service and check status"
cmdai "show failed systemd services"
cmdai "enable postgresql to start on boot"
cmdai "check if redis is running and responsive"
cmdai "reload docker daemon configuration"
```

---

## 🏗️ Infrastructure Management

### Docker & Container Operations
```bash
cmdai "remove all stopped containers"
cmdai "show container resource usage"
cmdai "find images larger than 1GB"
cmdai "inspect container logs from last 100 lines"
cmdai "export container filesystem to tarball"
```

### File & Directory Operations
```bash
cmdai "create timestamped backup of config directory"
cmdai "find and replace text in all yaml files"
cmdai "sync directory to remote server preserving permissions"
cmdai "compress logs directory excluding current month"
cmdai "find broken symbolic links"
```

### User & Permission Management
```bash
cmdai "show users who logged in today"
cmdai "find files with world-writable permissions"
cmdai "list sudo access for all users"
cmdai "show failed login attempts"
cmdai "find files owned by deleted users"
```

---

## 📊 Reporting & Automation

### Performance Metrics
```bash
cmdai "generate disk usage report for all mounted filesystems"
cmdai "show network bandwidth usage per interface"
cmdai "list services and their memory consumption"
cmdai "export process tree with resource usage"
cmdai "create CSV of top memory consumers"
```

### Batch Operations
```bash
cmdai "rename all txt files to add timestamp prefix"
cmdai "change ownership of web files to www-data"
cmdai "create directories for each month of 2024"
cmdai "download list of URLs from file in parallel"
cmdai "compress each subdirectory into separate archives"
```

### Security Auditing
```bash
cmdai "find files with setuid bit enabled"
cmdai "show open network ports and listening services"
cmdai "list cron jobs for all users"
cmdai "find recently modified system binaries"
cmdai "check for weak SSH configurations"
```

---

## 🎬 Demo Flow Suggestions

### **Act 1: The Problem** (30 seconds)
Show a typical day: googling syntax, checking man pages, asking ChatGPT for commands, copy-pasting from Stack Overflow.

### **Act 2: The Solution** (2 minutes)
```bash
# Start with simple
cmdai "list all files"

# Progress to real scenarios
cmdai "find nginx error logs from last hour with 502 errors"
cmdai "show processes using more than 2GB memory sorted by usage"
cmdai "backup database with timestamp and compress"

# Show safety validation
cmdai "delete everything in root directory"  # Should block/warn
```

### **Act 3: The Impact** (1 minute)
- **Before**: 2-5 minutes per command (search, verify, adapt)
- **After**: 2-3 seconds per command (type, review, execute)
- **ROI**: 10-50 commands/day × 5 minutes saved = 50-250 minutes/day per engineer

---

## 💡 Key Selling Points

### For Engineers
- ✅ No context switching (stays in terminal)
- ✅ No API keys or cloud dependencies
- ✅ Learns from your environment (shell detection)
- ✅ Safety validation prevents disasters
- ✅ POSIX-compliant for portability

### For Organizations
- 💰 Reduced onboarding time for junior engineers
- 💰 Faster incident response (seconds vs minutes)
- 💰 Lower cloud costs (local inference)
- 💰 Improved security (no sensitive data sent externally)
- 💰 Better compliance (all operations logged locally)

### Technical Differentiators
- 🚀 Apple Silicon optimization (MLX + Metal)
- 🚀 Single binary deployment (<5MB)
- 🚀 Model caching (1.1GB download once)
- 🚀 Sub-second inference after warm-up
- 🚀 Open source (AGPL-3.0)

---

## 🎯 Closing Statements

**"cmdai turns every engineer into a shell expert"**

**"Stop googling bash syntax. Start shipping faster."**

**"AI-powered productivity for the terminal generation."**

---

## 📈 Market Opportunity

- **TAM**: 15M+ DevOps/SRE/SysAdmin professionals globally
- **Use cases**: Daily runbook operations, incident response, infrastructure automation
- **Pricing model**: Freemium (local) + Enterprise (team features, compliance, support)
- **Expansion**: Cloud backends, custom models, IDE integrations, CI/CD plugins

---

## 🔮 Roadmap Highlights

- **Q1 2025**: Team collaboration features (shared command history)
- **Q2 2025**: Enterprise security (audit logs, RBAC, SSO)
- **Q3 2025**: IDE plugins (VSCode, JetBrains, Vim)
- **Q4 2025**: Cloud backends (AWS, GCP, Azure integration)

---

**Live Demo**: `cmdai "your infrastructure challenge here"`

**Website**: github.com/wildcard/cmdai  
**License**: AGPL-3.0 (Open Source)
