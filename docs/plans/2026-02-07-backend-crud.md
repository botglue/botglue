# Backend CRUD + SQLite Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add SQLite storage and CRUD REST endpoints for projects, environments, and agents to the BotGlue daemon.

**Architecture:** SQLite database managed via rusqlite with a `Db` struct holding a connection pool. Axum handlers share the database via Axum's `State` extractor. Each resource (project, environment, agent) gets its own Rust module with model structs, DB queries, and route handlers. Models mirror the TypeScript types in `ui-common/types/models.ts`.

**Tech Stack:** Rust, Axum 0.8, rusqlite (with bundled SQLite), serde, uuid, chrono, tokio

---

### Task 1: Add dependencies and create module structure

**Files:**
- Modify: `daemon/Cargo.toml`
- Create: `daemon/src/db.rs`
- Modify: `daemon/src/main.rs`

**Step 1: Add new dependencies to `daemon/Cargo.toml`**

Add these to `[dependencies]`:

```toml
rusqlite = { version = "0.34", features = ["bundled"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

**Step 2: Create `daemon/src/db.rs`**

Database initialization with schema migration:

```rust
use rusqlite::Connection;
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    fn migrate(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                repo_url TEXT NOT NULL,
                default_branch TEXT NOT NULL DEFAULT 'main',
                notification_prefs TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS environments (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id),
                branch TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'creating',
                container_id TEXT NOT NULL DEFAULT '',
                ports TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                last_active TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                env_id TEXT NOT NULL REFERENCES environments(id),
                type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'running',
                current_task TEXT NOT NULL DEFAULT '',
                blocker TEXT,
                started_at TEXT NOT NULL,
                last_activity TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }
}
```

**Step 3: Update `daemon/src/main.rs` to use the database**

Add `mod db;` and create the `Db` instance in `main()`, pass it as Axum state:

```rust
mod db;

