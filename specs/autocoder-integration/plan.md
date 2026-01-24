# Autocoder Integration - Implementation Plan

**Based on**: spec.md
**Focus**: Phase 1 Quick Wins (1-2 weeks)

## Prerequisites

- Claude Code CLI installed and authenticated
- Python 3.10+ available
- Git configured

## Quick Start Setup

### Step 1: Install Autocoder

```bash
# Create tools directory if not exists
mkdir -p tools

# Clone autocoder
git clone https://github.com/leonvanzyl/autocoder.git tools/autocoder

# Enter directory
cd tools/autocoder

# Create environment file
cp .env.example .env 2>/dev/null || touch .env

# The ANTHROPIC_API_KEY will be picked up from your existing Claude auth
# Or set explicitly:
# echo "ANTHROPIC_API_KEY=your-key-here" >> .env
```

### Step 2: Install Dependencies

```bash
# Backend dependencies
cd tools/autocoder
pip install -r requirements.txt

# Frontend dependencies (for web UI)
cd frontend
npm install
cd ..
```

### Step 3: Launch Web UI

```bash
# From tools/autocoder directory
./start_ui.sh

# Or manually:
# Terminal 1: uvicorn main:app --reload --port 8000
# Terminal 2: cd frontend && npm run dev
```

Access dashboard at: http://localhost:5173

## Caro-Specific Configuration

### Create Autocoder Project Config

```bash
# From caro root directory
cat > tools/autocoder/projects/caro.json << 'EOF'
{
  "name": "caro",
  "path": "/home/user/caro",
  "language": "rust",
  "test_command": "cargo test",
  "build_command": "cargo build",
  "lint_command": "cargo clippy -- -D warnings",
  "pre_commit_checks": [
    "cargo fmt --check",
    "cargo clippy -- -D warnings",
    "cargo test"
  ],
  "safety_rules": {
    "allowed_directories": [
      "src/",
      "tests/",
      "benches/",
      "specs/",
      "docs/"
    ],
    "forbidden_patterns": [
      "rm -rf /",
      "sudo rm",
      "> /dev/sda"
    ]
  }
}
EOF
```

### Create Spec-Kitty Bridge Script

This converts spec-kitty tasks to autocoder features:

```python
#!/usr/bin/env python3
"""
tools/spec-to-autocoder.py

Bridges Spec-Kitty task files to Autocoder's SQLite database.
Enables autonomous execution of planned features.

Usage:
    python tools/spec-to-autocoder.py sync        # Sync all specs
    python tools/spec-to-autocoder.py sync <dir>  # Sync specific spec
    python tools/spec-to-autocoder.py status      # Show sync status
"""

import sqlite3
import re
import sys
from pathlib import Path
from datetime import datetime

SPECS_DIR = Path(__file__).parent.parent / "specs"
KITTY_SPECS_DIR = Path(__file__).parent.parent / "kitty-specs"
AUTOCODER_DB = Path(__file__).parent / "autocoder/data/features.db"

def parse_tasks_md(tasks_file: Path) -> list[dict]:
    """Parse a spec-kitty tasks.md file into task dicts."""
    tasks = []
    content = tasks_file.read_text()

    # Match task lines: - [ ] T001 [P] Description
    pattern = r'-\s*\[([ x])\]\s*(T\d+)\s*(\[P\])?\s*(.+)'

    for match in re.finditer(pattern, content):
        completed = match.group(1) == 'x'
        task_id = match.group(2)
        parallel = match.group(3) is not None
        description = match.group(4).strip()

        tasks.append({
            'id': task_id,
            'description': description,
            'completed': completed,
            'parallel': parallel,
            'source_file': str(tasks_file)
        })

    return tasks

def init_db(db_path: Path):
    """Initialize the autocoder database if needed."""
    db_path.parent.mkdir(parents=True, exist_ok=True)

    conn = sqlite3.connect(db_path)
    conn.execute('''
        CREATE TABLE IF NOT EXISTS features (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            priority INTEGER DEFAULT 0,
            status TEXT DEFAULT 'pending',
            source TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ''')
    conn.execute('''
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            feature_id INTEGER,
            task_id TEXT,
            description TEXT,
            status TEXT DEFAULT 'pending',
            parallel BOOLEAN DEFAULT 0,
            FOREIGN KEY (feature_id) REFERENCES features(id)
        )
    ''')
    conn.commit()
    return conn

def sync_spec(conn: sqlite3.Connection, spec_dir: Path):
    """Sync a single spec directory to autocoder."""
    tasks_file = spec_dir / "tasks.md"
    spec_file = spec_dir / "spec.md"

    if not tasks_file.exists():
        print(f"  Skipping {spec_dir.name}: no tasks.md")
        return 0

    # Get feature name from spec.md or directory name
    feature_name = spec_dir.name
    if spec_file.exists():
        content = spec_file.read_text()
        if match := re.search(r'^#\s+(.+)$', content, re.MULTILINE):
            feature_name = match.group(1)

    # Check if feature already exists
    existing = conn.execute(
        'SELECT id FROM features WHERE source = ?',
        (str(spec_dir),)
    ).fetchone()

    if existing:
        feature_id = existing[0]
        # Update existing
        conn.execute(
            'UPDATE features SET name = ?, updated_at = ? WHERE id = ?',
            (feature_name, datetime.now(), feature_id)
        )
    else:
        # Create new feature
        cursor = conn.execute(
            'INSERT INTO features (name, source) VALUES (?, ?)',
            (feature_name, str(spec_dir))
        )
        feature_id = cursor.lastrowid

    # Parse and sync tasks
    tasks = parse_tasks_md(tasks_file)

    # Clear existing tasks for this feature
    conn.execute('DELETE FROM tasks WHERE feature_id = ?', (feature_id,))

    # Insert tasks
    for task in tasks:
        status = 'completed' if task['completed'] else 'pending'
        conn.execute(
            '''INSERT INTO tasks
               (feature_id, task_id, description, status, parallel)
               VALUES (?, ?, ?, ?, ?)''',
            (feature_id, task['id'], task['description'], status, task['parallel'])
        )

    conn.commit()
    print(f"  Synced {feature_name}: {len(tasks)} tasks")
    return len(tasks)

def sync_all(conn: sqlite3.Connection):
    """Sync all spec directories."""
    total = 0

    # Sync specs/ directory
    if SPECS_DIR.exists():
        print(f"Scanning {SPECS_DIR}...")
        for spec_dir in sorted(SPECS_DIR.iterdir()):
            if spec_dir.is_dir():
                total += sync_spec(conn, spec_dir)

    # Sync kitty-specs/ directory
    if KITTY_SPECS_DIR.exists():
        print(f"Scanning {KITTY_SPECS_DIR}...")
        for spec_dir in sorted(KITTY_SPECS_DIR.iterdir()):
            if spec_dir.is_dir():
                total += sync_spec(conn, spec_dir)

    return total

def show_status(conn: sqlite3.Connection):
    """Display sync status."""
    features = conn.execute('''
        SELECT f.name, f.source, f.status,
               COUNT(t.id) as total,
               SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END) as done
        FROM features f
        LEFT JOIN tasks t ON f.id = t.feature_id
        GROUP BY f.id
        ORDER BY f.created_at DESC
    ''').fetchall()

    print("\n=== Autocoder Feature Status ===\n")
    print(f"{'Feature':<40} {'Progress':<15} {'Status':<10}")
    print("-" * 70)

    for name, source, status, total, done in features:
        progress = f"{done or 0}/{total or 0}"
        print(f"{name[:40]:<40} {progress:<15} {status:<10}")

    print()

def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 1

    command = sys.argv[1]
    conn = init_db(AUTOCODER_DB)

    if command == "sync":
        if len(sys.argv) > 2:
            spec_dir = Path(sys.argv[2])
            sync_spec(conn, spec_dir)
        else:
            total = sync_all(conn)
            print(f"\nTotal tasks synced: {total}")

    elif command == "status":
        show_status(conn)

    else:
        print(f"Unknown command: {command}")
        print(__doc__)
        return 1

    conn.close()
    return 0

if __name__ == "__main__":
    sys.exit(main())
```

