# Tutorial: Working with Files

Master file operations with cmdai through practical, real-world examples.

## What You'll Build

In this tutorial, you'll learn to:
- 🔍 Find files by name, size, or date
- 📂 Organize files and directories
- 🔄 Copy, move, and backup files safely
- 🧹 Clean up temporary files

**Time to complete:** ~15 minutes
**Prerequisites:** [Your First Command](./first-command.md)

---

## Example 1: Finding Files

### Scenario: Find Your Photos

You have photos scattered across subdirectories. Let's find them!

```bash
cmdai "find all JPEG images in this directory and subdirectories"
```

**Generated:**
```bash
find . -type f \( -name "*.jpg" -o -name "*.jpeg" \) -print
```

**Execute it:**
```
./photos/vacation/IMG_001.jpg
./photos/vacation/IMG_002.jpg
./work/screenshot.jpeg
./downloads/photo.jpg
```

**Understanding the command:**
- `find .` - Search starting from current directory
- `-type f` - Only files (not directories)
- `-name "*.jpg" -o -name "*.jpeg"` - Match .jpg OR .jpeg
- `-print` - Display the results

<div class="info">
<strong>💡 Try it yourself:</strong> Modify the prompt to find PNG images instead!
</div>

---

### Scenario: Find Large Files

Your disk is full. Let's find space hogs:

```bash
cmdai "show files larger than 100MB"
```

**Generated:**
```bash
find . -type f -size +100M -exec ls -lh {} \;
```

**Execute it:**
```
-rw-r--r--  1 user  staff   256M Nov 15 10:30 ./videos/presentation.mp4
-rw-r--r--  1 user  staff   150M Nov 18 14:20 ./downloads/installer.dmg
```

**What's new:**
- `-size +100M` - Files larger than 100 megabytes
- `-exec ls -lh {} \;` - Run `ls -lh` on each file found

---

### Scenario: Find Recent Files

What did you work on this week?

```bash
cmdai "find files modified in the last 7 days"
```

**Generated:**
```bash
find . -type f -mtime -7 -ls
```

**Execute it:**
```
12345  8 -rw-r--r--  1 user staff  2048 Nov 18 09:15 ./report.txt
12346 16 -rw-r--r--  1 user staff  5120 Nov 19 14:30 ./data.csv
```

**Time-based options:**
- `-mtime -7` - Modified within last 7 days
- `-mtime +30` - Modified more than 30 days ago
- `-mtime 0` - Modified today

---

## Example 2: Organizing Files

### Scenario: Create a Backup Directory

Before cleaning up, let's create a backup:

```bash
cmdai "create a directory called backup"
```

**Generated:**
```bash
mkdir backup
```

**Safety check:** ✅ Safe (only creates a directory)

**Execute it:**
```
Execute this command? (y/N) y
✅ Directory created successfully
```

---

### Scenario: Copy Files to Backup

Now let's backup important files:

```bash
cmdai "copy all PDF files to the backup directory"
```

**Generated:**
```bash
find . -maxdepth 1 -name "*.pdf" -exec cp {} backup/ \;
```

