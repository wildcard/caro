# Caro Demos

This directory contains all demo materials for Caro.sh presentations and marketing.

## Directory Structure

```
demos/
├── demo.sh              # Main demo management script
├── asciinema/          # Asciinema recordings and scripts
│   ├── README.md       # Asciinema-specific documentation
│   ├── *.sh            # Demo execution scripts
│   └── *.cast          # Recorded demos
├── vhs/                # VHS tape files for automated recordings
│   ├── *.tape          # VHS configuration files
│   └── *.gif           # Generated GIFs
├── playground/         # Node.js web app demo environment
├── sysadmin-playground/ # Server ops demo environment
└── docs/               # Documentation and notes
```

## Demo Types

### 1. Vancouver.Dev Event (vancouver)
**Purpose**: Community presentation happening tomorrow  
**Duration**: ~60 seconds  
**Audience**: Local developers at Vancouver.Dev meetup  
**Content**: Full feature showcase with 4 working examples

**Run with**:
```bash
./demo.sh record vancouver
./demo.sh play vancouver
```

### 2. Website Hero (website)
**Purpose**: Homepage for caro.sh (repo: wildcard/carosh)  
**Duration**: ~30 seconds  
**Audience**: Website visitors  
**Content**: Quick, impressive demo for immediate impact

**Repository**: github.com/wildcard/carosh  
**Domain**: caro.sh

**Run with**:
```bash
./demo.sh record website
./demo.sh play website
```

### 3. SysAdmin/DevOps (sysadmin)
**Purpose**: Technical operations audience  
**Duration**: ~45 seconds  
**Audience**: SysAdmins, DevOps engineers, SREs  
**Content**: Log analysis and operations tasks

**Run with**:
```bash
./demo.sh record sysadmin
./demo.sh play sysadmin
```

## Quick Start

### Record a Demo

```bash
# Record Vancouver demo
./demo.sh record vancouver

# Record website demo  
./demo.sh record website

# Record all demos
./demo.sh record-all
```

### Play Back

```bash
./demo.sh play vancouver
```

### Generate GIF

```bash
# Install agg first
cargo install agg

# Generate GIF
./demo.sh gif vancouver

# Generate all GIFs
./demo.sh gif-all
```

### Upload to asciinema.org

```bash
./demo.sh upload vancouver
```

## Requirements

- **asciinema**: `brew install asciinema`
- **agg** (for GIFs): `cargo install agg`
- **Binary**: Build with `cargo build --release --features embedded-mlx`

## Demo Management Script

The `demo.sh` script provides all demo operations:

```bash
# List available demos
./demo.sh list

# Record a specific demo
./demo.sh record <demo-id>

# Play back a recording
./demo.sh play <demo-id>

# Upload to asciinema.org
./demo.sh upload <demo-id>

# Generate GIF
./demo.sh gif <demo-id>

# Record all demos
./demo.sh record-all

# Generate all GIFs
./demo.sh gif-all
```

## Technology Separation

### Asciinema (`asciinema/`)
- Real terminal recordings
- Can be played back with `asciinema play`
- Upload to asciinema.org for embedding
- Best for: Website embeds, sharing online

### VHS (`vhs/`)
- Automated tape-based recordings
- Generates GIFs directly
- Requires VHS: `brew install vhs`
- Best for: README files, documentation

## Embedding Demos

### On caro.sh Website

After uploading to asciinema.org:

```html
<script src="https://asciinema.org/a/YOUR_RECORDING_ID.js" 
        id="asciicast-YOUR_RECORDING_ID" 
        async>
</script>
```

Or self-host:

```html
<link rel="stylesheet" type="text/css" href="/asciinema-player.css" />
<div id="demo"></div>
<script src="/asciinema-player.min.js"></script>
<script>
  AsciinemaPlayer.create('/demos/website-demo.cast', 
                         document.getElementById('demo'));
</script>
```

## Notes

- **Repo name**: wildcard/carosh (no .sh in repo name)
- **Domain**: caro.sh (with .sh)
- All scripts automatically find the binary
- First run will be slow (model loading ~2-3s)
- Subsequent runs are fast (<1s)

## Troubleshooting

### Binary not found
```bash
cd ..
cargo build --release --features embedded-mlx
```

### Asciinema not installed
```bash
brew install asciinema
```

### GIF generation fails
```bash
cargo install agg
```

### Script can't find binary
Scripts look in:
1. `caro` command (if alias exists)
2. `cmdai` command (if installed globally)
3. `../target/release/cmdai`
4. `../target/debug/cmdai`

## Contributing

When adding new demos:
1. Create script in `asciinema/`
2. Add entry to `demo.sh` DEMOS array
3. Test with `./demo.sh record <id>`
4. Update this README

## Contact

Questions? Issues? Open a GitHub issue on wildcard/carosh
