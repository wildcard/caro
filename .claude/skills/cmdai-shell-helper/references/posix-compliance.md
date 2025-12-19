# POSIX Compliance Reference

## What is POSIX?

**POSIX** (Portable Operating System Interface) is a family of standards that ensures compatibility across different Unix-like operating systems. POSIX-compliant shell commands work reliably on bash, zsh, sh, dash, ksh, and other shells.

## Why POSIX Compliance Matters

### Portability Benefits

```bash
# ✓ POSIX-compliant - works everywhere
if [ "$var" = "value" ]; then
    echo "Match found"
fi

# ✗ Bash-specific - fails on sh, dash
if [[ "$var" == "value" ]]; then
    echo "Match found"
fi
```

**Compatibility Matrix:**

| Feature | sh | bash | zsh | dash | ksh | POSIX? |
|---------|----|----|-----|------|-----|--------|
| `[ ]` test | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `[[ ]]` test | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ |
| Arrays | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ |
| `$(cmd)` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `` `cmd` `` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (deprecated) |
| Process substitution | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ |

## Common POSIX Violations and Fixes

### 1. Test Conditionals

**Non-POSIX (Bash):**
```bash
if [[ $file == *.txt ]]; then
    echo "Text file"
fi

if [[ $count -gt 10 && $count -lt 20 ]]; then
    echo "In range"
fi
```

**POSIX Alternative:**
```bash
# Use single [ ] with = instead of ==
case "$file" in
    *.txt) echo "Text file" ;;
esac

# Split compound conditions
if [ "$count" -gt 10 ] && [ "$count" -lt 20 ]; then
    echo "In range"
fi

# Or use -a/-o (deprecated but POSIX)
if [ "$count" -gt 10 -a "$count" -lt 20 ]; then
    echo "In range"
fi
```

### 2. String Comparison

**Non-POSIX:**
```bash
if [[ "$string" == "value" ]]; then
    echo "Match"
fi

if [[ -z "$var" ]]; then  # This one is actually fine
    echo "Empty"
fi
```

**POSIX:**
```bash
# Use single = for equality
if [ "$string" = "value" ]; then
    echo "Match"
fi

# -z works in POSIX [ ]
if [ -z "$var" ]; then
    echo "Empty"
fi

# Multiple conditions - use separate [ ]
if [ -n "$var" ] && [ "$var" = "value" ]; then
    echo "Non-empty and matches"
fi
```

### 3. Arrays

**Non-POSIX (Bash):**
```bash
# Bash arrays
files=(file1.txt file2.txt file3.txt)
echo "${files[0]}"
echo "${files[@]}"

# Associative arrays
declare -A config
config[key]="value"
```

**POSIX Alternative:**
```bash
# Use space-separated strings
files="file1.txt file2.txt file3.txt"

# Iterate with for loop
for file in $files; do
    echo "$file"
done

# Or use positional parameters
set -- file1.txt file2.txt file3.txt
echo "$1"  # First file
echo "$@"  # All files

# For key-value storage, use case or separate variables
case "$key" in
    setting1) value="value1" ;;
    setting2) value="value2" ;;
esac
```

### 4. Process Substitution

**Non-POSIX (Bash):**
```bash
# Process substitution
diff <(sort file1.txt) <(sort file2.txt)

# Read from process
while read line; do
    echo "$line"
done < <(find . -name "*.txt")
```

**POSIX Alternative:**
```bash
# Use temporary files
sort file1.txt > /tmp/sorted1.$$
sort file2.txt > /tmp/sorted2.$$
diff /tmp/sorted1.$$ /tmp/sorted2.$$
rm /tmp/sorted1.$$ /tmp/sorted2.$$

# Or use pipes
find . -name "*.txt" | while read line; do
    echo "$line"
done

# Note: pipe creates subshell, variables don't persist
# Use temporary file if you need to preserve state
```

### 5. String Manipulation

**Non-POSIX (Bash):**
```bash
# Parameter expansion
filename="document.txt"
echo "${filename%.txt}"      # Remove suffix
echo "${filename#prefix-}"   # Remove prefix
echo "${filename/old/new}"   # Substitution
echo "${#filename}"          # Length

# Uppercase/lowercase
echo "${var^^}"  # Uppercase
echo "${var,,}"  # Lowercase
```

