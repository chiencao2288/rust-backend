#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "🛑 Stopping noti-service local infrastructure..."
podman kube down kube.yaml || true
echo "✅ Infrastructure stopped."
