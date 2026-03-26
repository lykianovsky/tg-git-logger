---
description: Review Rust code for correctness, idiomatic style, and architecture compliance
argument-hint: "<file or module path>"
allowed-tools: Read, Grep, Glob, Bash
---

Review the Rust code at: `$ARGUMENTS`

Read the file(s) and review for:

## Architecture compliance
- Does the code respect hexagonal layers? (domain ← application ← delivery / infrastructure)
- Are domain types used for business logic, or is infrastructure leaking into domain?
- Does each struct/command have a single responsibility?

## Rust idioms
- Prefer `?` over `unwrap()`/`expect()` in fallible paths
- Use `thiserror` for error types, not `Box<dyn Error>` or `String`
- Async functions: unnecessary `.await` on non-async code, missing `Send` bounds on spawned tasks
- Cloning: is `Arc<T>` used correctly vs unnecessary deep clones?
- Lifetime issues: borrowed data escaping async contexts
- Iterator chains vs explicit loops (prefer iterators when clearer)

## SeaORM specific
- N+1 queries: loading relations in a loop instead of using `find_with_related`
- Missing `.await?` on async DB calls
- Raw `unwrap()` on `Option` from DB results

## Security
- User input passed directly to queries (use parameterized `filter(Col.eq(val))`)
- Tokens/secrets logged via `tracing` macros (should be redacted)
- HMAC signature verification bypassed or weakened

## Output format
For each issue found:
- **File:line** — issue description
- Severity: `critical` / `warning` / `suggestion`
- Fixed code snippet (if non-trivial)

End with a short summary: overall quality, top 3 things to fix.