**Safety check:** ⚠️ Moderate (copies files, doesn't delete anything)

**What's happening:**
1. Find all PDF files in current directory (`-maxdepth 1`)
2. Copy each one to `backup/` directory
3. Original files remain untouched

---

### Scenario: Organize by Type

Let's organize files into subdirectories:

```bash
cmdai "create directories for documents, images, and videos"
```

**Generated:**
```bash
mkdir -p documents images videos
```

**Why `-p`?**
- Creates parent directories if needed
- No error if directory already exists

Now move files:

```bash
cmdai "move all PDF and text files to documents folder"
```

**Generated:**
```bash
mv *.pdf *.txt documents/
```

**Safety check:** ⚠️ Moderate (moves files, but reversible)

---

## Example 3: File Information

### Scenario: Count Files by Type

How many files of each type do you have?

```bash
cmdai "count files grouped by extension"
```

**Generated:**
```bash
find . -type f | sed 's/.*\.//' | sort | uniq -c | sort -nr
```

**Execute it:**
```
  45 jpg
  23 pdf
  12 txt
   8 png
   3 mp4
```

**Breaking it down:**
1. `find . -type f` - Find all files
2. `sed 's/.*\.//'` - Extract file extension
3. `sort` - Sort extensions
4. `uniq -c` - Count unique extensions
5. `sort -nr` - Sort by count (descending)

<div class="info">
<strong>🎯 Pro tip:</strong> cmdai generated a pipeline! Multiple commands connected with pipes create powerful workflows.
</div>

---

### Scenario: Show Directory Sizes

What's taking up space?

```bash
cmdai "show size of each directory sorted by size"
```

**Generated:**
```bash
du -sh */ | sort -h
```

**Execute it:**
```
 4.0K   backup/
 12M    documents/
 256M   photos/
 1.2G   videos/
```

**Understanding the output:**
- `du -sh */` - Disk usage, human-readable, for each directory
- `sort -h` - Sort by human-readable sizes

---

## Example 4: Safe Cleanup

### Scenario: Find Old Temporary Files

Let's find files we can safely delete:

```bash
cmdai "find temporary files older than 30 days"
```

**Generated:**
```bash
find . -type f -name "*.tmp" -mtime +30 -ls
```

**Review the list first!** Don't delete without checking.

---

### Scenario: Delete Specific Files

After reviewing, delete them:

```bash
cmdai "delete tmp files older than 30 days"
```

**Generated:**
```bash
find . -type f -name "*.tmp" -mtime +30 -delete
```

**Safety check:** 🔶 High Risk (deletes files!)

**What you'll see:**
```
⚠️  WARNING: This command will delete files

Generated command:
  find . -type f -name "*.tmp" -mtime +30 -delete

Files that will be affected:
  • Matches: *.tmp files
  • Age: Older than 30 days
  • Action: Permanent deletion

Are you absolutely sure? (y/N)
```

<div class="warning">
<strong>⚠️ Best Practice:</strong> Always run with `-ls` first to preview what will be deleted, then change to `-delete`.
</div>

---

## Example 5: Advanced File Operations

### Scenario: Find Duplicate Files

Find files with the same name in different directories:

```bash
cmdai "find duplicate filenames in this directory tree"
```

**Generated:**
```bash
find . -type f -printf '%f\n' | sort | uniq -d
```

**Execute it:**
```
config.json
README.md
test.txt
```

These filenames appear multiple times in different locations.

---

### Scenario: Batch Rename Files

Rename all files to lowercase:

```bash
cmdai "rename all files in current directory to lowercase"
```

**Generated:**
```bash
for f in *; do mv "$f" "$(echo $f | tr '[:upper:]' '[:lower:]')"; done
```

**Safety check:** ⚠️ Moderate (renames files)

**What it does:**
- Loop through all files
- Convert filename to lowercase
- Rename using `mv`

---

## Real-World Example: Project Cleanup

Let's combine what we learned for a common task:

**Goal:** Clean up a project directory

### Step 1: Survey the Situation

```bash
cmdai "show disk usage by directory sorted by size"
# Result: See what's using space
```

### Step 2: Find Large Files

```bash
cmdai "find files larger than 10MB"
# Result: Identify space hogs
```

### Step 3: Organize

```bash
cmdai "move all log files to a logs subdirectory"
# Result: Better organization
```

### Step 4: Clean Up

```bash
cmdai "find and list temporary files older than 7 days"
# Review the list first!

cmdai "delete temporary files older than 7 days"
# Only after reviewing!
```

---

## Safety Patterns

### ✅ DO: Preview Before Deleting

```bash
# First: Preview
cmdai "find old log files"

# Then: Delete
cmdai "delete old log files"
```

### ✅ DO: Backup Important Files

```bash
cmdai "copy all important files to backup directory"
```

### ✅ DO: Use Specific Patterns

```bash
# Good: Specific
cmdai "delete tmp files in current directory only"

# Bad: Too broad
cmdai "delete all temporary files everywhere"
```

### ❌ DON'T: Delete Without Review

```bash
# Dangerous!
cmdai --confirm "delete old files"
```

### ❌ DON'T: Use Wildcards Carelessly

```bash
# Too risky!
cmdai "delete * files"
```

---

## Practice Challenges

Try these scenarios yourself:

### Challenge 1: Photo Organization
"You have photos from 2023 and 2024. Create directories for each year and move photos accordingly."

<details>
<summary>Hint</summary>

Break it into steps:
1. Create directories: `mkdir 2023 2024`
2. Move files by date: Use `find` with `-newermt`
</details>

### Challenge 2: Log Rotation
"Find log files larger than 50MB and compress them."

<details>
<summary>Hint</summary>

Use `find` with `-size` and `gzip`:
1. Find: `find . -name "*.log" -size +50M`
2. Compress: Add `-exec gzip {} \;`
</details>

### Challenge 3: Duplicate Detection
"Find files with identical content (not just names) in your Downloads folder."

<details>
<summary>Hint</summary>

Use `fdupes` or generate checksums with `md5` or `sha256sum`:
```bash
cmdai "find duplicate files by content in Downloads"
```
</details>

---

## What You Learned

You now know how to:

✅ Find files by name, size, date, and content
✅ Organize files into directories
✅ Copy and move files safely
✅ Clean up temporary files with safety checks
✅ Use complex file operations
✅ Build pipelines for advanced tasks

---

## Next Steps

Continue learning:
- **[System Operations](./system-operations.md)** - Monitor and manage your system
- **[Try It Online](./playground.md)** - Interactive examples
- **[Safety & Security](../user-guide/safety.md)** - Advanced safety features

Or explore advanced topics:
- [Configuration](../user-guide/configuration.md) - Customize behavior
- [Quick Start](../user-guide/quick-start.md) - More examples

---

**Ready for system operations?** Continue to [Tutorial: System Operations](./system-operations.md) →
