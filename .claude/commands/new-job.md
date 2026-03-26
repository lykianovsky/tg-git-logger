---
description: Scaffold a new async RabbitMQ job consumer following project patterns
argument-hint: "<JobName> — e.g. SendEmailNotify or SyncUserRoles"
allowed-tools: Read, Write, Edit, Glob, Grep
---

Create a new async job consumer named `$ARGUMENTS` following the project's existing patterns.

## Steps

1. **Read existing consumers** for the exact pattern:
   - `src/delivery/jobs/consumers/` — list what's there
   - Read one consumer (e.g. `send_social_notify`) fully: struct, `JobConsumer` trait impl, payload deserialization, command call.

2. **Create the job payload struct** (in `src/domain/` or near the domain it belongs to):
   - `#[derive(Serialize, Deserialize)]`
   - Only the data needed by the consumer

3. **Create `src/delivery/jobs/consumers/<snake_case_name>/mod.rs`**:
   - Struct holds Arc references to the ports/services it needs
   - Implement the `JobConsumer` (or equivalent) trait
   - Deserialize payload from `&[u8]` / JSON
   - Call the appropriate application command

4. **Register the consumer** in `src/bootstrap/registry/` (wherever job consumers are wired up — check existing consumers for the registration pattern).

5. **Create a producer helper** (if not already present) so other parts of the system can enqueue this job — check `src/infrastructure/processing/` for the pattern.

Queues used in this project: `events`, `jobs_critical`, `jobs_normal`, `jobs_background`. Pick the appropriate one based on priority.
