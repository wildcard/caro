#!/bin/bash
# Application deployment script

set -e

APP_NAME="myapp"
VERSION="${1:-latest}"
ENVIRONMENT="${2:-production}"

echo "Deploying $APP_NAME:$VERSION to $ENVIRONMENT..."

# Pull latest image
docker pull "$APP_NAME:$VERSION"

# Rolling update
docker service update \
  --image "$APP_NAME:$VERSION" \
  --update-parallelism 2 \
  --update-delay 10s \
  "${APP_NAME}_${ENVIRONMENT}"

echo "Deployment complete!"
