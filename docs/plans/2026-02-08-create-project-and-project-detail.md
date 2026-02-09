# Create Project Form + Project Detail Page Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a "Create Project" form to the dashboard and a Project Detail page (`/projects/:id`) showing project info, its environments, and a "Create Environment" form.

**Architecture:** Both forms are inline (no modals) — the create project form appears at the top of the dashboard, the create environment form appears at the top of the project detail page. After successful creation, the form resets and the data reloads. Navigation between dashboard and project detail uses React Router `useNavigate` and `Link`. All data flows through the existing `api` client in `ui-common/api/`.

**Tech Stack:** TypeScript, React 19, React Router 7, Tailwind CSS v4

---

### Task 1: Create Project form on Dashboard

**Files:**
- Create: `web/src/components/CreateProjectForm.tsx`
- Modify: `web/src/pages/DashboardPage.tsx`

**Context:**
- The API client `api.projects.create` accepts `{ name: string; repo_url: string; default_branch: string }`. The daemon's `CreateProject` struct makes `default_branch` optional (defaults to "main"), but the API client types it as required — just send `"main"` as default.
- `web/tsconfig.app.json` has `"verbatimModuleSyntax": true` — use `import type` for type-only imports.
- Dark theme colors: bg `#0a0a0f`, card bg `#12121f`, border `#1a1a2f`, muted text `#6b6b7b`, text `#f0f0f5`.

**Step 1: Create the form component**

```typescript
// web/src/components/CreateProjectForm.tsx

import { useState } from "react";
import { api } from "@botglue/common/api";

interface CreateProjectFormProps {
  onCreated: () => void;
}

export function CreateProjectForm({ onCreated }: CreateProjectFormProps) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [repoUrl, setRepoUrl] = useState("");
  const [defaultBranch, setDefaultBranch] = useState("main");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.projects.create({
        name,
        repo_url: repoUrl,
        default_branch: defaultBranch,
      });
      setName("");
      setRepoUrl("");
      setDefaultBranch("main");
      setOpen(false);
      onCreated();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create project");
    } finally {
      setSubmitting(false);
    }
  }

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        className="text-sm text-[#a0a0b0] hover:text-[#f0f0f5] border border-dashed border-[#2a2a4f] rounded-lg px-4 py-2"
      >
        + New Project
      </button>
    );
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 space-y-3"
    >
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">New Project</h3>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="text-[#6b6b7b] hover:text-[#f0f0f5] text-sm"
        >
          Cancel
        </button>
      </div>
      <input
        type="text"
        placeholder="Project name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        required
        className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
      />
      <input
        type="text"
        placeholder="Repository URL"
        value={repoUrl}
        onChange={(e) => setRepoUrl(e.target.value)}
        required
        className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
      />
      <input
        type="text"
        placeholder="Default branch"
        value={defaultBranch}
        onChange={(e) => setDefaultBranch(e.target.value)}
        className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
      />
      {error && <p className="text-red-400 text-xs">{error}</p>}
      <button
        type="submit"
        disabled={submitting || !name || !repoUrl}
        className="bg-[#2a2a4f] hover:bg-[#3a3a5f] disabled:opacity-50 disabled:cursor-not-allowed text-sm px-4 py-1.5 rounded"
      >
        {submitting ? "Creating..." : "Create"}
      </button>
    </form>
  );
}
```

**Step 2: Add the form and navigation to DashboardPage**

Modify `web/src/pages/DashboardPage.tsx`:
- Import `CreateProjectForm` from `../components/CreateProjectForm`
- Import `useNavigate` from `react-router`
- Add `const navigate = useNavigate();` inside the component
- Add `<CreateProjectForm onCreated={loadData} />` between the heading and the attention queue
- Pass `onClick={() => navigate(`/projects/${project.id}`)}` to each `<ProjectCard>`

The heading section becomes:
```typescript
<div className="flex items-center justify-between mb-6">
  <h1 className="text-2xl font-semibold">Dashboard</h1>
  <CreateProjectForm onCreated={loadData} />
</div>
```

And the ProjectCard gets an onClick:
```typescript
<ProjectCard
  project={project}
  environmentCount={environments.length}
  agentCount={agents.length}
  onClick={() => navigate(`/projects/${project.id}`)}
/>
```

**Step 3: Verify it compiles**

```bash
cd web && pnpm build
```

**Step 4: Commit**

```bash
git add web/src/components/CreateProjectForm.tsx web/src/pages/DashboardPage.tsx
git commit -m "feat: add Create Project form to dashboard"
```

---

### Task 2: Project Detail page

**Files:**
- Create: `web/src/pages/ProjectDetailPage.tsx`
- Modify: `web/src/App.tsx` (add route)

**Context:**
- Route: `/projects/:id`
- Uses `useParams` from `react-router` to get the project ID
- Fetches project, its environments, and agents for each environment
- Shows: project name, repo URL, default branch, environment list with EnvironmentCard, delete project button
- Reuse `EnvironmentCard` from `@botglue/common/components`
- `web/tsconfig.app.json` has `"verbatimModuleSyntax": true` — use `import type` for type-only imports

