#!/bin/bash
#
# setup-autocoder.sh - Install and configure Autocoder for caro development
#
# Usage: ./tools/setup-autocoder.sh
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARO_ROOT="$(dirname "$SCRIPT_DIR")"
AUTOCODER_DIR="$SCRIPT_DIR/autocoder"

echo "=== Autocoder Setup for Caro ==="
echo "Root: $CARO_ROOT"
echo ""

# Check prerequisites
check_prereqs() {
    echo "Checking prerequisites..."

    if ! command -v python3 &> /dev/null; then
        echo "Error: Python 3 is required"
        exit 1
    fi

    if ! command -v npm &> /dev/null; then
        echo "Warning: npm not found - web UI will not work"
    fi

    if ! command -v git &> /dev/null; then
        echo "Error: git is required"
        exit 1
    fi

    echo "Prerequisites OK"
    echo ""
}

# Clone autocoder
clone_autocoder() {
    if [ -d "$AUTOCODER_DIR" ]; then
        echo "Autocoder already exists at $AUTOCODER_DIR"
        echo "Updating..."
        cd "$AUTOCODER_DIR"
        git pull origin main || echo "Update failed - continuing with existing version"
    else
        echo "Cloning autocoder..."
        git clone https://github.com/leonvanzyl/autocoder.git "$AUTOCODER_DIR"
    fi
    echo ""
}

# Install Python dependencies
install_python_deps() {
    echo "Installing Python dependencies..."
    cd "$AUTOCODER_DIR"

    if [ -f "requirements.txt" ]; then
        python3 -m pip install -r requirements.txt --quiet
    else
        echo "No requirements.txt found - installing core dependencies"
        python3 -m pip install fastapi uvicorn sqlalchemy --quiet
    fi

    echo "Python dependencies installed"
    echo ""
}

# Install frontend dependencies
install_frontend_deps() {
    if ! command -v npm &> /dev/null; then
        echo "Skipping frontend (npm not available)"
        return
    fi

    echo "Installing frontend dependencies..."
    if [ -d "$AUTOCODER_DIR/frontend" ]; then
        cd "$AUTOCODER_DIR/frontend"
        npm install --silent 2>/dev/null || echo "Frontend install had warnings"
    fi

    echo "Frontend dependencies installed"
    echo ""
}

# Create caro-specific configuration
create_config() {
    echo "Creating caro-specific configuration..."

    mkdir -p "$AUTOCODER_DIR/projects"

    cat > "$AUTOCODER_DIR/projects/caro.json" << 'EOF'
{
  "name": "caro",
  "description": "Natural language to shell commands CLI",
  "path": "/home/user/caro",
  "language": "rust",
  "test_command": "cargo test --quiet",
  "build_command": "cargo build",
  "lint_command": "cargo clippy -- -D warnings",
  "format_command": "cargo fmt --check",
  "pre_commit_checks": [
    "cargo fmt --check",
    "cargo clippy -- -D warnings",
    "cargo test --quiet"
  ],
  "safety_rules": {
    "allowed_directories": [
      "src/",
      "tests/",
      "benches/",
      "specs/",
      "kitty-specs/",
      "docs/",
      ".claude/",
      ".specify/",
      ".kittify/"
    ],
    "forbidden_commands": [
      "rm -rf /",
      "sudo rm",
      "> /dev/sda",
      "curl | bash"
    ]
  },
  "agent_instructions": {
    "tdd_required": true,
    "msrv": "1.83",
    "commit_style": "conventional",
    "branch_prefix": "claude/"
  }
}
EOF

    echo "Configuration created"
    echo ""
}

