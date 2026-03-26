# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**tg-bot-logger** (marketed as "GitEye Bot") is a Telegram bot + backend service that bridges GitHub webhooks, a Kaiten
task tracker, and Telegram notifications. When a PR is merged on GitHub, it extracts the task ID, moves the card on
Kaiten's board, and notifies Telegram users.

## Build & Run Commands

```bash
# Development
make dev-build          # cargo build
make dev-run            # cargo run

# Production
make prod-build         # cargo build --release
make prod-run           # git pull && build && run binary

# Docker (development)
make docker-dev-build
make docker-dev-up
make docker-dev-down
make docker-dev-logs

# Docker (production)
make docker-prod-build
make docker-prod-up
make docker-prod-down
make docker-prod-logs
make docker-prod-restart

# Database entities (SeaORM codegen from live MySQL)
make generate-database-entity

# Standard cargo
cargo check
cargo clippy
cargo fmt
cargo test
```

## Architecture

Hexagonal architecture with four strict layers — no layer may import from a layer above it:

```
Delivery  →  Application  →  Domain  ←  Infrastructure
```

- **`src/domain/`** — Entities, value objects, repository traits (ports), domain events. No external dependencies.
- **`src/application/`** — Use cases (Commands/Queries). Calls domain ports only. Never touches infrastructure directly.
- **`src/delivery/`** — Entry points: HTTP (Axum), Telegram bot (teloxide dialogues), event listeners, job consumers,
  cron scheduler.
- **`src/infrastructure/`** — Adapters: MySQL (SeaORM), Redis, RabbitMQ (lapin), GitHub OAuth, GitHub API (GraphQL +
  REST), Kaiten API.
- **`src/bootstrap/`** — Wires everything together at startup. `ApplicationSharedDependency` is the DI container.
- **`src/config/`** — Typed config structs loaded from `.env` via `dotenv`.
- **`migration/`** — Separate workspace crate for SeaORM migrations; applied automatically on startup.

### Typical Event Flow

```
GitHub webhook POST /webhook/github
  → AxumWebhookGithubController (HMAC-SHA256 verification)
  → DispatchWebhookEvent command
  → WebhookEventDispatchedEvent on the event bus
  → Event listener
  → MoveTaskToTest command → Kaiten API
  → SendSocialNotify job published to RabbitMQ
  → Job consumer → CompositionNotificationService → Telegram
```

### Delivery Layer Details

| Component       | Location                   | Role                                                                |
|-----------------|----------------------------|---------------------------------------------------------------------|
| HTTP server     | `delivery/http/axum/`      | `/oauth/github`, `/webhook/github`, `/ping`                         |
| Telegram bot    | `delivery/bot/telegram/`   | `/start`, `/register`, `/report` commands + dialogue state machines |
| Event listeners | `delivery/events/`         | Handle domain events (GitHub webhooks, user registration)           |
| Job consumers   | `delivery/jobs/consumers/` | Process async jobs from RabbitMQ                                    |
| Scheduler       | `delivery/scheduler/`      | Cron-based periodic tasks                                           |

All five run concurrently as separate Tokio tasks spawned in `bootstrap/mod.rs`.

### Key Abstractions

- **Ports** are traits in `domain/*/` (e.g., `TaskTrackerClient`, `VersionControlClient`, `OAuthClient`). Infrastructure
  provides the implementations.
- **Commands** in `application/*/` each implement a single use-case and receive only the traits they need via dependency
  injection.
- **Domain events** are published to an in-process event bus (`infrastructure/processing/`); heavier work is offloaded
  to RabbitMQ jobs.
- **Encryption**: AES-GCM (`utils/security/crypto/`) wraps sensitive data before Redis/DB storage. Key comes from
  `REVERSABLE_CIPHER_SECRET_KEY`.

## Configuration

Copy `.env.example` to `.env` and fill in values. Key variables:

| Group    | Variables                                                                                                                       |
|----------|---------------------------------------------------------------------------------------------------------------------------------|
| App      | `APPLICATION_PORT`, `DEBUG`                                                                                                     |
| Telegram | `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID`, `TELEGRAM_ADMIN_USER_ID`                                                              |
| MySQL    | `MYSQL_URL`, `MYSQL_USERNAME`, `MYSQL_PASSWORD`, `MYSQL_DATABASE_NAME`, `MYSQL_PORT`                                            |
| RabbitMQ | `RABBITMQ_URL`, `RABBITMQ_USER`, `RABBITMQ_PASSWORD`, `RABBITMQ_PORT`                                                           |
| Redis    | `REDIS_URL`, `REDIS_SECRET_KEY`                                                                                                 |
| GitHub   | `GITHUB_WEBHOOK_SECRET`, `GITHUB_OAUTH_CLIENT_ID`, `GITHUB_OAUTH_CLIENT_SECRET`, `GITHUB_REPOSITORY_OWNER`, `GITHUB_REPOSITORY` |
| Kaiten   | `KAITEN_BASE`, `KAITEN_API_TOKEN`, `TASK_TRACKER_SPACE_ID`, `TASK_TRACKER_QA_COLUMN_ID`, `TASK_TRACKER_EXTRACT_PATTERN_REGEXP`  |
| Security | `REVERSABLE_CIPHER_SECRET_KEY`                                                                                                  |

## Code Style

`rustfmt.toml` sets `max_width = 100`. Run `cargo fmt` before committing.

Errors use `thiserror`. Logging uses `tracing` macros (`info!`, `error!`, etc.). All async code runs on Tokio.

## Rust Development Rules

Rules derived from the actual patterns in this codebase. Follow them exactly when writing or modifying code.

### Errors

- Every fallible domain operation gets its own `thiserror` enum in the same module or a sibling `error.rs`.
- Compose errors from lower layers with `#[from]`; never leak infrastructure types into domain errors.
- Use `?` to propagate — no `.unwrap()` / `.expect()` outside of bootstrap/config initialization.
- `Box<dyn Error>` is only acceptable in `ApplicationSharedDependency::new()` bootstrap wiring.

```rust
// GOOD — domain error, composed from infrastructure
#[derive(Debug, Error)]
pub enum FindUserByIdError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("User not found")]
    NotFound,
}

// BAD — leaking sea_orm type into domain
#[error("DB: {0}")]
DbError(#[from] sea_orm::DbErr),  // ← only in application layer, never domain
```

### Async & Traits

- All trait definitions that need to be object-safe use `#[async_trait]` + explicit `Send + Sync` bounds.
- Never use AFIT (`async fn` in traits) — the codebase uses `async_trait` throughout.
- Spawned Tokio tasks must capture only `Send + 'static` data; use `Arc` to share.

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<User, FindUserByIdError>;
}
```

### Dependency Injection

- All dependencies are `Arc<dyn Trait>` — no generic struct parameters.
- `ApplicationSharedDependency` is the only place where concrete types are instantiated.
- Commands receive only the ports they actually use, nothing more.

```rust
// GOOD
pub struct MoveTaskToTestExecutor {
    task_tracker_client: Arc<dyn TaskTrackerClient>,
    test_column_id: u64,
}

// BAD — unnecessary generics on struct
pub struct MoveTaskToTestExecutor<T: TaskTrackerClient> { ... }
```

### Architecture layer rules

- `domain/` has zero external crate imports (no sea_orm, no reqwest, no lapin).
- `application/` imports only `domain/` ports and shared domain types.
- `infrastructure/` implements domain ports; never imported by `application/`.
- `delivery/` calls application commands; does not touch infrastructure directly.
- Cross-layer imports cause compiler errors by design — if you hit one, reconsider the design.

### Logging

Use `tracing` structured fields with `%` (Display) or `?` (Debug) formatters. Match log level to intent:

| Level | When |
|-------|------|
| `error!` | Operation failed, needs attention |
| `warn!` | Unexpected but recoverable state |
| `info!` | Significant lifecycle milestone (startup, connection established) |
| `debug!` | Request/response data, state transitions |
| `trace!` | Fine-grained detail (raw bytes, intermediate values) |

```rust
// GOOD — structured fields
tracing::debug!(state_id = %key, "Retrieving OAuth state from cache");
tracing::error!(error = %e, state_id = %key, "Failed to retrieve state from cache");

