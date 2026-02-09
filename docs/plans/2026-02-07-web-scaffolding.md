# Web Scaffolding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up the `daemon` (Rust/Axum), `ui-common` (shared TypeScript library), and `web` (React SPA) subdirectories, with build tooling, dev server, and the full stack compiling and running.

**Architecture:** Three subdirectories (`daemon/`, `ui-common/`, `web/`) with no npm workspaces. `web/` imports from `ui-common/` via TypeScript path aliases. Daemon serves `web/dist/` as static files. Vite dev server proxies API calls to daemon during development.

**Tech Stack:** Rust (Axum, Tokio, SQLite), React 19, Vite, TypeScript, Tailwind CSS v4, pnpm

---

### Task 1: Initialize `daemon` with Axum

**Files:**
- Create: `daemon/Cargo.toml`
- Create: `daemon/src/main.rs`

**Step 1: Initialize Rust project**

Run: `cd /Users/sergeyk/w/botglue && cargo init daemon`

**Step 2: Add dependencies to `daemon/Cargo.toml`**

```toml
[package]
name = "botglue-daemon"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["fs", "cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Step 3: Write `daemon/src/main.rs`**

Minimal server: health endpoint + static file serving.

```rust
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber;

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
    tracing_subscriber::init();

    let api_routes = Router::new().route("/api/health", get(health));

    // Serve web/dist/ as static files, fallback to index.html for SPA routing
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

**Step 5: Verify health endpoint**

Run: `cd /Users/sergeyk/w/botglue/daemon && cargo run &`
Then: `curl -s http://localhost:3001/api/health`
Expected: `{"status":"ok","version":"0.1.0"}`
Then: Kill the process.

**Step 6: Commit**

```bash
git add daemon/
git commit -m "feat: scaffold Rust daemon with Axum, health endpoint, static file serving"
```

---

### Task 2: Initialize `ui-common` package

**Files:**
- Create: `ui-common/package.json`
- Create: `ui-common/tsconfig.json`
- Create: `ui-common/types/index.ts`
- Create: `ui-common/types/models.ts`

**Step 1: Create `ui-common/package.json`**

```json
{
  "name": "botglue-ui-common",
  "version": "0.0.1",
  "private": true,
  "type": "module",
  "main": "index.ts",
  "types": "index.ts",
  "scripts": {
    "typecheck": "tsc --noEmit"
  },
  "peerDependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "typescript": "^5.7.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0"
  }
}
```

**Step 2: Create `ui-common/tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "skipLibCheck": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "dist",
    "rootDir": ".",
    "composite": true
  },
  "include": ["**/*.ts", "**/*.tsx"],
  "exclude": ["dist", "node_modules"]
}
```

**Step 3: Create `ui-common/types/models.ts`**

The core data model types from the design doc:

```typescript
export interface Project {
  id: string;
  name: string;
  repo_url: string;
  default_branch: string;
  notification_prefs: NotificationPrefs;
  created_at: string;
}

export interface NotificationPrefs {
  blocked: boolean;
  error: boolean;
  finished: boolean;
  progress: boolean;
}

export interface PortMapping {
  name: string;
  container_port: number;
  host_port?: number;
  protocol?: "http" | "ws";
}

export interface Environment {
  id: string;
  project_id: string;
  branch: string;
  status: "creating" | "running" | "paused" | "destroyed";
  container_id: string;
  ports: PortMapping[];
  created_at: string;
  last_active: string;
}

export interface Agent {
  id: string;
  env_id: string;
  type: "claude" | "cursor" | "opencode" | "custom";
  status: "running" | "blocked" | "finished" | "error";
  current_task: string;
  blocker: string | null;
  started_at: string;
  last_activity: string;
}

export interface AuditEntry {
  id: string;
  env_id: string;
  agent_id: string;
  operation: string;
  command: string;
  output: string;
  exit_code: number;
  timestamp: string;
}

export interface LLMUsageEntry {
  env_id: string;
  agent_id: string;
  provider: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  timestamp: string;
}

export type AgentEvent =
  | { type: "agent.blocked"; agent_id: string; blocker: string }
  | { type: "agent.finished"; agent_id: string; summary: string }
  | { type: "agent.error"; agent_id: string; error: string }
  | { type: "agent.progress"; agent_id: string; output_tail: string[] }
  | { type: "env.status"; env_id: string; status: Environment["status"] };
```

**Step 4: Create `ui-common/types/index.ts`**

```typescript
export * from "./models";
```

**Step 5: Create `ui-common/index.ts`**

```typescript
export * from "./types";
```

**Step 6: Install dependencies**

Run: `cd /Users/sergeyk/w/botglue/ui-common && pnpm install`

