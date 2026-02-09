# API Client + Routing + Dashboard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a typed API client in ui-common, add React Router to the web SPA, and implement the Dashboard page with live data from the daemon.

**Architecture:** The API client lives in `ui-common/api/` so it's reusable by a future desktop app. React Router provides client-side routing with a shared layout shell. The Dashboard page fetches projects, environments, and agents from the daemon REST API and renders them grouped by project.

**Tech Stack:** TypeScript, React 19, React Router 7, Tailwind CSS v4, Vite 7

---

### Task 1: API Client — core fetch wrapper

**Files:**
- Create: `ui-common/api/client.ts`

**Context:**
- The daemon REST API is at `/api/...` (proxied by Vite in dev, same-origin in production)
- All endpoints return JSON. Errors return HTTP status codes (404, 500) with no body
- Existing types are in `ui-common/types/models.ts`

**Step 1: Create the API client module**

```typescript
// ui-common/api/client.ts

import type {
  Project,
  Environment,
  Agent,
} from "../types";

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = "ApiError";
  }
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(path, {
    headers: { "Content-Type": "application/json", ...options?.headers },
    ...options,
  });
  if (!res.ok) {
    throw new ApiError(res.status, `${res.status} ${res.statusText}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

export const api = {
  projects: {
    list: () => request<Project[]>("/api/projects"),
    get: (id: string) => request<Project>(`/api/projects/${id}`),
    create: (data: { name: string; repo_url: string; default_branch: string }) =>
      request<Project>("/api/projects", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    delete: (id: string) =>
      request<void>(`/api/projects/${id}`, { method: "DELETE" }),
  },

  environments: {
    list: (projectId: string) =>
      request<Environment[]>(`/api/environments?project_id=${projectId}`),
    get: (id: string) => request<Environment>(`/api/environments/${id}`),
    create: (data: {
      project_id: string;
      branch: string;
      container_id: string;
      ports: string;
    }) =>
      request<Environment>("/api/environments", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    pause: (id: string) =>
      request<void>(`/api/environments/${id}/pause`, { method: "POST" }),
    resume: (id: string) =>
      request<void>(`/api/environments/${id}/resume`, { method: "POST" }),
    delete: (id: string) =>
      request<void>(`/api/environments/${id}`, { method: "DELETE" }),
  },

  agents: {
    list: (envId?: string) =>
      request<Agent[]>(
        envId ? `/api/agents?env_id=${envId}` : "/api/agents"
      ),
    get: (id: string) => request<Agent>(`/api/agents/${id}`),
    create: (data: {
      env_id: string;
      agent_type: string;
      current_task: string;
    }) =>
      request<Agent>("/api/agents", {
        method: "POST",
        body: JSON.stringify(data),
      }),
  },
};

export { ApiError };
```

**Step 2: Add barrel export**

Create `ui-common/api/index.ts`:
```typescript
export { api, ApiError } from "./client";
```

**Step 3: Commit**

```bash
git add ui-common/api/client.ts ui-common/api/index.ts
git commit -m "feat: add typed API client in ui-common"
```

---

### Task 2: Install React Router

**Files:**
- Modify: `web/package.json` (pnpm install)

**Step 1: Install react-router**

```bash
cd web && pnpm add react-router
```

React Router 7 is the current version. It exports everything from `react-router` (no separate `react-router-dom` package needed).

**Step 2: Commit**

```bash
git add web/package.json web/pnpm-lock.yaml
git commit -m "feat: add react-router dependency"
```

---

### Task 3: App shell layout + routing setup

**Files:**
- Create: `web/src/layouts/AppLayout.tsx`
- Create: `web/src/pages/DashboardPage.tsx` (placeholder)
- Modify: `web/src/App.tsx` (replace with router)
- Modify: `web/src/main.tsx` (wrap with BrowserRouter)

**Context:**
- Current `App.tsx` has mock data and hardcoded demo content — replace entirely
- Vite proxy for `/api` already configured in `web/vite.config.ts`
- Path alias `@botglue/common` → `../ui-common` already set up in `web/vite.config.ts` and `web/tsconfig.app.json`
- `web/tsconfig.app.json` has `"verbatimModuleSyntax": true` — use `import type` for type-only imports

**Step 1: Create the app layout shell**

```typescript
// web/src/layouts/AppLayout.tsx

import { Outlet, Link, useLocation } from "react-router";

const navItems = [
  { path: "/", label: "Dashboard" },
  { path: "/settings", label: "Settings" },
];

export function AppLayout() {
  const location = useLocation();

  return (
    <div className="min-h-screen bg-[#0a0a0f] text-[#f0f0f5]">
      <header className="border-b border-[#1a1a2f] px-6 py-3 flex items-center gap-8">
        <Link to="/" className="text-xl font-semibold">
          BotGlue
        </Link>
        <nav className="flex gap-4">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`text-sm ${
                location.pathname === item.path
                  ? "text-[#f0f0f5]"
                  : "text-[#6b6b7b] hover:text-[#a0a0b0]"
              }`}
            >
              {item.label}
            </Link>
          ))}
        </nav>
      </header>
      <main className="px-6 py-6">
        <Outlet />
      </main>
    </div>
  );
}
```

**Step 2: Create placeholder Dashboard page**

```typescript
// web/src/pages/DashboardPage.tsx

