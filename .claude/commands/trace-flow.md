---
description: Trace a feature or bug through all hexagonal architecture layers from entry point to database
argument-hint: "<feature or symptom> — e.g. 'PR merged webhook' or 'user not getting notification'"
allowed-tools: Read, Grep, Glob
---

Trace the full execution flow for: `$ARGUMENTS`

## What to do

Walk through every layer of the hexagonal architecture for the given feature/bug and produce a precise call chain:

1. **Delivery layer entry point** — which HTTP route, Telegram command, job consumer, or event listener handles this? Read `src/delivery/` to find it.

2. **Application command/query** — what use case is invoked? Read `src/application/` for the command, its inputs, outputs, and error types.

3. **Domain ports called** — what traits does the command depend on? List them with their method signatures from `src/domain/`.

4. **Infrastructure implementations** — which concrete structs implement those ports? (`src/infrastructure/`) Trace into the actual GitHub API call / DB query / Redis read / RabbitMQ publish.

5. **Database queries** — if SeaORM is involved, show the exact entity and active model used.

6. **Events and jobs** — if a domain event is published, show who listens and what job is enqueued next.

## Output format

```
[Delivery]  AxumWebhookGithubController::handle()
     ↓      src/delivery/http/axum/webhook/github/mod.rs:42
[App]       DispatchWebhookEvent::execute(payload)
     ↓      src/application/webhook/dispatch_event/mod.rs:18
[Domain]    WebhookRepository::save(&event)
     ↓      (port) src/domain/webhook/repository.rs:12
[Infra]     MysqlWebhookRepository::save()
            src/infrastructure/repositories/mysql/webhook/mod.rs:30
[Event]     WebhookEventDispatchedEvent → EventBus
[Job]       MoveTaskToTestJob enqueued → RabbitMQ jobs_normal
```

For a bug: after tracing the flow, highlight where the failure is most likely occurring and what to check.
