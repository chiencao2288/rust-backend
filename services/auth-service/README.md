# Central Auth Service (`auth-service`)

A centralized Identity and Session Management Service built in Rust using **Axum 0.8**, **PostgreSQL 18** (native `uuidv7()`), and **Redis**.

The service provides sub-millisecond ephemeral token verification for downstream API Gateways and microservices, pre-registration email OTP verification, password hashing with Argon2id, Google OAuth2 authentication, and single-use Refresh Token rotation.

---

## Quick Start

### 1. Prerequisites
- [Rust](https://rustup.rs/) (1.80+)
- [Podman](https://podman.io/) or Docker

### 2. Start Infrastructure (PostgreSQL 18 + Redis)
```bash
make up
```
*Starts PostgreSQL 18 on port `5432` (`auth_db`), Redis on port `6379`, and automatically applies database migrations on initial startup.*

### 3. Run Service
```bash
# Runs in local development mode with default configuration
make dev
```
The service will listen on `http://localhost:8080`.

👉 **Interactive Swagger UI:** **`http://localhost:8080/api/docs/`**

### 4. Stop Infrastructure
```bash
make down
```

### 5. Makefile Commands

| Command | Description |
|---|---|
| `make up` | Starts PostgreSQL 18 and Redis containers via Podman |
| `make down` | Stops and tears down infrastructure containers |
| `make dev` | Runs `auth-service` with local development environment variables |
| `make test` | Executes the complete test suite |
| `make check` | Runs linter checks (`cargo clippy --all-features`) |

---

## Environment Configuration

Configuration is loaded from environment variables (or `.env` in local development):

| Variable | Required | Default | Description |
|---|---|---|---|
| `PORT` | No | `8080` | HTTP listener port |
| `DATABASE_MASTER_URL` | **Yes** | - | Primary PostgreSQL connection URL (writes & transactions) |
| `DATABASE_SLAVE_URL_01` | No | `""` | Optional read replica URL for read queries |
| `REDIS_NODE_01` | **Yes** | - | Primary Redis cluster node URL |
| `REDIS_PASSWORD` | No | `""` | Optional Redis authentication password |
| `RABBITMQ_URL` | No | `amqp://guest:guest@127.0.0.1:5672/%2f` | RabbitMQ broker URL for publishing notification events |
| `SESSION_TTL_SEC` | No | `1800` | Access token session lifetime in seconds (default: 30 minutes) |
| `REFRESH_TOKEN_TTL_DAY` | No | `30` | Refresh token expiration period in days |
| `GOOGLE_CLIENT_ID` | No | `""` | Optional OAuth client audience for Google token validation |
| `GOOGLE_OAUTH_URL` | No | `https://oauth2.googleapis.com/tokeninfo` | Google OAuth token verification endpoint |
| `APP_ENV` | No | `development` | Environment mode (`development` attaches sanitized debug payloads) |
| `LOG_FORMAT` | No | `text` | Log format: `text` (human-readable) or `json` (structured) |
| `RUST_LOG` | No | `info,auth_service=debug` | Tracing log filter directive |

---

## API Reference

All endpoints accept and return `application/json`.
Full interactive schemas are available at **`/api/docs/`**.

### Endpoints Summary

| Method | Endpoint | Description | Auth Required |
|---|---|---|---|
| `GET` | `/health` | Health & liveness probe | No |
| `POST` | `/api/v1/auth/register/otp` | Request email registration OTP code | No |
| `POST` | `/api/v1/auth/register` | Complete registration with OTP code | No |
| `POST` | `/api/v1/auth/login` | Login with Email/Password or Google ID token | No |
| `POST` | `/api/v1/auth/verify` | Internal token verification endpoint (for Gateway / Services) | No |
| `POST` | `/api/v1/auth/refresh` | Rotate refresh token for a new token pair | No |
| `POST` | `/api/v1/auth/logout` | Terminate session, invalidate Redis cache, and delete refresh token | No |
| `POST` | `/api/v1/auth/forgot-password` | Request password reset OTP code | No |
| `POST` | `/api/v1/auth/reset-password` | Verify OTP code and set new password | No |
| `DELETE` | `/api/v1/auth/account` | Soft-delete user account (with 30-day re-registration cooldown) | Bearer Token |

---

### Sample Payloads

#### 1. Request Register OTP
`POST /api/v1/auth/register/otp`
```json
{
  "email": "user@example.com"
}
```

#### 2. Complete Registration
`POST /api/v1/auth/register`
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "otp": "123456"
}
```
*Response (`201 Created`)*:
```json
{
  "access_token": "79b329c3fa...",
  "refresh_token": "9a82cd11fe...",
  "token_type": "Bearer",
  "expires_in": 1800
}
```

#### 3. Login
`POST /api/v1/auth/login`
```json
// Email & Password:
{
  "type": "email",
  "email": "user@example.com",
  "password": "SecurePassword123!"
}

// Or Google ID Token:
{
  "type": "google",
  "id_token": "<google_id_token>"
}
```

#### 4. Token Verification (Internal RPC / HTTP)
`POST /api/v1/auth/verify`
```json
{
  "token": "<access_token>"
}
```
*Response (`200 OK`)*:
```json
{
  "user_id": "01a19a4d-e1d1-7822-ab00-0d07741a315b",
  "email": "user@example.com"
}
```

#### 5. Standard Error Envelope
On failure (`4xx` or `5xx`), responses return a consistent error envelope:
```json
{
  "success": false,
  "data": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid email or password",
    "details": null
  }
}
```