export function DashboardPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold mb-4">Dashboard</h1>
      <p className="text-[#6b6b7b]">Loading...</p>
    </div>
  );
}
```

**Step 3: Replace App.tsx with router configuration**

```typescript
// web/src/App.tsx

import { Routes, Route } from "react-router";
import { AppLayout } from "./layouts/AppLayout";
import { DashboardPage } from "./pages/DashboardPage";

function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<DashboardPage />} />
      </Route>
    </Routes>
  );
}

export default App;
```

**Step 4: Wrap main.tsx with BrowserRouter**

```typescript
// web/src/main.tsx

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router";
import "./index.css";
import App from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </StrictMode>
);
```

**Step 5: Verify it compiles**

```bash
cd web && pnpm build
```

Expected: builds successfully with no errors.

**Step 6: Commit**

```bash
git add web/src/layouts/AppLayout.tsx web/src/pages/DashboardPage.tsx web/src/App.tsx web/src/main.tsx
git commit -m "feat: add app shell layout with React Router"
```

---

### Task 4: ProjectCard shared component

**Files:**
- Create: `ui-common/components/ProjectCard.tsx`
- Modify: `ui-common/components/index.ts` (add export)

**Context:**
- Design doc section 3 lists `ProjectCard` as a shared component: "project overview with active environment count"
- Follow the same pattern as `AgentStatusBadge.tsx` — presentational, props-driven, Tailwind styled
- `ui-common/types/models.ts` has the `Project` interface

**Step 1: Create ProjectCard component**

```typescript
// ui-common/components/ProjectCard.tsx

import type { Project } from "../types";

interface ProjectCardProps {
  project: Project;
  environmentCount: number;
  agentCount: number;
  onClick?: () => void;
}

