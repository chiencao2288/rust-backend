#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "🚀 Starting noti-service local infrastructure (PostgreSQL 18 + Redis + RabbitMQ)..."

# Ensure SELinux container volume permissions on Fedora/RHEL
chcon -t container_file_t -R "$DIR/../migrations" 2>/dev/null || true

# Play the Kubernetes pod manifest via Podman
podman kube play --replace kube.yaml

echo "✅ Dev infrastructure is UP and running!"
echo "   - PostgreSQL 18:      localhost:5432 (User: postgres, DB: noti_db)"
echo "   - Redis:              localhost:6379"
echo "   - RabbitMQ AMQP:      localhost:5672 (User: guest, Pass: guest)"
echo "   - RabbitMQ Dashboard: http://localhost:15672"
echo "   - Auto-migrated schema: notifications table"