use axum::{routing::get, Json, Router};
use db::Db;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub type AppState = Arc<Db>;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = Arc::new(Db::open("botglue.db").expect("Failed to open database"));

    let api_routes = Router::new()
        .route("/api/health", get(health))
        .with_state(db);

    let static_files = ServeDir::new("../web/dist").fallback(
        tower_http::services::ServeFile::new("../web/dist/index.html"),
    );

    let app = api_routes.fallback_service(static_files);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("BotGlue daemon listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

**Step 4: Verify it compiles**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo build`
Expected: Compiles without errors.

**Step 5: Verify it starts and creates the DB file**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo run &`
Then: `ls -la /Users/sergeyk/w/botglue/daemon/botglue.db`
Expected: File exists.
Then: `curl -s http://localhost:3001/api/health`
Expected: `{"status":"ok","version":"0.1.0"}`
Then: Kill the process and delete botglue.db.

**Step 6: Commit**

```bash
git add daemon/
git commit -m "feat: add SQLite database with schema migration"
```

---

### Task 2: Projects CRUD - models and DB layer

**Files:**
- Create: `daemon/src/models/mod.rs`
- Create: `daemon/src/models/project.rs`
- Modify: `daemon/src/main.rs` (add `mod models;`)

**Step 1: Create `daemon/src/models/mod.rs`**

```rust
pub mod project;
```

**Step 2: Create `daemon/src/models/project.rs`**

Rust structs matching the TypeScript types, plus DB query functions:

```rust
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationPrefs {
    pub blocked: bool,
    pub error: bool,
    pub finished: bool,
    pub progress: bool,
}

impl Default for NotificationPrefs {
    fn default() -> Self {
        Self {
            blocked: true,
            error: true,
            finished: true,
            progress: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repo_url: String,
    pub default_branch: String,
    pub notification_prefs: NotificationPrefs,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub repo_url: String,
    pub default_branch: Option<String>,
    pub notification_prefs: Option<NotificationPrefs>,
}

impl Project {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let prefs_json: String = row.get("notification_prefs")?;
        let notification_prefs: NotificationPrefs =
            serde_json::from_str(&prefs_json).unwrap_or_default();
        Ok(Project {
            id: row.get("id")?,
            name: row.get("name")?,
            repo_url: row.get("repo_url")?,
            default_branch: row.get("default_branch")?,
            notification_prefs,
            created_at: row.get("created_at")?,
        })
    }
}

use crate::db::Db;

pub fn list_projects(db: &Db) -> Result<Vec<Project>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, repo_url, default_branch, notification_prefs, created_at FROM projects ORDER BY created_at DESC",
    )?;
    let projects = stmt
        .query_map([], |row| Project::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(projects)
}

pub fn get_project(db: &Db, id: &str) -> Result<Option<Project>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, repo_url, default_branch, notification_prefs, created_at FROM projects WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| Project::from_row(row))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn create_project(db: &Db, input: CreateProject) -> Result<Project, rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let default_branch = input.default_branch.unwrap_or_else(|| "main".to_string());
    let prefs = input.notification_prefs.unwrap_or_default();
    let prefs_json = serde_json::to_string(&prefs).unwrap();

    let conn = db.conn();
    conn.execute(
        "INSERT INTO projects (id, name, repo_url, default_branch, notification_prefs, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, input.name, input.repo_url, default_branch, prefs_json, now],
    )?;

    Ok(Project {
        id,
        name: input.name,
        repo_url: input.repo_url,
        default_branch,
        notification_prefs: prefs,
        created_at: now,
    })
}

pub fn delete_project(db: &Db, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let rows = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}
```

**Step 3: Add `mod models;` to `daemon/src/main.rs`**

Add `mod models;` after `mod db;`.

**Step 4: Add unit tests to `daemon/src/models/project.rs`**

Append to the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Db {
        Db::open_in_memory().expect("Failed to create test database")
    }

    #[test]
    fn test_create_and_get_project() {
        let db = test_db();
        let project = create_project(
            &db,
            CreateProject {
                name: "test".to_string(),
                repo_url: "https://github.com/example/test".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap();

        assert_eq!(project.name, "test");
        assert_eq!(project.default_branch, "main");
        assert!(project.notification_prefs.blocked);

        let fetched = get_project(&db, &project.id).unwrap().unwrap();
        assert_eq!(fetched.id, project.id);
        assert_eq!(fetched.name, "test");
    }

    #[test]
    fn test_list_projects() {
        let db = test_db();
        create_project(
            &db,
            CreateProject {
                name: "a".to_string(),
                repo_url: "https://github.com/a".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap();
        create_project(
            &db,
            CreateProject {
                name: "b".to_string(),
                repo_url: "https://github.com/b".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap();

        let projects = list_projects(&db).unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_delete_project() {
        let db = test_db();
        let project = create_project(
            &db,
            CreateProject {
                name: "to-delete".to_string(),
                repo_url: "https://github.com/del".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap();

        assert!(delete_project(&db, &project.id).unwrap());
        assert!(get_project(&db, &project.id).unwrap().is_none());
        assert!(!delete_project(&db, &project.id).unwrap()); // already deleted
    }

    #[test]
    fn test_get_nonexistent_project() {
        let db = test_db();
        assert!(get_project(&db, "nonexistent").unwrap().is_none());
    }
}
```

**Step 5: Run tests**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo test -- models::project`
Expected: 4 tests pass.

**Step 6: Commit**

```bash
git add daemon/
git commit -m "feat: add Project model with CRUD database functions and tests"
```

---

### Task 3: Projects REST endpoints

**Files:**
- Create: `daemon/src/routes/mod.rs`
- Create: `daemon/src/routes/projects.rs`
- Modify: `daemon/src/main.rs` (mount project routes)

**Step 1: Create `daemon/src/routes/mod.rs`**

```rust
pub mod projects;
```

**Step 2: Create `daemon/src/routes/projects.rs`**

Axum handlers for project CRUD:

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::project::{self, CreateProject, Project};
use crate::AppState;

pub async fn list(State(db): State<AppState>) -> Result<Json<Vec<Project>>, StatusCode> {
    project::list_projects(&db).map(Json).map_err(|e| {
        tracing::error!("Failed to list projects: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn get(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    match project::get_project(&db, &id) {
        Ok(Some(p)) => Ok(Json(p)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(db): State<AppState>,
    Json(input): Json<CreateProject>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    project::create_project(&db, input)
        .map(|p| (StatusCode::CREATED, Json(p)))
        .map_err(|e| {
            tracing::error!("Failed to create project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn delete(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match project::delete_project(&db, &id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
```

**Step 3: Mount project routes in `daemon/src/main.rs`**

Add `mod routes;` and update the router:

```rust
mod db;
mod models;
mod routes;

// ... (existing imports plus:)
use axum::routing::{get, post, delete as delete_method};

// In main(), replace the api_routes with:
    let api_routes = Router::new()
        .route("/api/health", get(health))
        .route("/api/projects", get(routes::projects::list).post(routes::projects::create))
        .route("/api/projects/{id}", get(routes::projects::get).delete(routes::projects::delete))
        .with_state(db);
```

Note: Axum 0.8 uses `{id}` syntax for path params, not `:id`.

**Step 4: Verify it compiles**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo build`

**Step 5: Test the endpoints manually**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo run &`

Create a project:
```bash
curl -s -X POST http://localhost:3001/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"test-project","repo_url":"https://github.com/example/test"}'
```
Expected: 201 with JSON containing `id`, `name`, `repo_url`, `default_branch`, `notification_prefs`, `created_at`.

List projects:
```bash
curl -s http://localhost:3001/api/projects
```
Expected: Array with the created project.

Get project by id (use the id from create response):
```bash
curl -s http://localhost:3001/api/projects/<id>
```
Expected: The project object.

Delete project:
```bash
curl -s -X DELETE http://localhost:3001/api/projects/<id>
```
Expected: 204 No Content.

Then kill the process and delete botglue.db.

**Step 6: Commit**

```bash
git add daemon/
git commit -m "feat: add Projects REST endpoints (list, get, create, delete)"
```

---

### Task 4: Environments CRUD - models and DB layer

**Files:**
- Create: `daemon/src/models/environment.rs`
- Modify: `daemon/src/models/mod.rs`

**Step 1: Create `daemon/src/models/environment.rs`**

```rust
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMapping {
    pub name: String,
    pub container_port: u16,
    pub host_port: Option<u16>,
    pub protocol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    pub id: String,
    pub project_id: String,
    pub branch: String,
    pub status: String,
    pub container_id: String,
    pub ports: Vec<PortMapping>,
    pub created_at: String,
    pub last_active: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEnvironment {
    pub project_id: String,
    pub branch: String,
    pub ports: Option<Vec<PortMapping>>,
}

impl Environment {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let ports_json: String = row.get("ports")?;
        let ports: Vec<PortMapping> = serde_json::from_str(&ports_json).unwrap_or_default();
        Ok(Environment {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            branch: row.get("branch")?,
            status: row.get("status")?,
            container_id: row.get("container_id")?,
            ports,
            created_at: row.get("created_at")?,
            last_active: row.get("last_active")?,
        })
    }
}

use crate::db::Db;

pub fn list_environments(
    db: &Db,
    project_id: &str,
) -> Result<Vec<Environment>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, branch, status, container_id, ports, created_at, last_active FROM environments WHERE project_id = ?1 ORDER BY created_at DESC",
    )?;
    let envs = stmt
        .query_map(params![project_id], |row| Environment::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(envs)
}

pub fn get_environment(db: &Db, id: &str) -> Result<Option<Environment>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, branch, status, container_id, ports, created_at, last_active FROM environments WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| Environment::from_row(row))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn create_environment(
    db: &Db,
    input: CreateEnvironment,
) -> Result<Environment, rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let ports = input.ports.unwrap_or_default();
    let ports_json = serde_json::to_string(&ports).unwrap();

    let conn = db.conn();
    conn.execute(
        "INSERT INTO environments (id, project_id, branch, status, container_id, ports, created_at, last_active) VALUES (?1, ?2, ?3, 'creating', '', ?4, ?5, ?6)",
        params![id, input.project_id, input.branch, ports_json, now, now],
    )?;

    Ok(Environment {
        id,
        project_id: input.project_id,
        branch: input.branch,
        status: "creating".to_string(),
        container_id: String::new(),
        ports,
        created_at: now.clone(),
        last_active: now,
    })
}

pub fn update_environment_status(
    db: &Db,
    id: &str,
    status: &str,
) -> Result<bool, rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.conn();
    let rows = conn.execute(
        "UPDATE environments SET status = ?1, last_active = ?2 WHERE id = ?3",
        params![status, now, id],
    )?;
    Ok(rows > 0)
}

pub fn delete_environment(db: &Db, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let rows = conn.execute("DELETE FROM environments WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}
```

**Step 2: Update `daemon/src/models/mod.rs`**

```rust
pub mod project;
pub mod environment;
```

**Step 3: Add unit tests to `daemon/src/models/environment.rs`**

Append to the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{create_project, CreateProject};

    fn test_db() -> Db {
        Db::open_in_memory().expect("Failed to create test database")
    }

    fn create_test_project(db: &Db) -> String {
        create_project(
            db,
            CreateProject {
                name: "test".to_string(),
                repo_url: "https://github.com/test".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn test_create_and_get_environment() {
        let db = test_db();
        let project_id = create_test_project(&db);

        let env = create_environment(
            &db,
            CreateEnvironment {
                project_id: project_id.clone(),
                branch: "feat/test".to_string(),
                ports: Some(vec![PortMapping {
                    name: "ui".to_string(),
                    container_port: 3000,
                    host_port: Some(10001),
                    protocol: Some("http".to_string()),
                }]),
            },
        )
        .unwrap();

        assert_eq!(env.status, "creating");
        assert_eq!(env.branch, "feat/test");
        assert_eq!(env.ports.len(), 1);
        assert_eq!(env.ports[0].container_port, 3000);

        let fetched = get_environment(&db, &env.id).unwrap().unwrap();
        assert_eq!(fetched.id, env.id);
    }

    #[test]
    fn test_list_environments_by_project() {
        let db = test_db();
        let p1 = create_test_project(&db);
        let p2 = create_project(
            &db,
            CreateProject {
                name: "other".to_string(),
                repo_url: "https://github.com/other".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap()
        .id;

        create_environment(&db, CreateEnvironment { project_id: p1.clone(), branch: "a".to_string(), ports: None }).unwrap();
        create_environment(&db, CreateEnvironment { project_id: p1.clone(), branch: "b".to_string(), ports: None }).unwrap();
        create_environment(&db, CreateEnvironment { project_id: p2.clone(), branch: "c".to_string(), ports: None }).unwrap();

        assert_eq!(list_environments(&db, &p1).unwrap().len(), 2);
        assert_eq!(list_environments(&db, &p2).unwrap().len(), 1);
    }

    #[test]
    fn test_update_environment_status() {
        let db = test_db();
        let project_id = create_test_project(&db);
        let env = create_environment(&db, CreateEnvironment { project_id, branch: "main".to_string(), ports: None }).unwrap();

        assert!(update_environment_status(&db, &env.id, "running").unwrap());
        let fetched = get_environment(&db, &env.id).unwrap().unwrap();
        assert_eq!(fetched.status, "running");

        assert!(update_environment_status(&db, &env.id, "paused").unwrap());
        let fetched = get_environment(&db, &env.id).unwrap().unwrap();
        assert_eq!(fetched.status, "paused");
    }

    #[test]
    fn test_delete_environment() {
        let db = test_db();
        let project_id = create_test_project(&db);
        let env = create_environment(&db, CreateEnvironment { project_id, branch: "main".to_string(), ports: None }).unwrap();

        assert!(delete_environment(&db, &env.id).unwrap());
        assert!(get_environment(&db, &env.id).unwrap().is_none());
    }
}
```

**Step 4: Run tests**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo test -- models::environment`
Expected: 4 tests pass.

**Step 5: Commit**

```bash
git add daemon/
git commit -m "feat: add Environment model with CRUD database functions and tests"
```

---

### Task 5: Environments REST endpoints

**Files:**
- Create: `daemon/src/routes/environments.rs`
- Modify: `daemon/src/routes/mod.rs`
- Modify: `daemon/src/main.rs` (mount environment routes)

**Step 1: Create `daemon/src/routes/environments.rs`**

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::environment::{self, CreateEnvironment, Environment};
use crate::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    pub project_id: String,
}

pub async fn list(
    State(db): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Environment>>, StatusCode> {
    environment::list_environments(&db, &query.project_id)
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list environments: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Environment>, StatusCode> {
    match environment::get_environment(&db, &id) {
        Ok(Some(e)) => Ok(Json(e)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(db): State<AppState>,
    Json(input): Json<CreateEnvironment>,
) -> Result<(StatusCode, Json<Environment>), StatusCode> {
    environment::create_environment(&db, input)
        .map(|e| (StatusCode::CREATED, Json(e)))
        .map_err(|e| {
            tracing::error!("Failed to create environment: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[derive(Deserialize)]
pub struct StatusUpdate {
    pub status: String,
}

pub async fn pause(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match environment::update_environment_status(&db, &id, "paused") {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to pause environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn resume(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match environment::update_environment_status(&db, &id, "running") {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to resume environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match environment::delete_environment(&db, &id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
```

**Step 2: Update `daemon/src/routes/mod.rs`**

```rust
pub mod projects;
pub mod environments;
```

**Step 3: Mount environment routes in `daemon/src/main.rs`**

Add to the router:

```rust
    let api_routes = Router::new()
        .route("/api/health", get(health))
        .route("/api/projects", get(routes::projects::list).post(routes::projects::create))
        .route("/api/projects/{id}", get(routes::projects::get).delete(routes::projects::delete))
        .route("/api/environments", get(routes::environments::list).post(routes::environments::create))
        .route("/api/environments/{id}", get(routes::environments::get).delete(routes::environments::delete))
        .route("/api/environments/{id}/pause", post(routes::environments::pause))
        .route("/api/environments/{id}/resume", post(routes::environments::resume))
        .with_state(db);
```

Add `post` to the `use axum::routing::{...}` import.

**Step 4: Verify it compiles**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo build`

**Step 5: Test endpoints manually**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo run &`

Create a project first, then create an environment:
```bash
# Create project
PROJECT=$(curl -s -X POST http://localhost:3001/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"test","repo_url":"https://github.com/example/test"}')
PROJECT_ID=$(echo $PROJECT | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

# Create environment
curl -s -X POST http://localhost:3001/api/environments \
  -H "Content-Type: application/json" \
  -d "{\"project_id\":\"$PROJECT_ID\",\"branch\":\"feat/test\"}"

# List environments
curl -s "http://localhost:3001/api/environments?project_id=$PROJECT_ID"
```

Expected: Environment created with status "creating", listed correctly.

Kill the process and delete botglue.db.

**Step 6: Commit**

```bash
git add daemon/
git commit -m "feat: add Environments REST endpoints (list, get, create, pause, resume, delete)"
```

---

### Task 6: Agents CRUD - models and DB layer

**Files:**
- Create: `daemon/src/models/agent.rs`
- Modify: `daemon/src/models/mod.rs`

**Step 1: Create `daemon/src/models/agent.rs`**

```rust
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: String,
    pub env_id: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub status: String,
    pub current_task: String,
    pub blocker: Option<String>,
    pub started_at: String,
    pub last_activity: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgent {
    pub env_id: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub current_task: String,
}

impl Agent {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Agent {
            id: row.get("id")?,
            env_id: row.get("env_id")?,
            agent_type: row.get("type")?,
            status: row.get("status")?,
            current_task: row.get("current_task")?,
            blocker: row.get("blocker")?,
            started_at: row.get("started_at")?,
            last_activity: row.get("last_activity")?,
        })
    }
}

use crate::db::Db;

pub fn list_agents(db: &Db, env_id: Option<&str>) -> Result<Vec<Agent>, rusqlite::Error> {
    let conn = db.conn();
    match env_id {
        Some(eid) => {
            let mut stmt = conn.prepare(
                "SELECT id, env_id, type, status, current_task, blocker, started_at, last_activity FROM agents WHERE env_id = ?1 ORDER BY started_at DESC",
            )?;
            let agents = stmt
                .query_map(params![eid], |row| Agent::from_row(row))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(agents)
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT id, env_id, type, status, current_task, blocker, started_at, last_activity FROM agents ORDER BY started_at DESC",
            )?;
            let agents = stmt
                .query_map([], |row| Agent::from_row(row))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(agents)
        }
    }
}

pub fn get_agent(db: &Db, id: &str) -> Result<Option<Agent>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, env_id, type, status, current_task, blocker, started_at, last_activity FROM agents WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| Agent::from_row(row))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn create_agent(db: &Db, input: CreateAgent) -> Result<Agent, rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn();
    conn.execute(
        "INSERT INTO agents (id, env_id, type, status, current_task, blocker, started_at, last_activity) VALUES (?1, ?2, ?3, 'running', ?4, NULL, ?5, ?6)",
        params![id, input.env_id, input.agent_type, input.current_task, now, now],
    )?;

    Ok(Agent {
        id,
        env_id: input.env_id,
        agent_type: input.agent_type,
        status: "running".to_string(),
        current_task: input.current_task,
        blocker: None,
        started_at: now.clone(),
        last_activity: now,
    })
}

pub fn update_agent_status(
    db: &Db,
    id: &str,
    status: &str,
    blocker: Option<&str>,
) -> Result<bool, rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.conn();
    let rows = conn.execute(
        "UPDATE agents SET status = ?1, blocker = ?2, last_activity = ?3 WHERE id = ?4",
        params![status, blocker, now, id],
    )?;
    Ok(rows > 0)
}
```

**Step 2: Update `daemon/src/models/mod.rs`**

```rust
pub mod project;
pub mod environment;
pub mod agent;
```

**Step 3: Add unit tests to `daemon/src/models/agent.rs`**

Append to the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::environment::{create_environment, CreateEnvironment};
    use crate::models::project::{create_project, CreateProject};

    fn test_db() -> Db {
        Db::open_in_memory().expect("Failed to create test database")
    }

    fn create_test_env(db: &Db) -> String {
        let project_id = create_project(
            db,
            CreateProject {
                name: "test".to_string(),
                repo_url: "https://github.com/test".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap()
        .id;
        create_environment(
            db,
            CreateEnvironment {
                project_id,
                branch: "main".to_string(),
                ports: None,
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn test_create_and_get_agent() {
        let db = test_db();
        let env_id = create_test_env(&db);

        let agent = create_agent(
            &db,
            CreateAgent {
                env_id: env_id.clone(),
                agent_type: "claude".to_string(),
                current_task: "implement feature".to_string(),
            },
        )
        .unwrap();

        assert_eq!(agent.status, "running");
        assert_eq!(agent.agent_type, "claude");
        assert!(agent.blocker.is_none());

        let fetched = get_agent(&db, &agent.id).unwrap().unwrap();
        assert_eq!(fetched.id, agent.id);
        assert_eq!(fetched.current_task, "implement feature");
    }

    #[test]
    fn test_list_agents_with_filter() {
        let db = test_db();
        let env1 = create_test_env(&db);
        // Create second env under same project
        let project_id = create_project(
            &db,
            CreateProject {
                name: "p2".to_string(),
                repo_url: "https://github.com/p2".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap()
        .id;
        let env2 = create_environment(
            &db,
            CreateEnvironment { project_id, branch: "main".to_string(), ports: None },
        )
        .unwrap()
        .id;

        create_agent(&db, CreateAgent { env_id: env1.clone(), agent_type: "claude".to_string(), current_task: "a".to_string() }).unwrap();
        create_agent(&db, CreateAgent { env_id: env2.clone(), agent_type: "cursor".to_string(), current_task: "b".to_string() }).unwrap();

        assert_eq!(list_agents(&db, Some(&env1)).unwrap().len(), 1);
        assert_eq!(list_agents(&db, Some(&env2)).unwrap().len(), 1);
        assert_eq!(list_agents(&db, None).unwrap().len(), 2);
    }

    #[test]
    fn test_update_agent_status() {
        let db = test_db();
        let env_id = create_test_env(&db);
        let agent = create_agent(&db, CreateAgent { env_id, agent_type: "claude".to_string(), current_task: "task".to_string() }).unwrap();

        assert!(update_agent_status(&db, &agent.id, "blocked", Some("which auth library?")).unwrap());
        let fetched = get_agent(&db, &agent.id).unwrap().unwrap();
        assert_eq!(fetched.status, "blocked");
        assert_eq!(fetched.blocker.as_deref(), Some("which auth library?"));

        assert!(update_agent_status(&db, &agent.id, "running", None).unwrap());
        let fetched = get_agent(&db, &agent.id).unwrap().unwrap();
        assert_eq!(fetched.status, "running");
        assert!(fetched.blocker.is_none());
    }
}
```

**Step 4: Run all tests**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo test`
Expected: All 11 tests pass (4 project + 4 environment + 3 agent).

**Step 5: Commit**

```bash
git add daemon/
git commit -m "feat: add Agent model with CRUD database functions and tests"
```

---

### Task 7: Agents REST endpoints

**Files:**
- Create: `daemon/src/routes/agents.rs`
- Modify: `daemon/src/routes/mod.rs`
- Modify: `daemon/src/main.rs` (mount agent routes)

**Step 1: Create `daemon/src/routes/agents.rs`**

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::agent::{self, Agent, CreateAgent};
use crate::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    pub env_id: Option<String>,
}

pub async fn list(
    State(db): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    agent::list_agents(&db, query.env_id.as_deref())
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Agent>, StatusCode> {
    match agent::get_agent(&db, &id) {
        Ok(Some(a)) => Ok(Json(a)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(db): State<AppState>,
    Json(input): Json<CreateAgent>,
) -> Result<(StatusCode, Json<Agent>), StatusCode> {
    agent::create_agent(&db, input)
        .map(|a| (StatusCode::CREATED, Json(a)))
        .map_err(|e| {
            tracing::error!("Failed to create agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
```

**Step 2: Update `daemon/src/routes/mod.rs`**

```rust
pub mod projects;
pub mod environments;
pub mod agents;
```

**Step 3: Mount agent routes in `daemon/src/main.rs`**

Add to the router:

```rust
        .route("/api/agents", get(routes::agents::list).post(routes::agents::create))
        .route("/api/agents/{id}", get(routes::agents::get))
```

**Step 4: Verify it compiles**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo build`

**Step 5: Full integration test**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo run &`

```bash
# Create project
PROJECT=$(curl -s -X POST http://localhost:3001/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"test","repo_url":"https://github.com/example/test"}')
PROJECT_ID=$(echo $PROJECT | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

# Create environment
ENV=$(curl -s -X POST http://localhost:3001/api/environments \
  -H "Content-Type: application/json" \
  -d "{\"project_id\":\"$PROJECT_ID\",\"branch\":\"main\"}")
ENV_ID=$(echo $ENV | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

# Create agent
curl -s -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d "{\"env_id\":\"$ENV_ID\",\"type\":\"claude\",\"current_task\":\"implement feature X\"}"

# List all agents
curl -s http://localhost:3001/api/agents

# List agents for environment
curl -s "http://localhost:3001/api/agents?env_id=$ENV_ID"

# Health check still works
curl -s http://localhost:3001/api/health
```

Expected: All endpoints return correct data. Foreign key constraint ensures env_id references a real environment.

Kill the process and delete botglue.db.

**Step 6: Commit**

```bash
git add daemon/
git commit -m "feat: add Agents REST endpoints (list, get, create)"
```

---

### Task 8: Add CORS and .gitignore for database file

**Files:**
- Modify: `daemon/src/main.rs` (add CORS middleware)
- Modify: `daemon/.gitignore` (ignore database files)

**Step 1: Add CORS to `daemon/src/main.rs`**

The web dev server runs on a different port (5173) than the daemon (3001). The Vite proxy handles `/api` requests, but CORS is still good practice for direct access and future use.

Add after the router is built:

```rust
use tower_http::cors::{CorsLayer, Any};

// After building api_routes, before fallback_service:
    let api_routes = Router::new()
        // ... routes ...
        .with_state(db)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));
```

**Step 2: Update `daemon/.gitignore`**

Read the current file, then add:

```
/target/
*.db
*.db-wal
*.db-shm
```

**Step 3: Verify it compiles and CORS headers appear**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo build && cargo run &`
Then: `curl -s -I -X OPTIONS http://localhost:3001/api/health`
Expected: Response includes `access-control-allow-origin` header.
Kill the process.

**Step 4: Commit**

```bash
git add daemon/
git commit -m "feat: add CORS middleware and gitignore database files"
```

---

## Summary

After all 8 tasks, the daemon has:

```
daemon/src/
├── main.rs          # Axum server with all routes mounted, CORS, DB state
├── db.rs            # SQLite connection pool, schema migration
├── models/
│   ├── mod.rs
│   ├── project.rs   # Project struct + CRUD queries
│   ├── environment.rs # Environment struct + CRUD queries
│   └── agent.rs     # Agent struct + CRUD queries
└── routes/
    ├── mod.rs
    ├── projects.rs    # GET/POST /api/projects, GET/DELETE /api/projects/{id}
    ├── environments.rs # GET/POST /api/environments, GET/DELETE /api/environments/{id}, POST pause/resume
    └── agents.rs      # GET/POST /api/agents, GET /api/agents/{id}
```

**API endpoints implemented:**
```
GET    /api/health
GET    /api/projects
POST   /api/projects
GET    /api/projects/{id}
DELETE /api/projects/{id}
GET    /api/environments?project_id=
POST   /api/environments
GET    /api/environments/{id}
DELETE /api/environments/{id}
POST   /api/environments/{id}/pause
POST   /api/environments/{id}/resume
GET    /api/agents?env_id=
POST   /api/agents
GET    /api/agents/{id}
```

**Not yet built (future plans):**
- POST /api/environments/{id}/fork
- POST /api/environments/{id}/exec
- GET /api/audit
- WebSocket /api/ws
- LLM Gateway /llm/v1/*
