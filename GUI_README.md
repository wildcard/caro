# cmdai GUI

A comprehensive desktop GUI for cmdai built with Tauri that provides an enhanced user experience for managing shell command generation, execution history, and community feedback.

## Features

### 🚀 Command Generation
- **Natural Language Input**: Convert plain English to shell commands
- **Dry Run Mode**: Inspect generated commands before execution
- **Multi-Shell Support**: bash, zsh, fish, POSIX sh
- **Safety Levels**: Strict, Moderate, Permissive
- **Real-time Execution**: Execute commands and see output in the GUI
- **Alternative Suggestions**: Get multiple command options
- **Risk Assessment**: Visual indicators for command safety

### 📜 Execution History
- **Comprehensive Search**: Filter by shell type, risk level, execution status
- **Detailed View**: See full command details, explanations, and warnings
- **Delete Records**: Remove unwanted history entries
- **Persistent Storage**: SQLite database for reliable data storage

### 👍 Community Feedback
- **Voting System**: Thumbs up/down for command quality
- **5-Star Ratings**: Rate command generation quality
- **Written Feedback**: Add detailed comments about executions
- **Community Insights**: See how others rated similar commands

### 📊 Analytics Dashboard
- **Usage Statistics**: Total executions, success rates, blocked commands
- **Performance Metrics**: Average generation time tracking
- **Risk Distribution**: Visualize safety levels across executions
- **Backend Usage**: See which backends are most used
- **Export Data**: Download history in JSON or CSV format

### ⚙️ Configuration Manager
- **Safety Level**: Configure command validation strictness
- **Default Shell**: Set preferred shell type
- **Log Level**: Control application verbosity
- **Cache Settings**: Manage model cache size
- **Log Rotation**: Configure log file retention

## Getting Started

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+
- Cargo

### Installation

1. **Install Dependencies**:
```bash
npm install
```

2. **Build Backend**:
```bash
cd src-tauri
cargo build
cd ..
```

### Development

Run the GUI in development mode:

```bash
npm run tauri:dev
```

This will:
- Start the Vite dev server for hot-reload
- Launch the Tauri application
- Connect frontend to backend

### Building

Create a production build:

```bash
npm run tauri:build
```

This generates platform-specific installers in `src-tauri/target/release/bundle/`.

## Architecture

### Backend (Rust)

```
src-tauri/
├── src/
│   ├── main.rs          # Application entry point, Tauri setup
│   ├── commands.rs      # IPC command handlers
│   ├── database.rs      # SQLite database operations
│   └── models.rs        # Data structures and types
├── Cargo.toml           # Rust dependencies
└── tauri.conf.json      # Tauri configuration
```

**Key Technologies**:
- **Tauri 2.9**: Desktop application framework
- **rusqlite**: SQLite database integration
- **tokio**: Async runtime
- **serde**: Serialization/deserialization

### Frontend (React + TypeScript)

```
src/
├── components/          # React components
│   ├── CommandTab.tsx   # Command generation interface
│   ├── HistoryTab.tsx   # Execution history browser
│   ├── ConfigTab.tsx    # Settings management
│   └── AnalyticsTab.tsx # Analytics dashboard
├── lib/
│   ├── tauri.ts        # Tauri command wrappers
│   └── utils.ts        # Helper functions
├── App.tsx             # Main application component
├── store.ts            # Zustand state management
├── types.ts            # TypeScript type definitions
└── index.css           # TailwindCSS styles
```

**Key Technologies**:
- **React 18**: UI framework
- **TypeScript**: Type safety
- **Vite**: Build tool and dev server
- **TailwindCSS**: Utility-first CSS
- **Zustand**: State management
- **Lucide React**: Icon library

### Database Schema

**execution_history**
- Stores all command generation records
- Includes prompt, command, timing, safety info
- Indexed for fast queries

**execution_ratings**
- 5-star ratings for executions
- Optional written feedback
- Foreign key to execution_history

**execution_votes**
- Simple up/down votes
- Tracks vote timestamps
- Foreign key to execution_history

## Usage Guide

### Generating Commands

1. Navigate to the **Command** tab
2. Enter your task description in natural language
3. Select shell type and safety level
4. **Enable Dry Run** to inspect without executing
5. Click **Generate Command**
6. Review the generated command, explanation, and warnings
7. Click the execute button or copy to use elsewhere

### Viewing History

1. Navigate to the **History** tab
2. Use the search box to filter by text
3. Click the filter icon for advanced filters
4. Select an execution to see full details
5. Vote or rate executions you find useful

### Managing Configuration

1. Navigate to the **Config** tab
2. Adjust settings as needed
3. Click **Save Changes** to persist
4. Use **Reload** to revert unsaved changes

### Viewing Analytics

1. Navigate to the **Analytics** tab
2. View usage statistics and metrics
3. Export history using JSON or CSV buttons
4. Monitor performance trends

## Tips & Best Practices

### Safety First
- Always use **Dry Run mode** for unfamiliar commands
- Review warnings carefully before execution
- Set appropriate safety level for your use case

### History Management
- Rate commands to help others
- Add feedback on particularly good or bad generations
- Regularly export history for backup

### Performance
- Keep cache size reasonable based on disk space
- Adjust log rotation to balance debugging needs and disk usage
- Monitor average generation time in analytics

## Troubleshooting

### GUI Won't Start
```bash
# Check Node.js version
node --version  # Should be 18+

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Rebuild Tauri
cd src-tauri && cargo clean && cargo build
```

### Database Errors
The database is stored in the application data directory:
- **Linux**: `~/.local/share/cmdai/cmdai.db`
- **macOS**: `~/Library/Application Support/cmdai/cmdai.db`
- **Windows**: `%APPDATA%\cmdai\cmdai.db`

Delete this file to reset the database (will lose history).

### Command Execution Fails
- Ensure the shell is properly configured on your system
- Check that the generated command syntax is valid
- Review execution warnings and blocked reasons

## Contributing

The GUI is designed to be easily extensible:

1. **Add New Tabs**: Create component in `src/components/`
2. **Add Backend Commands**: Implement in `src-tauri/src/commands.rs`
3. **Add Database Tables**: Update schema in `src-tauri/src/database.rs`
4. **Add Types**: Define in `src/types.ts` and `src-tauri/src/models.rs`

## License

AGPL-3.0 - Same as cmdai core

## Support

For issues, feature requests, or questions:
- Open an issue on GitHub
- Check existing issues for solutions
- Review the main cmdai documentation
