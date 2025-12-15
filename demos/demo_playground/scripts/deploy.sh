#!/bin/bash
# Deployment script for production

set -e

echo "Starting deployment..."

# Build release binary
cargo build --release

# Run tests
cargo test

# Stop existing service
systemctl stop webapi

# Copy new binary
cp target/release/web-api /usr/local/bin/

# Start service
systemctl start webapi

echo "Deployment completed!"