**Step 7: Typecheck**

Run: `cd /Users/sergeyk/w/botglue/ui-common && pnpm typecheck`
Expected: No errors.

**Step 8: Commit**

```bash
git add ui-common/
git commit -m "feat: scaffold ui-common with shared types"
```

---

### Task 3: Initialize `web` SPA with Vite + React

**Files:**
- Create: `web/package.json`
- Create: `web/tsconfig.json`
- Create: `web/tsconfig.node.json`
- Create: `web/vite.config.ts`
- Create: `web/index.html`
- Create: `web/src/main.tsx`
- Create: `web/src/App.tsx`
- Create: `web/src/vite-env.d.ts`

**Step 1: Scaffold with Vite**

Run: `cd /Users/sergeyk/w/botglue && pnpm create vite web --template react-ts`

This creates the `web/` directory with React + TypeScript template.

**Step 2: Install dependencies**

Run: `cd /Users/sergeyk/w/botglue/web && pnpm install`

**Step 3: Verify it runs**

Run: `cd /Users/sergeyk/w/botglue/web && pnpm dev &`
Then: `curl -s http://localhost:5173 | head -20`
Expected: HTML with `<div id="root">` and script tag.
Then: Kill the dev server.

**Step 4: Remove boilerplate**

Delete the default Vite/React boilerplate files we don't need:
- `web/src/App.css`
- `web/src/index.css`
- `web/src/assets/react.svg`
- `web/public/vite.svg`

**Step 5: Commit**

```bash
git add web/
git commit -m "feat: scaffold web SPA with Vite + React + TypeScript"
```

---

### Task 4: Wire `ui-common` imports into `web`

**Files:**
- Modify: `web/tsconfig.json` (add path alias)
- Modify: `web/vite.config.ts` (add alias)
- Modify: `web/src/App.tsx` (import a type from ui-common to prove it works)

**Step 1: Add path alias to `web/tsconfig.json`**

Add to `compilerOptions`:

```json
{
  "compilerOptions": {
    "paths": {
      "@botglue/common/*": ["../ui-common/*"],
      "@botglue/common": ["../ui-common"]
    }
  }
}
```

Note: Preserve existing compilerOptions from the Vite template, just add `paths` and set `baseUrl` to `"."`.

**Step 2: Add alias to `web/vite.config.ts`**

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@botglue/common": path.resolve(__dirname, "../ui-common"),
    },
  },
});
```

**Step 3: Update `web/src/App.tsx` to import from ui-common**

Replace contents with:

```tsx
import type { Project } from "@botglue/common/types";

const mockProject: Project = {
  id: "1",
  name: "botglue",
  repo_url: "https://github.com/example/botglue",
  default_branch: "main",
  notification_prefs: {
    blocked: true,
    error: true,
    finished: true,
    progress: false,
  },
  created_at: new Date().toISOString(),
};

function App() {
  return (
    <div>
      <h1>BotGlue</h1>
      <p>Project: {mockProject.name}</p>
      <p>Status: scaffolding complete</p>
    </div>
  );
}

export default App;
```

**Step 4: Verify the import works**

Run: `cd /Users/sergeyk/w/botglue/web && pnpm dev &`
Then: `curl -s http://localhost:5173 | head -20`
Expected: Page loads without errors. Check browser or terminal for no import errors.
Then: Kill the dev server.

**Step 5: Commit**

```bash
git add web/ ui-common/
git commit -m "feat: wire ui-common imports into web via path alias"
```

---

### Task 5: Add Tailwind CSS v4

**Files:**
- Modify: `web/package.json` (add tailwind)
- Create: `web/src/index.css`
- Modify: `web/src/main.tsx` (import css)
- Modify: `web/src/App.tsx` (use tailwind classes to verify)

**Step 1: Install Tailwind v4 and Vite plugin**

Run: `cd /Users/sergeyk/w/botglue/web && pnpm add tailwindcss @tailwindcss/vite`

**Step 2: Add Tailwind plugin to `web/vite.config.ts`**

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@botglue/common": path.resolve(__dirname, "../ui-common"),
    },
  },
});
```

**Step 3: Create `web/src/index.css`**

```css
@import "tailwindcss";
```

**Step 4: Update `web/src/main.tsx` to import CSS**

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
```

**Step 5: Update `web/src/App.tsx` with Tailwind classes**

Use the landing page's dark theme colors to verify Tailwind is working:

```tsx
import type { Project } from "@botglue/common/types";

const mockProject: Project = {
  id: "1",
  name: "botglue",
  repo_url: "https://github.com/example/botglue",
  default_branch: "main",
  notification_prefs: {
    blocked: true,
    error: true,
    finished: true,
    progress: false,
  },
  created_at: new Date().toISOString(),
};

function App() {
  return (
    <div className="min-h-screen bg-[#0a0a0f] text-[#f0f0f5] flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-semibold mb-4">BotGlue</h1>
        <p className="text-[#a0a0b0]">Project: {mockProject.name}</p>
        <p className="text-[#6b6b7b] mt-2">Scaffolding complete</p>
      </div>
    </div>
  );
}

export default App;
```

**Step 6: Verify Tailwind works**

Run: `cd /Users/sergeyk/w/botglue/web && pnpm dev &`
Check in browser: dark background, centered white text, muted secondary text.
Then: Kill the dev server.

**Step 7: Commit**

```bash
git add web/
git commit -m "feat: add Tailwind CSS v4 to web"
```

---

### Task 6: Create first shared component in `ui-common`

**Files:**
- Create: `ui-common/components/AgentStatusBadge.tsx`
- Create: `ui-common/components/index.ts`
- Modify: `ui-common/index.ts` (re-export components)
- Modify: `web/src/App.tsx` (render the component to prove cross-project components work)

**Step 1: Create `ui-common/components/AgentStatusBadge.tsx`**

```tsx
import type { Agent } from "../types";

const statusColors: Record<Agent["status"], string> = {
  running: "bg-green-500/20 text-green-400 border-green-500/30",
  blocked: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
  finished: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  error: "bg-red-500/20 text-red-400 border-red-500/30",
};

interface AgentStatusBadgeProps {
  status: Agent["status"];
}

export function AgentStatusBadge({ status }: AgentStatusBadgeProps) {
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${statusColors[status]}`}
    >
      {status}
    </span>
  );
}
```

**Step 2: Create `ui-common/components/index.ts`**

```typescript
export { AgentStatusBadge } from "./AgentStatusBadge";
```

**Step 3: Update `ui-common/index.ts`**

```typescript
export * from "./types";
export * from "./components";
```

**Step 4: Install react as dev dependency in ui-common**

Run: `cd /Users/sergeyk/w/botglue/ui-common && pnpm install`

This resolves the peer deps so TypeScript can check the component.

**Step 5: Typecheck ui-common**

Run: `cd /Users/sergeyk/w/botglue/ui-common && pnpm typecheck`
Expected: No errors.

**Step 6: Update `web/src/App.tsx` to render the badge**

```tsx
import type { Project } from "@botglue/common/types";
import { AgentStatusBadge } from "@botglue/common/components";

const mockProject: Project = {
  id: "1",
  name: "botglue",
  repo_url: "https://github.com/example/botglue",
  default_branch: "main",
  notification_prefs: {
    blocked: true,
    error: true,
    finished: true,
    progress: false,
  },
  created_at: new Date().toISOString(),
};

const statuses = ["running", "blocked", "finished", "error"] as const;

function App() {
  return (
    <div className="min-h-screen bg-[#0a0a0f] text-[#f0f0f5] flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-semibold mb-4">BotGlue</h1>
        <p className="text-[#a0a0b0]">Project: {mockProject.name}</p>
        <div className="flex gap-2 mt-4 justify-center">
          {statuses.map((s) => (
            <AgentStatusBadge key={s} status={s} />
          ))}
        </div>
        <p className="text-[#6b6b7b] mt-4">Scaffolding complete</p>
      </div>
    </div>
  );
}

export default App;
```

**Step 7: Verify shared component renders**

Run: `cd /Users/sergeyk/w/botglue/web && pnpm dev &`
Check in browser: four colored badges (green running, yellow blocked, blue finished, red error).
Then: Kill the dev server.

**Step 8: Commit**

```bash
git add ui-common/ web/
git commit -m "feat: add AgentStatusBadge shared component, verify cross-project import"
```

---

### Task 7: Add `.gitignore` and root config

**Files:**
- Create: `.gitignore`
- Create: `.nvmrc`

**Step 1: Create `.gitignore`**

```
node_modules/
dist/
.vite/
*.local
.DS_Store
target/
```

**Step 2: Create `.nvmrc`**

Run: `node -v` to check current version, then write that version.

```
22
```

(Use whatever major version is current.)

**Step 3: Verify nothing is accidentally tracked**

Run: `git status`
Expected: No `node_modules/` or `dist/` directories showing.

**Step 4: Commit**

```bash
git add .gitignore .nvmrc
git commit -m "chore: add .gitignore and .nvmrc"
```

---

### Task 8: Configure Tailwind to scan `ui-common` for classes

**Files:**
- Modify: `web/src/index.css` (add source for ui-common)

**Step 1: Update `web/src/index.css`**

Tailwind v4 uses `@source` to tell it where to scan for classes. We need it to scan `ui-common/` too, since shared components use Tailwind classes:

```css
@import "tailwindcss";
@source "../../ui-common";
```

**Step 2: Verify ui-common component styles work**

Run: `cd /Users/sergeyk/w/botglue/web && pnpm dev &`
Check in browser: AgentStatusBadge colors render correctly (not unstyled).
Then: Kill the dev server.

**Step 3: Commit**

```bash
git add web/src/index.css
git commit -m "feat: configure Tailwind to scan ui-common for classes"
```

---

### Task 9: Add Vite proxy to daemon and end-to-end verification

**Files:**
- Modify: `web/vite.config.ts` (add proxy)
- Modify: `web/src/App.tsx` (fetch from health endpoint)

**Step 1: Add API proxy to `web/vite.config.ts`**

Add `server.proxy` so the Vite dev server forwards `/api` requests to the Rust daemon:

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@botglue/common": path.resolve(__dirname, "../ui-common"),
    },
  },
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3001",
        changeOrigin: true,
      },
    },
  },
});
```

