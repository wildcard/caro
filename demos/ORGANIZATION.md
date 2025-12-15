# Demos Organization Complete ✅

## Structure

```
demos/
├── demo.sh              # Main demo management script
├── README.md            # Complete documentation
│
├── asciinema/          # Asciinema recordings
│   ├── README.md       # Recording instructions
│   ├── *.sh            # Executable demo scripts
│   └── *.cast          # Recorded sessions
│
├── vhs/                # VHS tape files
│   ├── *.tape          # VHS configurations
│   └── *.gif           # Generated GIFs
│
├── docs/               # All documentation
│   ├── BRANCH_SUMMARY.md
│   ├── DEMO_READY.md
│   ├── QUICK_START.md
│   ├── RETRY_LOGIC.md
│   └── ...
│
├── playground/         # Node.js web app demo environment
└── sysadmin-playground/ # Server ops demo environment
```

## Quick Start

```bash
cd demos

# List available demos
./demo.sh list

# Record Vancouver demo (for tomorrow's event)
./demo.sh record vancouver

# Record website demo (for caro.sh homepage)
./demo.sh record website

# Play back a recording
./demo.sh play vancouver

# Generate GIF
./demo.sh gif vancouver

# Upload to asciinema.org
./demo.sh upload vancouver

# Record all demos
./demo.sh record-all
```

## Demo Types

### 1. vancouver
- **Purpose**: Vancouver.Dev community event presentation (tomorrow)
- **Duration**: ~60 seconds
- **Audience**: Local developers
- **Content**: Full feature showcase

### 2. website
- **Purpose**: Homepage hero section for caro.sh
- **Duration**: ~30 seconds
- **Audience**: Website visitors
- **Content**: Quick, impressive demo

### 3. sysadmin
- **Purpose**: Technical operations audience
- **Duration**: ~45 seconds
- **Audience**: SysAdmins, DevOps, SREs
- **Content**: Log analysis and operations

## Repository Info

- **GitHub**: https://github.com/wildcard/caro
- **Domain**: caro.sh
- **Note**: Repo name is "caro" (no .sh), domain is "caro.sh" (with .sh)

## Requirements

- **asciinema**: `brew install asciinema`
- **agg** (for GIFs): `cargo install agg`
- **Binary**: `cargo build --release --features embedded-mlx`

## Root Directory

Clean and organized! All demo-related files are now in `demos/` folder.
No pollution in the project root.

## Next Steps

1. Record demos for tomorrow's event
2. Upload to asciinema.org
3. Embed on caro.sh website
4. Generate GIFs for README and social media
