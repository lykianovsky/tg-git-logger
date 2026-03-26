---
description: Create a new SeaORM database migration
argument-hint: "<description> — e.g. add_github_token_to_users"
allowed-tools: Read, Write, Edit, Glob, Bash
---

Create a new SeaORM migration for: `$ARGUMENTS`

## Steps

1. **Read existing migrations** in `migration/src/` to understand naming and structure. Look at the latest `mYYYYMMDD_HHMMSS_*.rs` file.

2. **Generate the migration file**:
   ```bash
   cd migration && cargo run -- generate MIGRATION_NAME
   ```
   Replace `MIGRATION_NAME` with a snake_case description based on `$ARGUMENTS`.

3. **Implement `up()` and `down()`** in the generated file using SeaORM's `SchemaManager`:
   - `create_table` / `drop_table`
   - `alter_table` (add/drop/modify columns)
   - `create_index` / `drop_index`
   - Use `ColumnDef` for column types (`string()`, `integer()`, `timestamp_with_time_zone()`, etc.)
   - Always implement a proper `down()` that reverses `up()` completely.

4. **Register** the new migration in `migration/src/lib.rs` inside the `migrations![]` macro.

5. **Show** the final migration code and remind to run `make generate-database-entity` after applying the migration to regenerate SeaORM entities.

Example column definition pattern:
```rust
.col(ColumnDef::new(User::GithubToken).string().not_null())
.col(ColumnDef::new(User::CreatedAt).timestamp_with_time_zone().not_null())
```
