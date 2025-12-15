# Vancouver.Dev Demo - Quick Reference Card
**PRINT THIS AND KEEP NEXT TO YOU DURING DEMO**

---

## 🎯 DEMO FLOW (Exact Commands to Type)

### 1. INTRODUCTION (30 sec)
**Say:** "Who here has googled shell syntax today? That's what we're fixing."

---

### 2. PYTHON DEMO (60 sec)
```bash
# If you have Python prototype ready
python caro.py

# Demo 2-3 commands, then say:
"This is the prototype. Now let's see the real thing..."
```

---

### 3. CLI DEMO (2.5 min) - TYPE THESE EXACTLY

#### Command 1: System Info (proven ⭐)
```bash
./target/release/cmdai "show system uptime and load average"
```
**Expected:** `uptime`  
**Then run:** `uptime`  
**Say:** "See? Instant. No googling."

---

#### Command 2: Process Management (proven ⭐)
```bash
./target/release/cmdai "show top 10 processes by CPU usage"
```
**Expected:** `ps aux | sort -nr -k 3 | head -n 10`  
**Then run:** `ps aux | sort -nr -k 3 | head -n 10 | head -3`  
**Say:** "Complex command, generated in under a second."

---

#### Command 3: File Operations (proven ⭐)
```bash
./target/release/cmdai "find all rust files modified in the last 7 days"
```
**Expected:** `find / -type f -name '*.rs' -mtime -7`  
**Say:** "Perfect for daily development workflow. I'd change / to . for current directory."

---

#### Command 4: Archive (proven ⭐)
```bash
./target/release/cmdai "archive current directory"
```
**Expected:** `tar czvf archive.tar.gz *`  
**Say:** "No more man page lookups for tar syntax."

---

#### Command 5: Security Audit (proven ⭐)
```bash
./target/release/cmdai "find files with setuid bit enabled"
```
**Expected:** `find /bin -type f -perm u+s`  
**Then run:** `find /usr/bin -type f -perm -u+s 2>/dev/null | head -3`  
**Say:** "Security auditing made simple."

---

#### Command 6: User Activity (proven ⭐)
```bash
./target/release/cmdai "show users who logged in today"
```
**Expected:** `last`  
**Then run:** `last | head -5`  
**Say:** "System administration in plain English."

---

#### Command 7: Network (proven ⭐)
```bash
./target/release/cmdai "check DNS resolution for api.example.com"
```
**Expected:** `dig api.example.com +short`  
**Say:** "Even complex networking commands work."

---

### 4. CLOSING (30 sec)
**Say:** 
"This is open source, free forever, and we need your help:
1. Star the repo
2. Test on your platform
3. Share what doesn't work
4. Contribute safety rules for your domain

Join us: github.com/[your-username]/caro.sh"

---

## 🚨 IF DEMO FAILS

### Backup Plan 1: Show Help
```bash
./target/release/cmdai --help
```
**Say:** "Live demos, right? But you can see the interface here..."

### Backup Plan 2: Show Pre-recorded
**Say:** "Let me show you a recording I made earlier..."
(Have a video ready)

### Backup Plan 3: Talk Through Architecture
**Say:** "Instead, let me walk you through how it works..."
(Pivot to architecture slide)

---

## 💬 KEY MESSAGES (Memorize These)

1. **"Specialized sub-agent, not a replacement"**
   - Big agents (Claude, ChatGPT) are starting points
   - Caro is for when you need terminal expertise

2. **"Not just a prompt - it's skills, tools, rules, community"**
   - Skills: Deep shell knowledge
   - Tools: Command generation engine
   - Rules: Community-curated safety
   - Community: People who care about terminals

3. **"44% perfect first try, 84% with iteration"**
   - Still faster than googling
   - Improving with community contributions

4. **"Local, private, free"**
   - No API costs
   - No data leaves your machine
   - Open source forever

5. **"We need builders, not just users"**
   - Test on your platform
   - Report what breaks
   - Contribute safety rules
   - Share ideas

---

## 📱 SOCIAL MEDIA (Show on Final Slide)

```
🐙 GitHub: github.com/[username]/caro.sh
💬 Discord: discord.gg/carosh
🐦 Twitter: @carosh_dev
📧 Email: hello@caro.sh
```

**QR CODE:** Generate for GitHub repo, display on final slide

---

## ⏱️ TIMING CHECKPOINTS

- **1:00** - Finished introduction
- **2:00** - Finished Python demo
- **4:30** - Finished CLI demo
- **5:00** - Finished closing + CTA

**If running long:** Skip commands 5-7, go straight to closing

**If running short:** Add more talking points about community

---

## 🎤 ENERGY LEVELS

- **High energy:** Introduction (hook the audience)
- **Steady:** Python demo (show concept quickly)
- **Building:** CLI demo (show value with each command)
- **Peak:** Closing (inspire them to contribute)

---

## 👥 AUDIENCE ENGAGEMENT

### During Demo:
- Make eye contact
- Pause for laughs/nods
- Ask rhetorical questions
- Show enthusiasm

### After Demo:
- "Questions?"
- Stick around for 1-on-1 chats
- Have laptop open for deeper dives
- Collect feedback

---

## ✅ PRE-TALK CHECKLIST (Day Of)

Technical:
- [ ] Binary compiled: `./target/release/cmdai --version`
- [ ] Model cached: Run one command to warm up
- [ ] Terminal font: Size 18+ (readable from back)
- [ ] All 7 demo commands tested once

Presentation:
- [ ] This cheat sheet printed
- [ ] Backup video downloaded
- [ ] Laptop charged (backup battery)
- [ ] Adapters (HDMI, USB-C)
- [ ] Water bottle

Mental:
- [ ] Deep breath
- [ ] Smile
- [ ] Have fun!

---

## 🎯 SUCCESS = 

- [ ] Delivered under 5 minutes
- [ ] At least 5 commands worked
- [ ] Audience engaged (nods, laughs)
- [ ] Called to action clear
- [ ] GitHub stars go up

---

**YOU'VE GOT THIS! 🚀**

Remember: Even if the demo fails, your passion and the vision matter more.
The community is what makes this special, not perfect command generation.

**GO BUILD IN THE OPEN! 🌟**
