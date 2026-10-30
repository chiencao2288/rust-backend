#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "🛑 Stopping local development infrastructure..."
podman kube down kube.yaml
echo "✅ Dev infrastructure stopped."