**Step 1: Create the page**

```typescript
// web/src/pages/ProjectDetailPage.tsx

import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router";
import type { Project, Environment, Agent } from "@botglue/common/types";
import { api } from "@botglue/common/api";
import { EnvironmentCard, AgentStatusBadge } from "@botglue/common/components";

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [agents, setAgents] = useState<Map<string, Agent[]>>(new Map());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (id) loadData();
  }, [id]);

  async function loadData() {
    try {
      setLoading(true);
      setError(null);
      const proj = await api.projects.get(id!);
      setProject(proj);

      const envs = await api.environments.list(id!);
      setEnvironments(envs);

      const agentMap = new Map<string, Agent[]>();
      await Promise.all(
        envs.map(async (env) => {
          const envAgents = await api.agents.list(env.id);
          agentMap.set(env.id, envAgents);
        })
      );
      setAgents(agentMap);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load project");
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete() {
    if (!project) return;
    try {
      await api.projects.delete(project.id);
      navigate("/");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete project");
    }
  }

  if (loading) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Project</h1>
        <p className="text-[#6b6b7b]">Loading...</p>
      </div>
    );
  }

  if (error || !project) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Project</h1>
        <p className="text-red-400">{error || "Project not found"}</p>
        <button
          onClick={() => navigate("/")}
          className="mt-2 text-sm text-[#a0a0b0] hover:text-[#f0f0f5] underline"
        >
          Back to Dashboard
        </button>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div>
          <button
            onClick={() => navigate("/")}
            className="text-sm text-[#6b6b7b] hover:text-[#a0a0b0] mb-2"
          >
            &larr; Dashboard
          </button>
          <h1 className="text-2xl font-semibold">{project.name}</h1>
        </div>
        <button
          onClick={handleDelete}
          className="text-sm text-red-400/70 hover:text-red-400 border border-red-400/30 hover:border-red-400/50 rounded px-3 py-1"
        >
          Delete Project
        </button>
      </div>

      {/* Project Info */}
      <section className="mb-8 rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4">
        <div className="grid gap-2 text-sm">
          <div>
            <span className="text-[#6b6b7b]">Repository:</span>{" "}
            <span className="font-mono">{project.repo_url}</span>
          </div>
          <div>
            <span className="text-[#6b6b7b]">Default branch:</span>{" "}
            <span className="font-mono">{project.default_branch}</span>
          </div>
        </div>
      </section>

      {/* Environments */}
      <section>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide">
            Environments
          </h2>
        </div>
        {environments.length === 0 ? (
          <p className="text-[#6b6b7b] text-sm">No environments yet.</p>
        ) : (
          <div className="space-y-2">
            {environments.map((env) => {
              const envAgents = agents.get(env.id) || [];
              return (
                <div key={env.id}>
                  <EnvironmentCard
                    environment={env}
                    agentCount={envAgents.length}
                  />
                  {envAgents.length > 0 && (
                    <div className="ml-4 mt-1 space-y-1">
                      {envAgents.map((agent) => (
                        <div
                          key={agent.id}
                          className="flex items-center gap-2 text-sm"
                        >
                          <AgentStatusBadge status={agent.status} />
                          <span className="text-[#6b6b7b] truncate">
                            {agent.current_task}
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </section>
    </div>
  );
}
```

**Step 2: Add the route to App.tsx**

Modify `web/src/App.tsx`:
- Import `ProjectDetailPage` from `./pages/ProjectDetailPage`
- Add route: `<Route path="projects/:id" element={<ProjectDetailPage />} />`

The full App.tsx becomes:
```typescript
import { Routes, Route } from "react-router";
import { AppLayout } from "./layouts/AppLayout";
import { DashboardPage } from "./pages/DashboardPage";
import { ProjectDetailPage } from "./pages/ProjectDetailPage";

function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<DashboardPage />} />
        <Route path="projects/:id" element={<ProjectDetailPage />} />
      </Route>
    </Routes>
  );
}

export default App;
```

**Step 3: Verify it compiles**

```bash
cd web && pnpm build
```

**Step 4: Commit**

```bash
git add web/src/pages/ProjectDetailPage.tsx web/src/App.tsx
git commit -m "feat: add Project Detail page with environments and agents"
```

---

### Task 3: Create Environment form on Project Detail page

**Files:**
- Create: `web/src/components/CreateEnvironmentForm.tsx`
- Modify: `web/src/pages/ProjectDetailPage.tsx`

