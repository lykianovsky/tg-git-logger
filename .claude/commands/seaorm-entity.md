---
description: Help with SeaORM entities — understanding, modifying, or querying them
argument-hint: "<entity or table name> — e.g. user or user_has_roles"
allowed-tools: Read, Edit, Glob, Grep, Bash
---

Work with the SeaORM entity for: `$ARGUMENTS`

## Steps

1. **Locate the entity** in `src/infrastructure/database/mysql/`:
   ```bash
   find src/infrastructure/database -name "*.rs" | xargs grep -l "$ARGUMENTS"
   ```
   Read the entity file fully — column types, relations, active model.

2. **If the task involves querying**, show idiomatic SeaORM patterns:
   - `Entity::find().filter(Column::Field.eq(val)).one(db).await?`
   - `.find_with_related(RelatedEntity)` for eager loading
   - `SelectTwo` / `find_also_related` for joins
   - Paginated: `.paginate(db, page_size).fetch_page(page).await?`

3. **If the task involves inserting/updating**, use ActiveModel:
   ```rust
   let model = entity::ActiveModel {
       field: Set(value),
       ..Default::default()
   };
   model.insert(db).await?
   // or for update:
   model.update(db).await?
   ```

4. **If the entity needs a new column**, create a migration first (`/new-migration`), apply it, then run:
   ```bash
   make generate-database-entity
   ```
   This regenerates entities from the live MySQL schema.

5. **Repository pattern**: check `src/infrastructure/repositories/mysql/` for the repository that wraps this entity. Prefer adding query methods there rather than writing raw queries in application code.
