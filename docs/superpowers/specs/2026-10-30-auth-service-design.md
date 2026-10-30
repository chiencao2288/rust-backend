# Central Auth Service Design Specification

- **Date**: 2026-10-30
- **Status**: Approved
- **Scope**: Centralized Authentication & Session Management Service (Rust, Axum, tarpc, PostgreSQL, Redis)

---

## 1. Overview & Objectives

The goal is to build a centralized, high-performance, and minimal **Auth Service** (Identity & Session Service) to serve internal client applications (Web, Mobile) and downstream microservices/API Gateways.

### Core Architectural Decisions
- **Service Responsibility**: Pure authentication and session verification. Profile data (display name, avatar, address, etc.) is delegated to dedicated profile/downstream services.
- **Token Model**: Stateful session model.
  - `access_token`: Random cryptographic string (32 bytes), stored in **Redis** with short TTL (15–30 minutes). Gateway verifies tokens via RPC in $< 1\text{ ms}$ ($O(1)$ lookup).
  - `refresh_token`: Cryptographically hashed (SHA-256) and stored in **PostgreSQL**. Hard-deleted on rotation or logout.
- **Inter-service RPC**: Powered by `tarpc` (Rust-native RPC), enabling compile-time type-safe verification between API Gateway and Auth Service.
- **Monorepo Structure**: Modular micro-crates under `packages/` implemented as reusable, standalone libraries.

---

## 2. Monorepo Package Topology

```
services/
 └── auth-service/
      ├── migrations/
      └── src/
           ├── main.rs                 # Bootstrap and composition root
           ├── config.rs               # Strict fail-fast configuration loader
           ├── common/                 # Cross-cutting error mappings
           │    └── error.rs
           └── modules/                # NestJS-style feature modules
                ├── auth/              # Auth module: controller.rs, service.rs, dto.rs, mod.rs
                ├── users/             # Users module: entity.rs, repository.rs, mod.rs
                └── tokens/            # Tokens module: entity.rs, repository.rs, session_store.rs, mod.rs
packages/
 ├── rn-redis/                         # Standalone generic Redis library (Cluster-native, get/set/eval)
 ├── rn-postgres/                      # Standalone PostgreSQL client (Master / Multi-Slaves, Round-Robin)
 ├── rn-contracts/                     # HTTP DTOs and verify_token client helper
 └── rn-utils/                         # Hashing (Argon2id), cryptographic token & OTP generation
```

### Library Coding Standards for `packages/*`
1. Reusable library design: No hardcoded `std::env` reads; all configuration is injected via builders or constructors.
2. Strongly-typed errors using `thiserror`.
3. Standard trait derivations (`Clone`, `Debug`, `Send`, `Sync`).
4. English-only documentation: Concise doc-comments (`///`) on public functions only. Code must be self-documenting with no redundant inline commentary.

---

## 3. Database Schema (PostgreSQL)

### 3.1 Types & Enums
```sql
CREATE TYPE user_status AS ENUM ('ACTIVE', 'BANNED', 'SUSPENDED', 'DELETED');
CREATE TYPE identity_provider AS ENUM ('EMAIL', 'GOOGLE');
```

### 3.2 Tables

#### `users`
Represents the core human identity.
- `id`: `UUID PRIMARY KEY DEFAULT uuidv7()`
- `email`: `VARCHAR(255) UNIQUE NOT NULL` (Sanitized to `deleted_<uuid>@auth.local` on soft-delete)
- `status`: `user_status NOT NULL DEFAULT 'ACTIVE'`
- `deleted_at`: `TIMESTAMPTZ NULL`
- `created_at`: `TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at`: `TIMESTAMPTZ NOT NULL DEFAULT now()`


