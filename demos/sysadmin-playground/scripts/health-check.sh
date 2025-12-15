#!/bin/bash
# Service health check script

SERVICES=("nginx" "docker" "postgresql" "redis")
FAILED=0

for service in "${SERVICES[@]}"; do
    if systemctl is-active --quiet "$service"; then
        echo "✓ $service is running"
    else
        echo "✗ $service is DOWN"
        FAILED=1
    fi
done

exit $FAILED