Save as `tools/spec-to-autocoder.py`

## Usage Workflow

### Daily Development with Autocoder

```bash
# Morning: Sync latest specs
python tools/spec-to-autocoder.py sync

# Start autocoder dashboard
cd tools/autocoder && ./start_ui.sh &

# Launch autonomous agent for current sprint
cd tools/autocoder && ./start.sh

# Agent will:
# 1. Pick highest priority feature
# 2. Get next uncompleted task
# 3. Implement with TDD
# 4. Run tests, commit on success
# 5. Mark task complete
# 6. Repeat
```

### Monitor Progress

```bash
# Check status
python tools/spec-to-autocoder.py status

# Or via API
curl http://localhost:8000/api/features/stats

# Or web dashboard
open http://localhost:5173
```

### Pause and Resume

```bash
# Autocoder saves state automatically
# Press Ctrl+C to pause

# Resume later - picks up where it left off
./start.sh
```

## Integration with Existing Workflows

### With Spec-Kitty Commands

```bash
# 1. Create feature spec
/spec-kitty.specify "Add YAML config support"

# 2. Generate implementation plan
/spec-kitty.plan

# 3. Generate tasks
/spec-kitty.tasks

# 4. Sync to autocoder
python tools/spec-to-autocoder.py sync specs/yaml-config-support

# 5. Launch autonomous execution
cd tools/autocoder && ./start.sh
```

### With GitHub Issues

```bash
# Convert GitHub milestone to features
gh issue list --milestone "v1.2.0" --json title,body | \
  python tools/github-to-autocoder.py

# Agent works through milestone items
./start.sh --milestone v1.2.0
```

## Validation Checklist

- [ ] Autocoder cloned to `tools/autocoder/`
- [ ] Dependencies installed (Python + Node)
- [ ] Web UI accessible at localhost:5173
- [ ] Bridge script working (`python tools/spec-to-autocoder.py status`)
- [ ] Test feature synced successfully
- [ ] Agent can pick up and work on tasks

## Troubleshooting

### API Key Issues
```bash
# Check Claude auth
claude --version

# Or set explicitly
export ANTHROPIC_API_KEY="your-key"
```

### Database Reset
```bash
# Clear autocoder state
rm tools/autocoder/data/features.db
python tools/spec-to-autocoder.py sync
```

### Web UI Not Loading
```bash
# Check backend
curl http://localhost:8000/health

# Check frontend
cd tools/autocoder/frontend && npm run dev
```

## Next Steps

After Phase 1 is working:

1. **Phase 2**: Build MCP server for direct Claude Code integration
2. **Phase 2**: Add bidirectional sync (autocoder → spec-kitty)
3. **Phase 3**: Create caro-specialized agent with Rust/safety awareness
