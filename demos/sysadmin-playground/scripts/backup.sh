#!/bin/bash
# Database backup script
# Runs daily via cron

set -euo pipefail

BACKUP_DIR="/var/backups/postgres"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DATABASE="mydb"

echo "[$(date)] Starting backup for $DATABASE..."

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Dump database
pg_dump -U postgres "$DATABASE" | gzip > "$BACKUP_DIR/${DATABASE}_${TIMESTAMP}.sql.gz"

# Keep only last 7 days
find "$BACKUP_DIR" -name "*.sql.gz" -mtime +7 -delete

echo "[$(date)] Backup completed: ${DATABASE}_${TIMESTAMP}.sql.gz"
