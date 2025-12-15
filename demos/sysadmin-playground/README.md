# Production Server Environment

Internal infrastructure management workspace.

## Structure

- `logs/` - Application and system logs
- `configs/` - Service configuration files
- `scripts/` - Automation and maintenance scripts
- `backups/` - Database and config backups
- `.ssh/` - SSH connection profiles

## Services

- **Web**: nginx reverse proxy (ports 80, 443)
- **App**: Application servers (3 instances)
- **DB**: PostgreSQL 14
- **Cache**: Redis 7
- **Monitoring**: Prometheus + Grafana

## Common Tasks

```bash
# Check service health
./scripts/health-check.sh

# Deploy new version
./scripts/deploy.sh v2.3.1 production

# Run backup
./scripts/backup.sh

# View recent errors
grep ERROR logs/app.log | tail -20
```