#### `identities`
Represents authentication methods linked to a user.
- `id`: `UUID PRIMARY KEY DEFAULT uuidv7()`
- `user_id`: `UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE`
- `provider`: `identity_provider NOT NULL`
- `identifier`: `VARCHAR(255) NOT NULL` (Email for `EMAIL`, Google `sub` for `GOOGLE`)
- `credential`: `VARCHAR(255) NULL` (Argon2id password hash; `NULL` for OAuth providers)
- `verified_at`: `TIMESTAMPTZ NOT NULL DEFAULT now()`
- `created_at`: `TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at`: `TIMESTAMPTZ NOT NULL DEFAULT now()`

*Constraints & Indexes*:
```sql
CONSTRAINT uq_identities_provider_identifier UNIQUE (provider, identifier);
CREATE INDEX idx_identities_user_id ON identities(user_id);
```

#### `refresh_tokens`
Stores active persistent sessions. Hard-deleted on rotation or logout.
- `id`: `UUID PRIMARY KEY DEFAULT uuidv7()`
- `user_id`: `UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE`
- `token_hash`: `VARCHAR(64) NOT NULL UNIQUE` (SHA-256 of plaintext refresh token)
- `expires_at`: `TIMESTAMPTZ NOT NULL`
- `created_at`: `TIMESTAMPTZ NOT NULL DEFAULT now()`

*Indexes*:
```sql
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
```

---

## 4. Cache & Ephemeral State (Redis)

### Key Hierarchy
- `auth:session:at_{access_token}`: JSON session payload `{"user_id": "...", "email": "..."}`, TTL 15–30 minutes.
- `auth:otp:register:{email}`: Pre-registration 6-digit OTP, TTL 5 minutes.
- `auth:otp:reset_pwd:{email}`: Password reset 6-digit OTP, TTL 5 minutes.

### Scaling & Clustering
- Cluster-first architecture via `fred`.
- Single or multi-node cluster configurations supported transparently via `RNRedisBuilder`.
- Atomic Lua scripts (`eval`) for check-and-delete OTP verification to prevent replay attacks.

---

## 5. Shared Contract (`packages/rn-contracts`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyTokenResponse {
    pub user_id: uuid::Uuid,
    pub email: String,
}

pub async fn verify_token(
    client: &reqwest::Client,
    auth_service_url: &str,
    token: &str,
) -> Result<VerifyTokenResponse, reqwest::Error>;
```

---

## 6. HTTP REST API (`services/auth-service` - Port 8080)

1. `POST /api/v1/auth/register/otp`: Request pre-registration OTP.
2. `POST /api/v1/auth/register`: Create account (`{ email, password, otp }`), auto-login.
3. `POST /api/v1/auth/login`: Unified login endpoint (`{ type: "email" | "google", ... }`).
4. `POST /api/v1/auth/refresh`: Token rotation (`{ refresh_token }`). Old token hard-deleted.
5. `POST /api/v1/auth/logout`: Hard-delete refresh token and revoke Redis session.
6. `POST /api/v1/auth/forgot-password`: Request password reset OTP.
7. `POST /api/v1/auth/reset-password`: Reset password, auto-link email identity if previously Google-only.
8. `DELETE /api/v1/auth/account`: Soft delete user, scrub PII email, hard-delete identities and tokens.
9. `POST /api/v1/auth/verify`: Internal token verification for Gateway & microservices ($O(1)$ Redis lookup $< 0.5\text{ ms}$). Load-balanced automatically per-request in k3s.
10. `GET /health`: Health and readiness probe for k3s.
11. `GET /swagger-ui/`: Interactive Swagger OpenAPI documentation and testing UI.
12. `GET /api-docs/openapi.json`: Raw OpenAPI 3.1 specification JSON.
---

## 7. Migration Strategy

- Migrations are pure, standard PostgreSQL scripts stored in `services/auth-service/migrations/`.
- Manual execution only: Executed by developers or deployment pipelines via CLI/Makefile or direct database IDE (DBeaver, TablePlus). Never auto-executed on application boot.
