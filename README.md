# BotGlue

Command center for AI agent-assisted development. Manage multiple AI coding agents across multiple projects from a single dashboard.

## Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (22+)
- [pnpm](https://pnpm.io/) (10+)
- [Podman](https://podman.io/) (4.0+) — used to run environment containers

## Data Model

```
Project (standard | incubator)
├── Ideas (draft → active → completed → archived)
│   └── Agents (assigned to work on the idea)
│       └── operates in → Environment
└── Environments (Podman containers)
```

- **Projects** hold ideas and environments. An *incubator* project is a scratchpad for early-stage ideas that can later graduate into their own projects.
- **Ideas** represent features or tasks to implement. Agents are assigned to ideas.
- **Agents** are AI coding sessions (Claude, Cursor, etc.) that run inside environments.
- **Environments** are containerized workspaces (Podman) with shell access and port forwarding.

## Project Structure

```
botglue/
├── daemon/          # Rust backend (Axum + SQLite)
│   └── src/
│       ├── main.rs      # HTTP server, route mounting
│       ├── db.rs         # SQLite connection + schema migration
│       ├── models/       # Data structs + DB queries (project, idea, agent, environment)
│       ├── routes/       # Axum HTTP handlers
│       └── podman.rs     # Podman container lifecycle
├── ui-common/       # Shared TypeScript library (types, components)
├── web/             # React SPA (Vite + Tailwind)
└── docs/            # Design docs and plans
```

## Setup

```bash
# Install TypeScript dependencies
cd ui-common && pnpm install && cd ..
cd web && pnpm install && cd ..

# Build the daemon
cd daemon && cargo build && cd ..
```

## Development

Run two terminals:

**Terminal 1 — Daemon** (port 3001):
```bash
cd daemon
cargo run
```

**Terminal 2 — Web dev server** (port 5173, proxies /api to daemon):
```bash
cd web
pnpm dev
```

Open http://localhost:5173 — you'll see the Dashboard page. It fetches projects, environments, and agents from the daemon API.

### Try it out

With both servers running:

1. Click **"+ New Project"** on the Dashboard to create a project (name, repo URL, branch, type)
   - Choose **Standard** for a regular project or **Incubator** for a scratchpad of ideas
2. Click a project card to open the **Project Detail** page
3. Click **"+ New Idea"** to create an idea (title, description)
4. Click an idea card to open the **Idea Detail** page — here you can:
   - Change the idea status (Start Working, Mark Complete, Archive)
   - **Assign an agent** to the idea (pick an environment, agent type, and task)
   - **Graduate** an idea from an incubator project into its own standalone project
5. Click **"+ New Environment"** to add an environment (branch name)
6. Click an environment card to open the **Environment Detail** page — here you can:
   - **Pause/Resume/Delete** the environment with the action buttons
   - Use the **Terminal** panel to run shell commands inside the container (e.g. `echo hello`, `ls /`)
   - Click **"+ New Agent"** to add an agent (select type, describe the task)
7. Click an agent row to open the **Agent Detail** page — here you can:
   - See the full task, blocker, linked idea, and environment
   - Change the agent status (Running, Blocked, Finished, Error)
8. Agents with `blocked`, `error`, or `finished` status appear in the **"Needs Attention"** queue on the Dashboard

You can also create test data via curl:

```bash
# Create an incubator project
curl -X POST http://localhost:3001/api/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"my-incubator","repo_url":"https://github.com/me/ideas","default_branch":"main","project_type":"incubator"}'

# Create a standard project
curl -X POST http://localhost:3001/api/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"my-app","repo_url":"https://github.com/me/my-app","default_branch":"main"}'

# Create an idea in a project
curl -X POST http://localhost:3001/api/ideas \
  -H 'Content-Type: application/json' \
  -d '{"project_id":"<PROJECT_ID>","title":"Add login page","description":"OAuth support"}'

# Create an environment
curl -X POST http://localhost:3001/api/environments \
  -H 'Content-Type: application/json' \
  -d '{"project_id":"<PROJECT_ID>","branch":"feature-x"}'

# Create an agent assigned to an idea
curl -X POST http://localhost:3001/api/agents \
  -H 'Content-Type: application/json' \
  -d '{"env_id":"<ENV_ID>","type":"claude","current_task":"Implement login page","idea_id":"<IDEA_ID>"}'

# Update agent status
curl -X PATCH http://localhost:3001/api/agents/<AGENT_ID> \
  -H 'Content-Type: application/json' \
  -d '{"status":"blocked","blocker":"waiting for API key"}'

# Graduate an idea from an incubator into a new project
curl -X POST http://localhost:3001/api/ideas/<IDEA_ID>/graduate \
  -H 'Content-Type: application/json' \
  -d '{"name":"login-service","repo_url":"https://github.com/me/login-service"}'

# Run a command inside an environment's container
curl -X POST http://localhost:3001/api/environments/<ENV_ID>/exec \
  -H 'Content-Type: application/json' \
  -d '{"command":"echo hello"}'
```

## Production Build

```bash
# Build the web SPA
cd web && pnpm build

# Run the daemon (serves web/dist/ as static files)
cd daemon && cargo run
```

Open http://localhost:3001

## Tests

```bash
# Run daemon unit tests (25 tests across models + podman port allocation)
cd daemon && cargo test

# Run Podman integration tests (requires podman running)
cd daemon && cargo test -- --ignored
```

## Typecheck

```bash
# Check shared types
cd ui-common && pnpm typecheck

# Check web app
cd web && pnpm tsc --noEmit
```

## API Endpoints

```
GET    /api/health
GET    /api/projects
POST   /api/projects                    # { name, repo_url, default_branch, project_type? }
GET    /api/projects/{id}
DELETE /api/projects/{id}
GET    /api/ideas?project_id=
POST   /api/ideas                       # { project_id, title, description? }
GET    /api/ideas/{id}
PUT    /api/ideas/{id}                  # { title, description }
DELETE /api/ideas/{id}
PUT    /api/ideas/{id}/status           # { status }
POST   /api/ideas/{id}/graduate         # { name, repo_url }
GET    /api/environments?project_id=
POST   /api/environments
GET    /api/environments/{id}
DELETE /api/environments/{id}
POST   /api/environments/{id}/pause
POST   /api/environments/{id}/resume
POST   /api/environments/{id}/exec
GET    /api/agents?env_id=&idea_id=
POST   /api/agents                      # { env_id, type, current_task, idea_id? }
GET    /api/agents/{id}
PATCH  /api/agents/{id}                 # { status, blocker? }
DELETE /api/agents/{id}
```

## Architecture

- **daemon** — Rust/Axum server with SQLite storage. Serves the REST API (`/api/*`) and the production SPA build. CRUD for projects, ideas, environments, and agents. Manages Podman containers for environments (create, pause/resume via stop/start, delete, exec). Auto-allocates host ports from a configurable range (default 10000-11000).
- **ui-common** — Shared React components and TypeScript types. Imported by `web` via `@botglue/common` path alias. Designed to be reusable in a future Tauri desktop app.
- **web** — React + Vite SPA. In development, Vite proxies `/api` requests to the daemon. In production, the daemon serves the built files directly.