**POSIX Alternative:**
```bash
# Basic suffix/prefix removal works in POSIX
filename="document.txt"
echo "${filename%.txt}"      # ✓ POSIX - Remove shortest suffix
echo "${filename#prefix-}"   # ✓ POSIX - Remove shortest prefix
echo "${filename##*/}"       # ✓ POSIX - Remove longest prefix (basename)
echo "${filename%/*}"        # ✓ POSIX - Remove longest suffix (dirname)

# Substitution - use external commands
echo "$filename" | sed 's/old/new/'

# Length - use external command
echo "$filename" | wc -c

# Case conversion - use tr
echo "$var" | tr '[:lower:]' '[:upper:]'  # Uppercase
echo "$var" | tr '[:upper:]' '[:lower:]'  # Lowercase
```

### 6. Arithmetic

**Non-POSIX (Bash):**
```bash
# Bash arithmetic
((count++))
((total = count * 2))
echo $((count + 1))

# Arithmetic comparison
if ((count > 10)); then
    echo "Greater"
fi
```

**POSIX Alternative:**
```bash
# Use $(( )) for arithmetic (POSIX)
count=$((count + 1))
total=$((count * 2))
echo $((count + 1))

# Comparison - use test [ ]
if [ "$count" -gt 10 ]; then
    echo "Greater"
fi

# Or use expr (older POSIX)
count=$(expr $count + 1)
total=$(expr $count \* 2)
```

### 7. Here-Strings

**Non-POSIX (Bash):**
```bash
# Here-string
grep "pattern" <<< "$variable"

# Feed string to command
cat <<< "Hello, World"
```

**POSIX Alternative:**
```bash
# Use echo and pipe
echo "$variable" | grep "pattern"

# Or printf (more robust)
printf '%s\n' "$variable" | grep "pattern"

# Here-document (POSIX)
cat << EOF
Hello, World
EOF
```

### 8. Extended Globbing

**Non-POSIX (Bash):**
```bash
# Bash extended globbing
shopt -s extglob
rm !(important.txt)         # Delete all except
ls *(*.jpg|*.png)          # Match multiple patterns
find . -name "@(*.jpg|*.png)"
```

**POSIX Alternative:**
```bash
# Use find with -not/-o
find . -type f -not -name "important.txt" -delete
find . -name "*.jpg" -o -name "*.png"

# Or multiple commands
for ext in jpg png; do
    ls *.$ext
done

# Case pattern matching
for file in *; do
    case "$file" in
        *.jpg|*.png) echo "$file" ;;
    esac
done
```

### 9. Local Variables

**Non-POSIX (Bash):**
```bash
function my_func() {
    local var="value"
    echo "$var"
}
```

**POSIX Alternative:**
```bash
# POSIX doesn't have 'local', use function-scoped approach
my_func() {
    # Best practice: prefix variables to avoid conflicts
    _my_func_var="value"
    echo "$_my_func_var"
    # Or carefully manage scope
}

# Alternatively, use subshell
my_func() (
    var="value"  # Only exists in subshell
    echo "$var"
)
```

### 10. Bash-isms in Find

**Non-POSIX (GNU find):**
```bash
# GNU find extensions
find . -name "*.txt" -printf "%f\n"  # Print just filename
find . -name "*.log" -delete         # Direct delete
find . -maxdepth 2 -name "*.py"      # Depth limit
```

**POSIX Alternative:**
```bash
# Print just filename - use basename
find . -name "*.txt" -exec basename {} \;

# Delete - use -exec rm
find . -name "*.log" -exec rm {} \;

# Or use -delete (widely supported, technically not POSIX)
# For strict POSIX, use -exec

# Depth limit - no POSIX equivalent
# Use explicit path patterns or prune
find . -name "*.py" | grep -v '/.*/.*/.*/'  # Hacky depth limit
```

## POSIX-Compliant Command Patterns

### File Operations

```bash
# ✓ Create directory with parents
mkdir -p /path/to/directory

# ✓ Copy with directory structure
cp -r source/ destination/

# ✓ Move files
mv source destination

# ✓ Remove files
rm -f file.txt
rm -rf directory/  # Recursive remove

# ✓ Link files
ln -s target linkname  # Symbolic link
ln target linkname     # Hard link
```

### Text Processing

```bash
# ✓ Search files
grep "pattern" file.txt
grep -r "pattern" directory/

# ✓ Stream editing
sed 's/old/new/g' file.txt
sed -i.bak 's/old/new/g' file.txt  # In-place with backup

# ✓ Field processing
awk '{print $1}' file.txt
awk -F: '{print $1}' /etc/passwd

# ✓ Sorting and uniqueness
sort file.txt
sort -u file.txt  # Sort and unique
uniq file.txt     # Remove adjacent duplicates
```

### Finding Files

