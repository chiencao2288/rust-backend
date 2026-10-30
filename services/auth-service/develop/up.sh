#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "🚀 Starting local development infrastructure (PostgreSQL 18 + Redis)..."

# Ensure SELinux container volume permissions on Fedora/RHEL
chcon -t container_file_t -R "$DIR/../migrations" 2>/dev/null || true

# Play the Kubernetes pod manifest via Podman
podman kube play --replace kube.yaml

echo "✅ Dev infrastructure is UP and running!"
echo "   - PostgreSQL 18: localhost:5432 (User: postgres, DB: auth_db)"
echo "   - Redis:         localhost:6379"
echo "   - Auto-migrated schema: users, identities, refresh_tokens (uuidv7)"
