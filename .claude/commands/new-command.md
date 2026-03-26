---
description: Scaffold a new application layer Command or Query (use case) following hexagonal architecture
argument-hint: "<CommandName> in <domain> — e.g. AssignUserRole in user"
allowed-tools: Read, Write, Edit, Glob, Grep
---

Create a new application command/query: `$ARGUMENTS`

## Steps

1. **Read an existing command** in `src/application/` that is closest in scope to the new one. Understand:
   - How port traits are received (generic params vs `Arc<dyn Trait>`)
   - Error type pattern (`thiserror` enum)
   - Return type convention

2. **Create `src/application/<domain>/<snake_case_name>/mod.rs`**:
   ```rust
   pub struct YourCommand<Dep1, Dep2> {
       dep1: Arc<Dep1>,
       dep2: Arc<Dep2>,
   }

   impl<Dep1, Dep2> YourCommand<Dep1, Dep2>
   where
       Dep1: SomeDomainPort + Send + Sync,
       Dep2: AnotherDomainPort + Send + Sync,
   {
       pub fn new(dep1: Arc<Dep1>, dep2: Arc<Dep2>) -> Self { ... }
       pub async fn execute(&self, input: YourInput) -> Result<YourOutput, YourError> { ... }
   }
   ```

3. **Define the error type** using `thiserror` in the same file or a sibling `error.rs`.

4. **Only depend on domain ports** (traits in `src/domain/`). Never import from `infrastructure/` directly.

5. **Wire the command** in `src/bootstrap/executors.rs` or wherever the application layer is assembled — check existing commands to find where they are instantiated.

6. **Export** the module in `src/application/<domain>/mod.rs`.
