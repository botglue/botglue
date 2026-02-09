# Podman Container Lifecycle Integration

## Context

BotGlue's environment routes currently only update database records — creating an environment inserts a row with status "creating" but no actual container is created. This plan adds real Podman container management: when a user creates an environment, a Podman container is spun up; pause/resume/delete map to `podman stop/start/rm`; and a new exec endpoint lets users run commands inside containers.

**Scope:** Core lifecycle only — no templates, no repo cloning, no agent launching, no startup commands.

## Tasks

### Task 1: Create `podman.rs` — CLI wrapper module

**Files:**
- Create: `daemon/src/podman.rs`
- Modify: `daemon/src/main.rs` (add `mod podman;`, make it `pub`)

A standalone async module that shells out to `podman` CLI via `tokio::process::Command`. No database or HTTP dependency.

**Types:**
- `PodmanError` enum: `NotInstalled`, `CommandFailed { command, stderr, exit_code }`, `ParseError(String)`
- `PodmanConfig` struct: `podman_path: String`, `port_range_start: u16`, `port_range_end: u16` (defaults: `"podman"`, `10000`, `11000`)
- `ExecResult` struct: `output: String`, `exit_code: i32`

**Functions:**
- `check_podman(config) -> Result<String>` — runs `podman --version`
- `create_container(config, name, image, port_bindings) -> Result<String>` — runs `podman run -d --name {name} -p {h}:{c} ... {image} sleep infinity`, returns container ID
- `stop_container(config, container_id) -> Result<()>` — `podman stop`
- `start_container(config, container_id) -> Result<()>` — `podman start`
- `remove_container(config, container_id) -> Result<()>` — `podman rm -f`
- `exec_in_container(config, container_id, command) -> Result<ExecResult>` — `podman exec {id} sh -c {cmd}`

Container naming convention: `botglue-{first 8 chars of env_id}`.
Default image: `ubuntu:22.04` (constant).
`sleep infinity` keeps container alive as a sandbox for exec.

---

### Task 2: Port allocation logic + DB helpers

**Files:**
- Modify: `daemon/src/podman.rs` (add `allocate_ports` function)
- Modify: `daemon/src/models/environment.rs` (add `get_used_ports` and `update_environment_container` DB functions)

**Port allocator** (pure function in `podman.rs`):
- `allocate_ports(config, used_ports: &HashSet<u16>, requested: &[PortMapping]) -> Result<Vec<PortMapping>>`
- Auto-assigns `host_port` from range for any mapping where `host_port` is `None`
- Rejects conflicts (requested port already in use)
- Errors on range exhaustion

**`PortMapping` needs `Clone` derive** — add it in `environment.rs`.

**New DB functions in `environment.rs`:**
- `get_used_ports(db) -> Result<HashSet<u16>>` — queries all non-destroyed environments, extracts host_port from their ports JSON
- `update_environment_container(db, id, container_id, ports, status) -> Result<bool>` — updates container_id, ports JSON, status, and last_active

**Unit tests for port allocation:** 4 tests (auto-assign, skip used, explicit conflict, range exhaustion).
**Unit test for `update_environment_container`:** Create env, update with container info, verify fields changed.

---

### Task 3: Expand AppState to hold PodmanConfig

**Files:**
- Modify: `daemon/src/main.rs` — change `AppState` from `Arc<Db>` to `Arc<AppStateInner>` struct with `db: Db` and `podman: PodmanConfig`
- Modify: `daemon/src/routes/environments.rs` — update all handlers: `State(db)` → `State(state)`, `&db` → `&state.db`
- Modify: `daemon/src/routes/projects.rs` — same mechanical change
- Modify: `daemon/src/routes/agents.rs` — same mechanical change

This is a pure refactor — behavior unchanged. Every handler just adds `.db` to access the database.

```rust
// main.rs
pub struct AppStateInner {
    pub db: Db,
    pub podman: podman::PodmanConfig,
}
pub type AppState = Arc<AppStateInner>;
```

**Verify:** `cargo test` — all existing tests still pass. `cargo build` compiles.

---

### Task 4: Wire Podman into environment routes

**Files:**
- Modify: `daemon/src/routes/environments.rs`

This is the core task. Change return types of create/pause/resume/delete from `Result<T, StatusCode>` to `Result<T, impl IntoResponse>` to support JSON error responses.

