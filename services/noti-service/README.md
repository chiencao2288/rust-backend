# Central Notification Service (`noti-service`)

A multi-channel Notification Gateway and In-App Notification Center built in Rust using **Axum 0.8**, **PostgreSQL 18** (native `uuidv7()`), **Redis**, and **RabbitMQ**.

The service handles asynchronous multi-channel notification dispatch (Email via SMTP/Mock, SMS via Mock/Twilio) consumed from RabbitMQ, alongside high-performance in-app user notifications with real-time read tracking.

---

## Quick Start

### 1. Prerequisites
- [Rust](https://rustup.rs/) (1.80+)
- [Podman](https://podman.io/) or Docker

### 2. Start Infrastructure (PostgreSQL 18 + Redis + RabbitMQ 4)
```bash
make up
```
*Starts PostgreSQL 18 on `5432` (`noti_db`), Redis on `6379`, RabbitMQ AMQP on `5672`, and RabbitMQ Management UI on `15672`.*

- **RabbitMQ Dashboard:** 👉 **`http://localhost:15672`** (`guest` / `guest`)

### 3. Run Service
```bash
# Runs in local development mode with default configuration
make dev
```
The service will listen on `http://localhost:8081`.

👉 **Interactive Swagger UI:** **`http://localhost:8081/api/docs/`**

### 4. Stop Infrastructure
```bash
make down
```

### 5. Makefile Commands

| Command | Description |
|---|---|
| `make up` | Starts PostgreSQL 18, Redis, and RabbitMQ containers via Podman |
| `make down` | Stops and tears down infrastructure containers |
| `make dev` | Runs `noti-service` with local development environment variables |
| `make test` | Executes the complete test suite |
| `make check` | Runs linter checks (`cargo clippy --all-features`) |

---

## Environment Configuration

Configuration is loaded strictly from environment variables:

| Variable | Required | Default | Description |
|---|---|---|---|
| `PORT` | No | `8081` | HTTP listener port |
| `DATABASE_MASTER_URL` | **Yes** | - | Primary PostgreSQL connection URL (writes & transactions) |
| `DATABASE_SLAVE_URL_01` | No | `""` | Optional read replica URL for read queries |
| `REDIS_NODE_01` | **Yes** | - | Primary Redis cluster node URL |
| `REDIS_PASSWORD` | No | `""` | Optional Redis authentication password |
| `RABBITMQ_URL` | **Yes** | - | AMQP broker connection URL |
| `AUTH_SERVICE_URL` | No | `http://127.0.0.1:8080` | Auth service URL for validating Bearer tokens |
| `EMAIL_PROVIDER` | No | `smtp` | Email delivery provider: `smtp` or `mock` |
| `MAIL_HOST` | No | `""` | SMTP host (e.g. `smtp.gmail.com`) |
| `MAIL_PORT` | No | `587` | SMTP port |
| `MAIL_SECURE` | No | `false` | Enable TLS/SSL directly (STARTTLS used for 587) |
| `MAIL_USER` | No | `""` | SMTP username / email |
| `MAIL_PASS` | No | `""` | SMTP password / app password |
| `MAIL_FROM` | No | `chiencao19920808@gmail.com` | Sender email address |
| `MAIL_FROM_NAME` | No | `Podkeen` | Sender display name |
| `SMS_PROVIDER` | No | `mock` | SMS delivery provider: `mock` |
| `APP_ENV` | No | `development` | Environment mode (`development` / `production`) |
| `LOG_FORMAT` | No | `text` | Log format: `text` or `json` |

---

## API Reference (In-App Notification Center)

All endpoints accept and return `application/json`.
Full interactive schemas are available at **`/api/docs/`**.

### Endpoints Summary

| Method | Endpoint | Description | Auth Required |
|---|---|---|---|
| `GET` | `/health` | Health & liveness probe | No |
| `GET` | `/api/v1/notifications` | List user notifications (supports `limit`, `offset`, `unread_only`) | Bearer Token |
| `GET` | `/api/v1/notifications/unread-count` | Total count of unread notifications | Bearer Token |
| `PATCH` | `/api/v1/notifications/{id}/read` | Mark a single notification as read | Bearer Token |
| `PATCH` | `/api/v1/notifications/read-all` | Mark all unread notifications as read | Bearer Token |
| `DELETE` | `/api/v1/notifications/{id}` | Delete a specific notification | Bearer Token |

---

## Event-Driven Architecture (RabbitMQ)

Internal microservices publish events to RabbitMQ topic exchange `notifications.topic`.

| Channel | Queue | Routing Key | Payload Contract |
|---|---|---|---|
| **Email** | `notifications.email` | `notification.email` | `rn_contracts::noti::SendEmailPayload` |
| **SMS** | `notifications.sms` | `notification.sms` | `rn_contracts::noti::SendSmsPayload` |
| **In-App** | `notifications.in_app` | `notification.in_app` | `rn_contracts::noti::SendInAppPayload` |
