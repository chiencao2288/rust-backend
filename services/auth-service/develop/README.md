# Local Development Setup with Podman

This directory contains the Kubernetes-compliant pod manifest to run the local development stack via Podman.

## Components Started
- **PostgreSQL 18** on port `5432` (`postgres:18` with native `uuidv7()` support)
  - Pre-loaded with database `auth_db`
  - Auto-mounts and executes `../migrations/20261030000001_init_auth_schema.sql` on first boot
- **Redis** on port `6379` (`redis:alpine`)

## Commands

### 1. Start the Dev Stack
```bash
# Option A: Using Podman directly
podman kube play --replace develop/kube.yaml

# Option B: Using the convenience script
bash develop/up.sh

# Option C: Using Make from services/auth-service/
make up
```

### 2. Run the Auth Service
```bash
# From services/auth-service/
make dev

# Or with cargo directly
export DATABASE_MASTER_URL="postgres://postgres:postgres@127.0.0.1:5432/auth_db"
export REDIS_NODES="redis://127.0.0.1:6379"
export PORT=8080
export APP_ENV=development
cargo run
```

### 3. Open Swagger UI in Browser
Open your browser to:
👉 **`http://localhost:8080/swagger-ui/`**

### 4. Stop the Dev Stack
```bash
# Option A: Using Podman directly
podman kube down develop/kube.yaml

# Option B: Using the script
bash develop/down.sh

# Option C: Using Make
make down
```
