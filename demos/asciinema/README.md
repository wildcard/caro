# Asciinema Demo Scripts

These scripts are designed to be recorded with asciinema for the Caro.sh project.

## Available Demos

### 1. Vancouver Dev Demo (`vancouver-dev-demo.sh`)
**Duration:** ~3-4 minutes  
**Purpose:** Community presentation with full feature showcase  
**Context:** In `demos/playground/` directory  
**Usage:**
```bash
cd demos/playground
asciinema rec vancouver-dev-demo.cast -c "../vancouver-dev-demo.sh"
```

### 2. Website Hero Demo (`website-hero-demo.sh`)
**Duration:** ~1 minute  
**Purpose:** Quick, impactful demo for website embedding  
**Context:** In `demos/playground/` directory  
**Usage:**
```bash
cd demos/playground
asciinema rec website-hero-demo.cast -c "../website-hero-demo.sh"
```

### 3. SysAdmin Demo (`sysadmin-demo.sh`)
**Duration:** ~2 minutes  
**Purpose:** Targeted for SysAdmins/DevOps/SRE audience  
**Context:** In `demos/sysadmin-playground/` directory  
**Usage:**
```bash
cd demos/sysadmin-playground
asciinema rec sysadmin-demo.cast -c "../sysadmin-demo.sh"
```

## Recording Instructions

### Before Recording

**The scripts automatically detect the caro binary!** They check for:
1. `caro` command (if you have the alias)
2. `caro` command (if installed globally)
3. `../target/release/caro` (release build)
4. `../target/debug/caro` (debug build)

**You just need to:**
1. Build caro: `cargo build --release` (or `cargo build` for debug)
2. Navigate to the appropriate playground directory (see each demo's "Context" above)

### Recording Commands

**Standard recording:**
```bash
asciinema rec <demo-name>.cast -c "./<script-name>.sh"
```

**With custom settings (recommended):**
```bash
asciinema rec <demo-name>.cast \
  --cols 120 \
  --rows 30 \
  -c "./<script-name>.sh"
```

**Interactive recording (manual typing):**
```bash
asciinema rec <demo-name>.cast
# Then manually run commands from the script
```

### Publishing to Asciinema

```bash
# Upload to asciinema.org
asciinema upload <demo-name>.cast

# Get shareable link and embed code
```

### Embedding in Website

After uploading, you'll get embed code like:
```html
<script src="https://asciinema.org/a/<id>.js" id="asciicast-<id>" async></script>
```

Or use asciinema-player for self-hosted:
```html
<link rel="stylesheet" type="text/css" href="/asciinema-player.css" />
<div id="demo"></div>
<script src="/asciinema-player.min.js"></script>
<script>
  AsciinemaPlayer.create('/path/to/demo.cast', document.getElementById('demo'));
</script>
```

## Tips

- **Speed:** Scripts include `sleep` commands for pacing. Adjust if needed.
- **Editing:** You can edit `.cast` files (JSON format) to adjust timing.
- **Terminal size:** Use `--cols 120 --rows 30` for consistent sizing.
- **Theme:** Configure your terminal theme before recording for visual consistency.

## Recommended Terminal Settings

- **Font:** Monospace, 14-16px
- **Theme:** Dracula, Nord, or Tokyo Night (high contrast)
- **Window:** Full-screen or at least 120x30 chars
- **Shell prompt:** Keep simple (PS1 should just show `$` or similar)

## Quick Test

To test your setup:
```bash
cd demos/playground

# The script will auto-detect your caro binary
../vancouver-dev-demo.sh

# Or test manually:
caro "list files"
# Or if you have the alias: caro "list files"
```

If caro runs and generates a command, you're ready to record!