**Context:**
- The API client `api.environments.create` accepts `{ project_id: string; branch: string; container_id: string; ports: string }`.
- The daemon's `CreateEnvironment` struct makes `container_id` and `ports` optional. But the API client currently types `container_id: string` and `ports: string`. For now, send empty string for container_id and `"[]"` for ports — the daemon handles optional/default values.
- Actually, looking at the daemon code, `CreateEnvironment` accepts `container_id: Option<String>` and `ports: Option<Vec<PortMapping>>`. The API client type is slightly wrong (it types `ports` as `string` instead of a JSON array). We need to fix the API client type to match what the daemon actually accepts, or work around it. The simplest fix: update the API client's `environments.create` type to accept `container_id?: string` and `ports?: PortMapping[]` and serialize properly.

**Step 1: Fix the environments.create type in the API client**

Modify `ui-common/api/client.ts` lines 45-50. Change the `create` method's data parameter from:
```typescript
create: (data: {
  project_id: string;
  branch: string;
  container_id: string;
  ports: string;
})
```
to:
```typescript
create: (data: {
  project_id: string;
  branch: string;
  container_id?: string;
  ports?: { name: string; container_port: number; host_port?: number; protocol?: string }[];
})
```

This matches the daemon's `CreateEnvironment` struct where `container_id` and `ports` are optional.

**Step 2: Create the form component**

```typescript
// web/src/components/CreateEnvironmentForm.tsx

import { useState } from "react";
import { api } from "@botglue/common/api";

interface CreateEnvironmentFormProps {
  projectId: string;
  onCreated: () => void;
}

export function CreateEnvironmentForm({
  projectId,
  onCreated,
}: CreateEnvironmentFormProps) {
  const [open, setOpen] = useState(false);
  const [branch, setBranch] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.environments.create({
        project_id: projectId,
        branch,
      });
      setBranch("");
      setOpen(false);
      onCreated();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to create environment"
      );
    } finally {
      setSubmitting(false);
    }
  }

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        className="text-sm text-[#a0a0b0] hover:text-[#f0f0f5] border border-dashed border-[#2a2a4f] rounded-lg px-4 py-2"
      >
        + New Environment
      </button>
    );
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 space-y-3"
    >
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">New Environment</h3>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="text-[#6b6b7b] hover:text-[#f0f0f5] text-sm"
        >
          Cancel
        </button>
      </div>
      <input
        type="text"
        placeholder="Branch name (e.g. feature/login)"
        value={branch}
        onChange={(e) => setBranch(e.target.value)}
        required
        className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
      />
      {error && <p className="text-red-400 text-xs">{error}</p>}
      <button
        type="submit"
        disabled={submitting || !branch}
        className="bg-[#2a2a4f] hover:bg-[#3a3a5f] disabled:opacity-50 disabled:cursor-not-allowed text-sm px-4 py-1.5 rounded"
      >
        {submitting ? "Creating..." : "Create"}
      </button>
    </form>
  );
}
```

**Step 3: Add the form to ProjectDetailPage**

Modify `web/src/pages/ProjectDetailPage.tsx`:
- Import `CreateEnvironmentForm` from `../components/CreateEnvironmentForm`
- In the Environments section, add `<CreateEnvironmentForm projectId={project.id} onCreated={loadData} />` next to the "Environments" heading

The environments header becomes:
```typescript
<div className="flex items-center justify-between mb-3">
  <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide">
    Environments
  </h2>
  <CreateEnvironmentForm projectId={project.id} onCreated={loadData} />
</div>
```

**Step 4: Verify it compiles**

```bash
cd web && pnpm build
```

**Step 5: Commit**

```bash
git add ui-common/api/client.ts web/src/components/CreateEnvironmentForm.tsx web/src/pages/ProjectDetailPage.tsx
git commit -m "feat: add Create Environment form to project detail page"
```

---

### Task 4: End-to-end verification

**Files:** None (verification only)

**Step 1: Build web SPA**

```bash
cd web && pnpm build
```

**Step 2: Run daemon and test the flow**

```bash
cd daemon && cargo run &
```

Wait for "BotGlue daemon listening on http://127.0.0.1:3001", then:

```bash
# Verify SPA loads
curl -s http://localhost:3001/ | head -5

# Dashboard should show empty state
curl -s http://localhost:3001/api/projects

# Create a project via API (as if the form submitted)
curl -s -X POST http://localhost:3001/api/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"test-project","repo_url":"https://github.com/test/repo","default_branch":"main"}'

# Get the project ID from response, then test project detail route (SPA fallback)
curl -s http://localhost:3001/projects/some-id | head -5

# Create an environment
curl -s -X POST http://localhost:3001/api/environments \
  -H 'Content-Type: application/json' \
  -d '{"project_id":"<PROJECT_ID>","branch":"feature/test"}'
```

Expected:
- SPA HTML is returned for both `/` and `/projects/some-id` (SPA fallback works)
- Project creation returns 201
- Environment creation returns 201 (with `container_id: ""` and `ports: []` as defaults)

**Step 3: Stop daemon and clean up**

```bash
kill %1
rm -f daemon/botglue.db*
```

No commit needed — verification only.