**Error helper:**
```rust
#[derive(Serialize)]
struct ErrorResponse { error: String }

fn podman_err(e: PodmanError) -> (StatusCode, Json<ErrorResponse>) { ... }
fn internal_err(msg: String) -> (StatusCode, Json<ErrorResponse>) { ... }
```

**`create` handler changes:**
1. Insert DB record (status "creating") — same as before
2. Allocate ports via `allocate_ports(config, used_ports, requested_ports)`
3. Create container via `podman::create_container(config, name, image, port_bindings)`
4. Update DB with container_id, allocated ports, status "running" via `update_environment_container`
5. On Podman failure: set status to "destroyed" in DB, return error
6. Return the updated environment

**`pause` handler changes:**
1. Get environment, check status is "running" (return 409 Conflict otherwise)
2. `podman::stop_container` if container_id is non-empty
3. Update DB status to "paused"

**`resume` handler changes:**
1. Get environment, check status is "paused" (return 409 Conflict otherwise)
2. `podman::start_container` if container_id is non-empty
3. Update DB status to "running"

**`delete` handler changes:**
1. Get environment
2. `podman::remove_container` (best-effort, log warning on failure)
3. Delete from DB

**`list` and `get` handlers:** Unchanged (just the `state.db` refactor from Task 3).

**Verify:** `cargo build`. Manual curl test of lifecycle.

---

### Task 5: Add exec endpoint

**Files:**
- Modify: `daemon/src/routes/environments.rs` (add `exec` handler + request/response types)
- Modify: `daemon/src/main.rs` (mount route)

**Types:**
```rust
#[derive(Deserialize)]
struct ExecRequest { command: String }

#[derive(Serialize)]
struct ExecResponse { output: String, exit_code: i32 }
```

**Handler:**
1. Get environment, check status is "running" (409 otherwise)
2. Check container_id is non-empty (409 otherwise)
3. `podman::exec_in_container(config, container_id, command)`
4. Return `{ output, exit_code }`

**Route:** `POST /api/environments/{id}/exec`

**Also update `ui-common/api/client.ts`** to add the exec method:
```typescript
exec: (id: string, command: string) =>
  request<{ output: string; exit_code: number }>(`/api/environments/${id}/exec`, {
    method: "POST",
    body: JSON.stringify({ command }),
  }),
```

---

### Task 6: Integration tests + E2E verification

**Files:**
- Modify: `daemon/src/podman.rs` (add `#[ignore]` integration tests)

**Integration tests** (require live Podman, run with `cargo test -- --ignored`):
- `test_container_lifecycle`: create → exec → stop → start → exec → remove
- `test_exec_nonexistent_container`: verify error handling

**E2E verification script:**
1. `cargo test` — all unit tests pass (existing 11 + new port allocation + update_container)
2. `cargo test -- --ignored` — integration tests pass
3. Manual: start daemon, create project, create environment (verify container appears in `podman ps`), exec command, pause (verify container stopped), resume (verify container running), delete (verify container removed)
4. Clean up: kill daemon, `rm -f botglue.db*`, `podman rm -f` any stray containers

## Critical Files

| File | Action |
|------|--------|
| `daemon/src/podman.rs` | Create — CLI wrappers, port allocation, PodmanConfig |
| `daemon/src/main.rs` | Modify — add mod, expand AppState |
| `daemon/src/models/environment.rs` | Modify — add get_used_ports, update_environment_container, Clone on PortMapping |
| `daemon/src/routes/environments.rs` | Modify — wire Podman into all handlers, add exec |
| `daemon/src/routes/projects.rs` | Modify — mechanical state.db refactor |
| `daemon/src/routes/agents.rs` | Modify — mechanical state.db refactor |
| `ui-common/api/client.ts` | Modify — add exec method |
| `daemon/Cargo.toml` | No changes needed (tokio with "full" features already includes process) |

## Existing Code to Reuse

- `daemon/src/models/environment.rs`: `PortMapping` struct, `update_environment_status`, `get_environment`, `create_environment`, `delete_environment`
- `daemon/src/db.rs`: `Db` struct with `Mutex<Connection>` pattern
- `ui-common/api/client.ts`: `request<T>` helper function
