---
description: Scaffold a new Axum HTTP endpoint following the project's hexagonal architecture
argument-hint: "<METHOD> <path> — e.g. POST /webhook/gitlab"
allowed-tools: Read, Edit, Write, Glob, Grep, Bash
---

Add a new Axum HTTP endpoint for `$ARGUMENTS` following the project's existing patterns.

## Steps

1. **Read existing endpoints** to understand the pattern:
   - `src/delivery/http/axum/mod.rs` — router setup
   - An existing controller (e.g. `src/delivery/http/axum/webhook/github/`) — handler structure

2. **Create the controller module**:
   - New file in `src/delivery/http/axum/<domain>/<name>/mod.rs`
   - Extract input from request (path params, query, JSON body)
   - Call the appropriate application command/query
   - Return proper `axum::response::Response` or typed JSON

3. **Register the route** in `src/delivery/http/axum/mod.rs`

4. **If a new application command is needed**, scaffold it in `src/application/<domain>/` following the command pattern (struct + `execute` method + trait bounds for ports it needs).

5. **Update `ApplicationSharedDependency`** in `src/bootstrap/shared_dependency/` if new dependencies are injected.

Follow existing patterns exactly: error mapping, tracing spans, state extraction via `axum::extract::State`.