```bash
# ✓ Find by name
find /path -name "*.txt"

# ✓ Find by type
find /path -type f        # Regular files
find /path -type d        # Directories

# ✓ Find by time
find /path -mtime -7      # Modified within 7 days
find /path -mtime +30     # Modified more than 30 days ago

# ✓ Find by size
find /path -size +10M     # Larger than 10MB
find /path -size -1k      # Smaller than 1KB

# ✓ Execute command on results
find /path -name "*.log" -exec rm {} \;
find /path -name "*.txt" -exec grep "pattern" {} +
```

### Conditionals and Loops

```bash
# ✓ If statements
if [ -f "$file" ]; then
    echo "File exists"
fi

if [ "$var" = "value" ]; then
    echo "Match"
elif [ "$var" = "other" ]; then
    echo "Other match"
else
    echo "No match"
fi

# ✓ Case statements
case "$var" in
    pattern1) echo "Match 1" ;;
    pattern2|pattern3) echo "Match 2 or 3" ;;
    *) echo "Default" ;;
esac

# ✓ While loops
while [ "$count" -lt 10 ]; do
    echo "$count"
    count=$((count + 1))
done

# ✓ For loops
for file in *.txt; do
    echo "$file"
done

for i in 1 2 3 4 5; do
    echo "$i"
done
```

## Shell Portability Checklist

When cmdai generates commands, it ensures:

- [ ] Uses `[ ]` instead of `[[ ]]` for tests
- [ ] Uses `=` instead of `==` for string comparison
- [ ] Avoids arrays (uses space-separated strings)
- [ ] Avoids process substitution (`<()`)
- [ ] Quotes all variable expansions: `"$var"`
- [ ] Uses `$(command)` instead of backticks
- [ ] Avoids `local` keyword in functions
- [ ] Uses standard utilities (no GNU-specific flags)
- [ ] Tests with `/bin/sh` not just `/bin/bash`

## Testing POSIX Compliance

```bash
# Test your script with POSIX shell
sh script.sh          # System sh (often dash or bash in POSIX mode)
dash script.sh        # Debian Almquist Shell (strict POSIX)

# Use shellcheck for validation
shellcheck script.sh

# Check for bash-isms
checkbashisms script.sh  # From devscripts package

# Run in POSIX mode
bash --posix script.sh
```

## Common Gotchas

### 1. Empty Variable Expansion

```bash
# ✗ Fails if var is empty
[ $var = "value" ]

# ✓ Always quote
[ "$var" = "value" ]
```

### 2. Word Splitting

```bash
# ✗ Splits on spaces
for file in $(ls *.txt); do

# ✓ Use globbing directly
for file in *.txt; do

# ✓ Or use find
find . -name "*.txt" -print0 | while IFS= read -r -d '' file; do
```

### 3. Command Substitution

```bash
# ✗ Deprecated (but still POSIX)
var=`command`

# ✓ Preferred
var=$(command)
```

### 4. Function Definition

```bash
# ✓ POSIX style (preferred)
funcname() {
    commands
}

# ✗ Bash style (avoid)
function funcname {
    commands
}

# ✗ Combined (really avoid)
function funcname() {
    commands
}
```

## Benefits of POSIX Compliance

1. **Portability**: Works on any Unix-like system
2. **Reliability**: Fewer surprising behaviors
3. **Compatibility**: Works in restricted environments (Docker, embedded)
4. **Maintainability**: More developers understand POSIX than bash-isms
5. **Security**: Fewer parsing edge cases
6. **Longevity**: POSIX standards are stable

## When to Break POSIX

It's acceptable to use non-POSIX features when:
- Target environment is known (e.g., "bash-only script")
- POSIX alternative is significantly more complex
- Performance requires specific shell features
- Script explicitly requires bash (`#!/bin/bash` shebang)

**But document it!**
```bash
#!/bin/bash
# Requires: bash 4.0+
# Reason: Uses associative arrays for configuration
```

## Resources

- **POSIX Shell Command Language**: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html
- **POSIX Utilities**: https://pubs.opengroup.org/onlinepubs/9699919799/idx/utilities.html
- **ShellCheck**: https://www.shellcheck.net/
- **Bash POSIX Mode**: https://www.gnu.org/software/bash/manual/html_node/Bash-POSIX-Mode.html

## Summary

cmdai prioritizes POSIX compliance to ensure:
- ✅ Maximum portability across systems
- ✅ Predictable behavior
- ✅ Compatibility with all major shells
- ✅ Security through standard patterns
- ✅ Long-term maintainability

When in doubt, choose the POSIX-compliant approach.
