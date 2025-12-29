# Quickstart: Caro Sync

This guide walks you through setting up sync between multiple devices.

---

## Prerequisites

- Caro CLI installed (`caro --version` shows 0.2.0+)
- Node.js 20+ installed (for sync daemon)
- Internet connection (for initial setup)

---

## Step 1: Install Sync Daemon

The sync daemon is bundled with Caro but needs a one-time setup:

```bash
# Install sync daemon dependencies
caro sync install

# Verify installation
caro sync doctor
```

Expected output:
```
Sync Daemon Status
------------------
Daemon version:     0.1.0
Node.js version:    v20.10.0
Socket path:        ~/.config/cmdai/sync.sock
Status:             Not initialized

All prerequisites met.
```

---

## Step 2: Initialize Sync on First Device

On your primary device, create a new sync identity:

```bash
caro sync init
```

This will:
1. Generate a 24-word recovery phrase
2. Create encryption keys
3. Connect to Jazz relay
4. Register this device

Output:
```
Creating new sync identity...

============================================
IMPORTANT: Save your recovery phrase!

   abandon ability able about above absent
   absorb abstract absurd abuse access accident
   account accuse achieve acid acoustic acquire
   across act action actor actress actual

This phrase is the ONLY way to recover your
sync data. Store it securely (password manager,
written down in a safe place).
============================================

Press Enter to confirm you've saved the phrase...

Sync initialized!
Device ID:     d3f4a1b2-c5e6-7890-abcd-ef1234567890
Device name:   MacBook Pro
Account ID:    co_zABC123...

Run 'caro sync status' to check sync status.
```

---

## Step 3: Add a Second Device

On your second device, run:

```bash
caro sync join
```

Enter your recovery phrase when prompted:

```
Enter your 24-word recovery phrase:
> abandon ability able about above absent absorb abstract absurd abuse access accident account accuse achieve acid acoustic acquire across act action actor actress actual

Verifying phrase...
Connecting to sync...
Downloading existing data...

Sync joined!
Device ID:     a1b2c3d4-e5f6-7890-ghij-klmnopqrstuv
Device name:   Desktop PC
Synced:        1,234 commands, 12 preferences

Your devices are now synced!
```

---

## Step 4: Verify Sync Works

On Device A, run a command:
```bash
caro "list all files in current directory"
# Generated: ls -la
# Execute? [y/n] y
```

On Device B, check history:
```bash
caro history
```

You should see the command from Device A:
```
Recent Commands
---------------
1. [2 minutes ago] ls -la
   Prompt: "list all files in current directory"
   Device: MacBook Pro

2. [5 minutes ago] git status
   Prompt: "show git status"
   Device: Desktop PC
```

---

## Step 5: Check Sync Status

```bash
caro sync status
```

Output:
```
Sync Status
-----------
Status:        Online
Last sync:     2 seconds ago
Devices:       2 connected

Device List:
  * MacBook Pro (this device)
    Last seen: now

  * Desktop PC
    Last seen: 30 seconds ago

History:
  Total commands: 1,234
  Pending sync:   0 (up to date)
```

---

## Common Operations

### Pause Sync

```bash
caro sync pause
```

### Resume Sync

```bash
caro sync resume
```

### View Sync Logs

```bash
caro sync logs
```

### Reset Sync (removes all synced data)

```bash
caro sync reset --confirm
```

---

## Recovery Scenarios

### Lost Recovery Phrase

If you lose your recovery phrase, you cannot recover synced data from other devices. You must:

1. Run `caro sync reset --confirm` on each device
2. Run `caro sync init` to create a new identity
3. Start fresh (local history is preserved)

### Device Lost/Stolen

Your data is encrypted. A stolen device cannot access your commands without your recovery phrase. However, for peace of mind:

1. Note the lost device's name
2. Run `caro sync device remove <device-name>` from another device
3. Consider rotating your identity if the device was unlocked

### Offline for Extended Period

Sync works offline. When you come back online:

1. Daemon automatically reconnects
2. Queued changes are pushed
3. New changes from other devices are pulled
4. No action needed

---

## Troubleshooting

### Daemon Won't Start

```bash
# Check if daemon is running
caro sync doctor

# View daemon logs
caro sync logs --daemon

# Restart daemon
caro sync restart
```

### Sync Stuck

```bash
# Force sync
caro sync now

# Check for errors
caro sync status --verbose
```

### Connection Issues

```bash
# Test relay connectivity
caro sync ping

# Use different relay (self-hosted)
caro sync config set relay wss://your-relay.example.com
```

---

## Privacy Notes

- Your commands are encrypted before leaving your device
- The sync relay only sees encrypted blobs
- Even Caro maintainers cannot read your commands
- You can self-host the relay for maximum privacy:

```bash
# Run your own Jazz relay
npx jazz-run sync --port 4200

# Configure Caro to use it
caro sync config set relay ws://localhost:4200
```

---

## Next Steps

- [Learn about exclusion patterns](./docs/sync-exclusions.md) to prevent sensitive commands from syncing
- [Set up self-hosted relay](./docs/self-hosted-sync.md) for enterprise use
- [Understand the encryption](./contracts/encryption.md) for security review