# Create bridge script
create_bridge_script() {
    echo "Creating spec-kitty bridge script..."

    cat > "$SCRIPT_DIR/spec-to-autocoder.py" << 'PYTHON'
#!/usr/bin/env python3
"""
spec-to-autocoder.py - Bridge Spec-Kitty tasks to Autocoder

Converts .specify/ and specs/ task files to Autocoder's SQLite format.

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

SCRIPT_DIR = Path(__file__).parent
CARO_ROOT = SCRIPT_DIR.parent
SPECS_DIR = CARO_ROOT / "specs"
KITTY_SPECS_DIR = CARO_ROOT / "kitty-specs"
AUTOCODER_DB = SCRIPT_DIR / "autocoder/data/features.db"


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
        conn.execute(
            'UPDATE features SET name = ?, updated_at = ? WHERE id = ?',
            (feature_name, datetime.now(), feature_id)
        )
    else:
        cursor = conn.execute(
            'INSERT INTO features (name, source) VALUES (?, ?)',
            (feature_name, str(spec_dir))
        )
        feature_id = cursor.lastrowid

    # Parse and sync tasks
    tasks = parse_tasks_md(tasks_file)

    conn.execute('DELETE FROM tasks WHERE feature_id = ?', (feature_id,))

    for task in tasks:
        status = 'completed' if task['completed'] else 'pending'
        conn.execute(
            '''INSERT INTO tasks
               (feature_id, task_id, description, status, parallel)
               VALUES (?, ?, ?, ?, ?)''',
            (feature_id, task['id'], task['description'], status, task['parallel'])
        )

    conn.commit()
    print(f"  {feature_name}: {len(tasks)} tasks")
    return len(tasks)


def sync_all(conn: sqlite3.Connection):
    """Sync all spec directories."""
    total = 0

    for specs_dir in [SPECS_DIR, KITTY_SPECS_DIR]:
        if specs_dir.exists():
            print(f"Scanning {specs_dir}...")
            for spec_dir in sorted(specs_dir.iterdir()):
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
PYTHON

    chmod +x "$SCRIPT_DIR/spec-to-autocoder.py"
    echo "Bridge script created"
    echo ""
}

# Create convenience scripts
create_launcher_scripts() {
    echo "Creating launcher scripts..."

    # Dashboard launcher
    cat > "$SCRIPT_DIR/autocoder-dashboard.sh" << 'EOF'
#!/bin/bash
# Launch Autocoder web dashboard
cd "$(dirname "$0")/autocoder"
./start_ui.sh 2>/dev/null || {
    echo "Starting manually..."
    uvicorn main:app --reload --port 8000 &
    cd frontend && npm run dev
}
EOF
    chmod +x "$SCRIPT_DIR/autocoder-dashboard.sh"

    # Agent launcher
    cat > "$SCRIPT_DIR/autocoder-start.sh" << 'EOF'
#!/bin/bash
# Launch Autocoder agent for caro development
cd "$(dirname "$0")/autocoder"
./start.sh "$@"
EOF
    chmod +x "$SCRIPT_DIR/autocoder-start.sh"

    echo "Launcher scripts created"
    echo ""
}

# Print summary
print_summary() {
    echo "=== Setup Complete ==="
    echo ""
    echo "Installed to: $AUTOCODER_DIR"
    echo ""
    echo "Quick Start:"
    echo "  1. Sync existing specs:"
    echo "     python tools/spec-to-autocoder.py sync"
    echo ""
    echo "  2. Start web dashboard:"
    echo "     ./tools/autocoder-dashboard.sh"
    echo "     Open http://localhost:5173"
    echo ""
    echo "  3. Launch autonomous agent:"
    echo "     ./tools/autocoder-start.sh"
    echo ""
    echo "  4. Check progress:"
    echo "     python tools/spec-to-autocoder.py status"
    echo ""
    echo "Documentation: specs/autocoder-integration/spec.md"
}

# Main
main() {
    cd "$CARO_ROOT"

    check_prereqs
    clone_autocoder
    install_python_deps
    install_frontend_deps
    create_config
    create_bridge_script
    create_launcher_scripts
    print_summary
}

main "$@"