// BAD — unstructured string formatting
tracing::debug!("Retrieving state {}", key);
```

Never log token values, secrets, or raw OAuth codes — even at `trace!` level.

### Module structure

- Use `pub mod` declarations; avoid `pub use` re-exports unless building a clean public API boundary.
- One concept per file. Large modules split into submodules, not into one giant file.
- Error types live next to the code that produces them, not in a global `errors.rs`.

### SeaORM

- Schema changes go through a migration first, then `make generate-database-entity` regenerates entities.
- Never hand-edit generated entity files in `infrastructure/database/mysql/` — they will be overwritten.
- Pass `&DatabaseTransaction` into repository methods when atomicity is needed; commit/rollback in the command layer.
- Avoid N+1: use `find_with_related` or `find_also_related` instead of loading relations in a loop.
- Use `ActiveModel` with `Set(value)` for inserts/updates; never construct raw SQL.

```rust
// GOOD — transaction passed from command layer
async fn create(&self, txn: &DatabaseTransaction, user: &User) -> Result<User, CreateUserError>;

// GOOD — parameterized filter, no injection risk
Entity::find().filter(Column::Email.eq(&email)).one(db).await?

// BAD — loading related data in a loop (N+1)
for user in users {
    let roles = RoleEntity::find_by_user(user.id).await?;
}
```

### Domain Events & Jobs

- Events implement `DomainEvent` (with `const EVENT_NAME`) + `Serialize`/`Deserialize`.
- Publish events via `MessageBrokerPublisher::publish()`, not via `EventBus::dispatch()` directly from delivery layer.
- `EventBus` is for in-process listeners; RabbitMQ is for durable async jobs.
- Job structs implement `MessageBrokerMessage` and declare their priority queue:
  - `Critical` — user-facing real-time actions (send notification)
  - `Normal` — standard async work (move task, sync data)
  - `Background` — reports, non-urgent processing
- Ignore publish errors on the happy path with `.ok()` — listener failures must not abort the main flow.

```rust
// Publish and ignore error deliberately
self.publisher.publish(&SomeJob { ... }).await.ok();
```

### Executors

Every `execute()` method must return `Result<Response, Error>`. Even when the result carries no data, define a unit response struct. Errors always propagate to the caller — never swallow them inside the executor.

```rust
// GOOD
pub struct DeleteFooResponse;

impl DeleteFooExecutor {
    pub async fn execute(&self, cmd: &DeleteFooCommand) -> Result<DeleteFooResponse, DeleteFooExecutorError> {
        self.repo.delete(cmd.id).await?;
        Ok(DeleteFooResponse)
    }
}

// BAD — swallowing errors hides failures from the caller
pub async fn execute(&self) {
    if let Err(e) = self.run().await {
        tracing::error!(error = %e, "...");
    }
}
```

The caller (scheduler, delivery handler, consumer) is responsible for deciding what to do with an error — log it, return a user-facing message, or retry.

### Structs & Constructors

- Use `new()` constructors for all types with dependencies.
- Use the fluent builder pattern (methods returning `self`) only for value-object assembly like `MessageBuilder`.
- No `Default` derives unless all fields have meaningful zero values.

### Config

- New config values go in `src/config/application/mod.rs` as typed fields (not raw strings passed around).
- Parse numeric/bool config with `.parse().unwrap()` — panic on startup is acceptable; panic at runtime is not.
- Group related env vars into a sub-config struct (`ApplicationXxxConfig`).

## Поведение при неясных задачах

- **Если задача размыта — спроси, не угадывай.** Лучше один уточняющий вопрос, чем неверная реализация.
- **Если не знаешь — скажи прямо.** Не придумывай ответ. Допустимые формулировки: "Не знаю, нужно проверить", "Не уверен — вот моё понимание, поправь если ошибаюсь".
- **Перед нетривиальной реализацией** — кратко сформулируй своё понимание задачи и спроси подтверждение.
- **Вопросы задавай списком** — все сразу, не по одному.

## Commit convention

- feat(domain): — новая фича в домене
- feat(api): — новый endpoint
- fix: — исправление бага
- refactor: — рефакторинг без изменения поведения
- test: — тесты
- chore: — зависимости, конфиги