export function ProjectCard({
  project,
  environmentCount,
  agentCount,
  onClick,
}: ProjectCardProps) {
  return (
    <div
      onClick={onClick}
      className={`rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 ${
        onClick ? "cursor-pointer hover:border-[#2a2a4f]" : ""
      }`}
    >
      <h3 className="text-lg font-medium mb-1">{project.name}</h3>
      <p className="text-sm text-[#6b6b7b] mb-3 truncate">{project.repo_url}</p>
      <div className="flex gap-4 text-sm text-[#a0a0b0]">
        <span>
          {environmentCount} {environmentCount === 1 ? "env" : "envs"}
        </span>
        <span>
          {agentCount} {agentCount === 1 ? "agent" : "agents"}
        </span>
      </div>
    </div>
  );
}
```

**Step 2: Add to barrel export**

Add to `ui-common/components/index.ts`:
```typescript
export { ProjectCard } from "./ProjectCard";
```

**Step 3: Commit**

```bash
git add ui-common/components/ProjectCard.tsx ui-common/components/index.ts
git commit -m "feat: add ProjectCard shared component"
```

---

### Task 5: EnvironmentCard shared component

**Files:**
- Create: `ui-common/components/EnvironmentCard.tsx`
- Modify: `ui-common/components/index.ts` (add export)

**Context:**
- Design doc section 3: `EnvironmentCard` — "environment status with branch, ports, resource usage, controls"
- For now, skip resource usage (not tracked yet) and controls (needs callbacks). Show: branch, status badge, port count, agent count
- Environment status values: "creating" | "running" | "paused" | "destroyed"

**Step 1: Create EnvironmentCard component**

```typescript
// ui-common/components/EnvironmentCard.tsx

import type { Environment } from "../types";

const statusColors: Record<Environment["status"], string> = {
  creating: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  running: "bg-green-500/20 text-green-400 border-green-500/30",
  paused: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
  destroyed: "bg-[#333]/50 text-[#666] border-[#333]",
};

interface EnvironmentCardProps {
  environment: Environment;
  agentCount: number;
  onClick?: () => void;
}