**Step 2: Update `web/src/App.tsx` to call the daemon**

Add a fetch to `/api/health` to prove the full stack works:

```tsx
import { useState, useEffect } from "react";
import type { Project } from "@botglue/common/types";
import { AgentStatusBadge } from "@botglue/common/components";

const mockProject: Project = {
  id: "1",
  name: "botglue",
  repo_url: "https://github.com/example/botglue",
  default_branch: "main",
  notification_prefs: {
    blocked: true,
    error: true,
    finished: true,
    progress: false,
  },
  created_at: new Date().toISOString(),
};

const statuses = ["running", "blocked", "finished", "error"] as const;

function App() {
  const [daemonStatus, setDaemonStatus] = useState<string>("checking...");

  useEffect(() => {
    fetch("/api/health")
      .then((r) => r.json())
      .then((data) => setDaemonStatus(`${data.status} (v${data.version})`))
      .catch(() => setDaemonStatus("not running"));
  }, []);

  return (
    <div className="min-h-screen bg-[#0a0a0f] text-[#f0f0f5] flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-semibold mb-4">BotGlue</h1>
        <p className="text-[#a0a0b0]">Project: {mockProject.name}</p>
        <div className="flex gap-2 mt-4 justify-center">
          {statuses.map((s) => (
            <AgentStatusBadge key={s} status={s} />
          ))}
        </div>
        <p className="text-[#6b6b7b] mt-4">Daemon: {daemonStatus}</p>
      </div>
    </div>
  );
}

export default App;
```

**Step 3: End-to-end verification**

Terminal 1: `cd /Users/sergeyk/w/botglue/daemon && cargo run`
Terminal 2: `cd /Users/sergeyk/w/botglue/web && pnpm dev`

Open browser at `http://localhost:5173`. Expected:
- Dark background with "BotGlue" heading
- Four colored status badges
- "Daemon: ok (v0.1.0)" text (proves proxy → daemon works)

**Step 4: Verify production build**

Run: `cd /Users/sergeyk/w/botglue/web && pnpm build`
Then with daemon running: open `http://localhost:3001`
Expected: Same page served by the daemon from `web/dist/`.

**Step 5: Commit**

```bash
git add web/
git commit -m "feat: add Vite proxy to daemon, end-to-end verification"
```

---

## Summary

After all 9 tasks, the repo looks like:

```
botglue/
├── .gitignore
├── .nvmrc
├── LICENSE
├── daemon/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── docs/
│   ├── landing/
│   └── plans/
├── ui-common/
│   ├── package.json
│   ├── tsconfig.json
│   ├── index.ts
│   ├── types/
│   │   ├── index.ts
│   │   └── models.ts
│   └── components/
│       ├── index.ts
│       └── AgentStatusBadge.tsx
└── web/
    ├── package.json
    ├── tsconfig.json
    ├── vite.config.ts
    ├── index.html
    └── src/
        ├── main.tsx
        ├── index.css
        ├── vite-env.d.ts
        └── App.tsx
```

**What's proven:**
- Rust daemon compiles and serves health endpoint + static files
- TypeScript compiles across both TS projects
- `web` imports types and components from `ui-common` via path alias
- Tailwind picks up classes from both `web` and `ui-common`
- Vite dev server proxies API calls to daemon
- Production build served directly by daemon
- One shared component (`AgentStatusBadge`) renders correctly
- Full stack end-to-end: browser → Vite → daemon → response

**Next plan:** API client in `ui-common/api/`, Dashboard page with mock data, routing, more daemon endpoints.
