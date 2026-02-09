# BotGlue

Command center for AI agent-assisted development. Manage multiple AI coding agents across multiple projects from a single dashboard.

## Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (22+)
- [pnpm](https://pnpm.io/) (10+)

## Project Structure

```
botglue/
├── daemon/          # Rust backend (Axum + SQLite)
│   └── src/
│       ├── main.rs      # HTTP server, route mounting
│       ├── db.rs         # SQLite connection + schema migration
│       ├── models/       # Data structs + DB queries
│       └── routes/       # Axum HTTP handlers
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

Open http://localhost:5173

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
# Run daemon unit tests (11 tests across models)
cd daemon && cargo test
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

## Architecture

- **daemon** — Rust/Axum server with SQLite storage. Serves the REST API (`/api/*`) and the production SPA build. CRUD for projects, environments, and agents. Will manage Podman containers, monitor agents, and proxy LLM calls.
- **ui-common** — Shared React components and TypeScript types. Imported by `web` via `@botglue/common` path alias. Designed to be reusable in a future Tauri desktop app.
- **web** — React + Vite SPA. In development, Vite proxies `/api` requests to the daemon. In production, the daemon serves the built files directly.