export function EnvironmentCard({
  environment,
  agentCount,
  onClick,
}: EnvironmentCardProps) {
  return (
    <div
      onClick={onClick}
      className={`rounded-lg border border-[#1a1a2f] bg-[#0e0e1a] p-3 ${
        onClick ? "cursor-pointer hover:border-[#2a2a4f]" : ""
      }`}
    >
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-mono">{environment.branch}</span>
        <span
          className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border ${statusColors[environment.status]}`}
        >
          {environment.status}
        </span>
      </div>
      <div className="flex gap-4 text-xs text-[#6b6b7b]">
        <span>{agentCount} {agentCount === 1 ? "agent" : "agents"}</span>
        {environment.ports.length > 0 && (
          <span>{environment.ports.length} {environment.ports.length === 1 ? "port" : "ports"}</span>
        )}
      </div>
    </div>
  );
}
```

**Step 2: Add to barrel export**

Add to `ui-common/components/index.ts`:
```typescript
export { EnvironmentCard } from "./EnvironmentCard";
```

**Step 3: Commit**

```bash
git add ui-common/components/EnvironmentCard.tsx ui-common/components/index.ts
git commit -m "feat: add EnvironmentCard shared component"
```

---

### Task 6: Dashboard page — fetch and display data

**Files:**
- Modify: `web/src/pages/DashboardPage.tsx` (replace placeholder)

**Context:**
- API client is at `@botglue/common/api` — import `{ api }` from there
- Shared components at `@botglue/common/components` — `ProjectCard`, `EnvironmentCard`, `AgentStatusBadge`
- Types at `@botglue/common/types` — `Project`, `Environment`, `Agent`
- Design doc section 5 Dashboard spec:
  - Attention queue at top — agents needing attention (blocked > error > finished)
  - Active environments grouped by project
- The daemon serves on localhost:3001, Vite proxies `/api` to it (configured in `web/vite.config.ts`)
- `web/tsconfig.app.json` has `"verbatimModuleSyntax": true` — use `import type` for type-only imports
- Keep it simple — `useEffect` + `useState` for data fetching, no external state library

**Step 1: Implement the Dashboard page**

```typescript
// web/src/pages/DashboardPage.tsx

import { useState, useEffect } from "react";
import type { Project, Environment, Agent } from "@botglue/common/types";
import { api } from "@botglue/common/api";
import {
  ProjectCard,
  EnvironmentCard,
  AgentStatusBadge,
} from "@botglue/common/components";

interface ProjectData {
  project: Project;
  environments: Environment[];
  agents: Agent[];
}

export function DashboardPage() {
  const [projects, setProjects] = useState<ProjectData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      setLoading(true);
      setError(null);
      const projectList = await api.projects.list();

      const data = await Promise.all(
        projectList.map(async (project) => {
          const environments = await api.environments.list(project.id);
          const agentLists = await Promise.all(
            environments.map((env) => api.agents.list(env.id))
          );
          const agents = agentLists.flat();
          return { project, environments, agents };
        })
      );

      setProjects(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load data");
    } finally {
      setLoading(false);
    }
  }

  const attentionAgents = projects
    .flatMap((p) =>
      p.agents
        .filter((a) => a.status === "blocked" || a.status === "error" || a.status === "finished")
        .map((a) => ({
          agent: a,
          project: p.project,
          environment: p.environments.find((e) => e.id === a.env_id),
        }))
    )
    .sort((a, b) => {
      const priority: Record<string, number> = { blocked: 0, error: 1, finished: 2 };
      return (priority[a.agent.status] ?? 3) - (priority[b.agent.status] ?? 3);
    });

  if (loading) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Dashboard</h1>
        <p className="text-[#6b6b7b]">Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Dashboard</h1>
        <p className="text-red-400">{error}</p>
        <button
          onClick={loadData}
          className="mt-2 text-sm text-[#a0a0b0] hover:text-[#f0f0f5] underline"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Dashboard</h1>

      {/* Attention Queue */}
      {attentionAgents.length > 0 && (
        <section className="mb-8">
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide mb-3">
            Needs Attention
          </h2>
          <div className="space-y-2">
            {attentionAgents.map(({ agent, project, environment }) => (
              <div
                key={agent.id}
                className="flex items-center gap-3 rounded-lg border border-[#1a1a2f] bg-[#12121f] p-3"
              >
                <AgentStatusBadge status={agent.status} />
                <div className="flex-1 min-w-0">
                  <span className="text-sm font-medium">{project.name}</span>
                  {environment && (
                    <span className="text-[#6b6b7b] text-sm"> / {environment.branch}</span>
                  )}
                  <p className="text-xs text-[#6b6b7b] truncate">
                    {agent.blocker || agent.current_task}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Projects */}
      {projects.length === 0 ? (
        <p className="text-[#6b6b7b]">No projects yet. Create one to get started.</p>
      ) : (
        <section>
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide mb-3">
            Projects
          </h2>
          <div className="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
            {projects.map(({ project, environments, agents }) => (
              <div key={project.id}>
                <ProjectCard
                  project={project}
                  environmentCount={environments.length}
                  agentCount={agents.length}
                />
                {environments.length > 0 && (
                  <div className="mt-2 ml-2 space-y-1">
                    {environments.map((env) => (
                      <EnvironmentCard
                        key={env.id}
                        environment={env}
                        agentCount={agents.filter((a) => a.env_id === env.id).length}
                      />
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
```

**Step 2: Verify it compiles**

```bash
cd web && pnpm build
```

Expected: builds successfully.

**Step 3: Commit**

```bash
git add web/src/pages/DashboardPage.tsx
git commit -m "feat: implement Dashboard page with live data"
```

---

### Task 7: End-to-end verification

**Files:** None (verification only)

**Step 1: Build the web SPA**

```bash
cd web && pnpm build
```

Expected: builds successfully, outputs to `web/dist/`.

**Step 2: Run daemon and verify**

```bash
cd daemon && cargo run &
```

Wait for "BotGlue daemon listening on http://127.0.0.1:3001", then:

```bash
# Check health
curl http://localhost:3001/api/health

# Create a project
curl -X POST http://localhost:3001/api/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"test-project","repo_url":"https://github.com/test/repo","default_branch":"main"}'

# Verify dashboard loads the SPA
curl -s http://localhost:3001/ | head -5
```

Expected:
- Health returns `{"status":"ok","version":"0.1.0"}`
- Project creation returns 201 with project JSON
- Root URL returns the SPA HTML

**Step 3: Stop the daemon**

```bash
kill %1
```

No commit needed — this is verification only.
