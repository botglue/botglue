# Environment Detail Page + Environment Actions Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an Environment Detail page at `/projects/:projectId/environments/:envId` showing environment info, port links, action buttons (pause/resume/delete), agent list, and a Create Agent form. Wire up navigation from the Project Detail page.

**Architecture:** The page fetches a single environment and its agents via the existing API client. Action buttons call existing `api.environments.pause/resume/delete` methods. A Create Agent form lets users add agents to the environment. Navigation flows: Dashboard → Project Detail → Environment Detail. The back link returns to the project.

**Tech Stack:** TypeScript, React 19, React Router 7, Tailwind CSS v4

---

### Task 1: Navigate to Environment Detail from Project Detail

**Files:**
- Modify: `web/src/pages/ProjectDetailPage.tsx`
- Modify: `web/src/App.tsx`

**Context:**
- `EnvironmentCard` already accepts an `onClick?: () => void` prop and shows a hover effect when it's set.
- The route will be `/projects/:projectId/environments/:envId` so environment detail knows which project to link back to.
- `web/tsconfig.app.json` has `"verbatimModuleSyntax": true` — use `import type` for type-only imports.

**Step 1: Add onClick to EnvironmentCard in ProjectDetailPage**

In `web/src/pages/ProjectDetailPage.tsx`, the `EnvironmentCard` is rendered around line 129. Add an `onClick` that navigates to the environment detail:

```typescript
<EnvironmentCard
  environment={env}
  agentCount={envAgents.length}
  onClick={() => navigate(`/projects/${id}/environments/${env.id}`)}
/>
```

Note: `navigate` and `id` (project ID from useParams) are already available in this component.

**Step 2: Add the route to App.tsx**

In `web/src/App.tsx`, add the environment detail route. The page component doesn't exist yet — create a placeholder that we'll implement in Task 2.

Add import:
```typescript
import { EnvironmentDetailPage } from "./pages/EnvironmentDetailPage";
```

Add route inside the `<Route element={<AppLayout />}>` block:
```typescript
<Route path="projects/:projectId/environments/:envId" element={<EnvironmentDetailPage />} />
```

**Step 3: Create a placeholder page**

Create `web/src/pages/EnvironmentDetailPage.tsx` with a minimal placeholder:

```typescript
import { useParams } from "react-router";

export function EnvironmentDetailPage() {
  const { projectId, envId } = useParams<{ projectId: string; envId: string }>();
  return (
    <div>
      <h1 className="text-2xl font-semibold mb-4">Environment {envId}</h1>
      <p className="text-[#6b6b7b]">Project: {projectId}</p>
    </div>
  );
}
```

**Step 4: Verify it compiles**

```bash
cd web && pnpm build
```

**Step 5: Commit**

```bash
git add web/src/pages/ProjectDetailPage.tsx web/src/App.tsx web/src/pages/EnvironmentDetailPage.tsx
git commit -m "feat: add environment detail route and navigation from project page"
```

---

### Task 2: Environment Detail page — info, ports, and action buttons

**Files:**
- Modify: `web/src/pages/EnvironmentDetailPage.tsx` (replace placeholder)

**Context:**
- Route params: `projectId` and `envId` from `/projects/:projectId/environments/:envId`
- Fetch environment via `api.environments.get(envId)` and project via `api.projects.get(projectId)` (for showing project name in breadcrumb)
- Fetch agents via `api.agents.list(envId)`
- Show environment info: branch, status, container_id (if set), created_at, last_active
- Show port links: for each port with a `host_port`, render a clickable link to `http://localhost:<host_port>` that opens in a new tab
- Action buttons depend on status:
  - `running` → show Pause and Delete buttons
  - `paused` → show Resume and Delete buttons
  - `creating` → show only Delete button
  - `destroyed` → show only "Back to Project" link
- After pause/resume, reload data. After delete, navigate back to project detail.
- Reuse `AgentStatusBadge` from `@botglue/common/components` for agent list
- Dark theme colors: bg `#0a0a0f`, card bg `#12121f`, border `#1a1a2f`, hover border `#2a2a4f`, muted text `#6b6b7b`, secondary text `#a0a0b0`, text `#f0f0f5`
- The TS types for Agent have the field named `agent_type` in the API response (the Rust struct uses `agent_type`), but the TypeScript type in `ui-common/types/models.ts` may name it `type`. Check what the daemon actually serializes. The daemon's `Agent` struct has field `agent_type` which Serde serializes as `agent_type`. The TypeScript `Agent` type should match — if it uses `type` instead of `agent_type`, display code should use whichever field name the TS type defines.

**Step 1: Implement the full page**

Replace the placeholder in `web/src/pages/EnvironmentDetailPage.tsx`:

```typescript
import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router";
import type { Project, Environment, Agent } from "@botglue/common/types";
import { api } from "@botglue/common/api";
import { AgentStatusBadge } from "@botglue/common/components";

export function EnvironmentDetailPage() {
  const { projectId, envId } = useParams<{
    projectId: string;
    envId: string;
  }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [environment, setEnvironment] = useState<Environment | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState(false);

  useEffect(() => {
    if (projectId && envId) loadData();
  }, [projectId, envId]);

  async function loadData() {
    try {
      setLoading(true);
      setError(null);
      const [proj, env, agentList] = await Promise.all([
        api.projects.get(projectId!),
        api.environments.get(envId!),
        api.agents.list(envId!),
      ]);
      setProject(proj);
      setEnvironment(env);
      setAgents(agentList);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load environment");
    } finally {
      setLoading(false);
    }
  }

  async function handlePause() {
    setActionLoading(true);
    try {
      await api.environments.pause(envId!);
      await loadData();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to pause");
    } finally {
      setActionLoading(false);
    }
  }

  async function handleResume() {
    setActionLoading(true);
    try {
      await api.environments.resume(envId!);
      await loadData();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to resume");
    } finally {
      setActionLoading(false);
    }
  }

  async function handleDelete() {
    setActionLoading(true);
    try {
      await api.environments.delete(envId!);
      navigate(`/projects/${projectId}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to delete");
      setActionLoading(false);
    }
  }

  if (loading) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Environment</h1>
        <p className="text-[#6b6b7b]">Loading...</p>
      </div>
    );
  }

  if (error || !environment) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Environment</h1>
        <p className="text-red-400">{error || "Environment not found"}</p>
        <button
          onClick={() => navigate(`/projects/${projectId}`)}
          className="mt-2 text-sm text-[#a0a0b0] hover:text-[#f0f0f5] underline"
        >
          Back to Project
        </button>
      </div>
    );
  }

  const statusColors: Record<string, string> = {
    creating: "bg-blue-500/20 text-blue-400 border-blue-500/30",
    running: "bg-green-500/20 text-green-400 border-green-500/30",
    paused: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
    destroyed: "bg-[#333]/50 text-[#666] border-[#333]",
  };

  const portsWithHost = environment.ports.filter((p) => p.host_port);

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <button
            onClick={() => navigate(`/projects/${projectId}`)}
            className="text-sm text-[#6b6b7b] hover:text-[#a0a0b0] mb-2"
          >
            &larr; {project?.name || "Project"}
          </button>
          <h1 className="text-2xl font-semibold">{environment.branch}</h1>
        </div>
        <div className="flex items-center gap-2">
          {environment.status === "running" && (
            <button
              onClick={handlePause}
              disabled={actionLoading}
              className="text-sm text-yellow-400/70 hover:text-yellow-400 border border-yellow-400/30 hover:border-yellow-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Pause
            </button>
          )}
          {environment.status === "paused" && (
            <button
              onClick={handleResume}
              disabled={actionLoading}
              className="text-sm text-green-400/70 hover:text-green-400 border border-green-400/30 hover:border-green-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Resume
            </button>
          )}
          {environment.status !== "destroyed" && (
            <button
              onClick={handleDelete}
              disabled={actionLoading}
              className="text-sm text-red-400/70 hover:text-red-400 border border-red-400/30 hover:border-red-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Delete
            </button>
          )}
        </div>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      {/* Environment Info */}
      <section className="mb-8 rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4">
        <div className="grid gap-2 text-sm">
          <div className="flex items-center gap-2">
            <span className="text-[#6b6b7b]">Status:</span>
            <span
              className={`inline-block rounded-full border px-2 py-0.5 text-xs font-medium ${statusColors[environment.status] || ""}`}
            >
              {environment.status}
            </span>
          </div>
          <div>
            <span className="text-[#6b6b7b]">Branch:</span>{" "}
            <span className="font-mono">{environment.branch}</span>
          </div>
          {environment.container_id && (
            <div>
              <span className="text-[#6b6b7b]">Container:</span>{" "}
              <span className="font-mono text-xs">{environment.container_id}</span>
            </div>
          )}
          <div>
            <span className="text-[#6b6b7b]">Created:</span>{" "}
            <span>{new Date(environment.created_at).toLocaleString()}</span>
          </div>
          <div>
            <span className="text-[#6b6b7b]">Last active:</span>{" "}
            <span>{new Date(environment.last_active).toLocaleString()}</span>
          </div>
        </div>
      </section>

      {/* Ports */}
      {portsWithHost.length > 0 && (
        <section className="mb-8">
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide mb-3">
            Ports
          </h2>
          <div className="flex flex-wrap gap-2">
            {portsWithHost.map((port) => (
              <a
                key={port.container_port}
                href={`http://localhost:${port.host_port}`}
                target="_blank"
                rel="noopener noreferrer"
                className="rounded border border-[#1a1a2f] bg-[#12121f] px-3 py-1.5 text-sm hover:border-[#2a2a4f]"
              >
                <span className="text-[#a0a0b0]">{port.name}</span>
                <span className="text-[#6b6b7b] ml-2">:{port.host_port}</span>
              </a>
            ))}
          </div>
        </section>
      )}

      {/* Agents */}
      <section>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide">
            Agents
          </h2>
        </div>
        {agents.length === 0 ? (
          <p className="text-[#6b6b7b] text-sm">No agents in this environment.</p>
        ) : (
          <div className="space-y-2">
            {agents.map((agent) => (
              <div
                key={agent.id}
                className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-3 flex items-center gap-3"
              >
                <AgentStatusBadge status={agent.status} />
                <div className="flex-1 min-w-0">
                  <span className="text-sm font-medium">{agent.agent_type}</span>
                  <p className="text-xs text-[#6b6b7b] truncate">
                    {agent.blocker || agent.current_task}
                  </p>
                </div>
                <span className="text-xs text-[#6b6b7b]">
                  {new Date(agent.last_activity).toLocaleString()}
                </span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
```

**Important note on Agent field name:** The daemon serializes the field as `agent_type` (Rust struct field name). Check if the TypeScript `Agent` type uses `type` or `agent_type`. If it uses `type`, you'll need to use `agent.type` in the JSX above. Read `ui-common/types/models.ts` to confirm and adjust accordingly.

**Step 2: Verify it compiles**

```bash
cd web && pnpm build
```

**Step 3: Commit**

```bash
git add web/src/pages/EnvironmentDetailPage.tsx
git commit -m "feat: implement Environment Detail page with info, ports, and actions"
```

---

### Task 3: Create Agent form on Environment Detail

**Files:**
- Create: `web/src/components/CreateAgentForm.tsx`
- Modify: `web/src/pages/EnvironmentDetailPage.tsx`

**Context:**
- The API client `api.agents.create` accepts `{ env_id: string; agent_type: string; current_task: string }`.
- Agent types from the design doc: `"claude" | "cursor" | "opencode" | "custom"`.
- The form should follow the same toggle pattern as CreateProjectForm and CreateEnvironmentForm.
- Dark theme colors same as other forms.

**Step 1: Create the form component**

```typescript
// web/src/components/CreateAgentForm.tsx

import { useState } from "react";
import { api } from "@botglue/common/api";

interface CreateAgentFormProps {
  envId: string;
  onCreated: () => void;
}

const AGENT_TYPES = ["claude", "cursor", "opencode", "custom"];

export function CreateAgentForm({ envId, onCreated }: CreateAgentFormProps) {
  const [open, setOpen] = useState(false);
  const [agentType, setAgentType] = useState("claude");
  const [currentTask, setCurrentTask] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.agents.create({
        env_id: envId,
        agent_type: agentType,
        current_task: currentTask,
      });
      setCurrentTask("");
      setAgentType("claude");
      setOpen(false);
      onCreated();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create agent");
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
        + New Agent
      </button>
    );
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 space-y-3"
    >
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">New Agent</h3>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="text-[#6b6b7b] hover:text-[#f0f0f5] text-sm"
        >
          Cancel
        </button>
      </div>
      <select
        value={agentType}
        onChange={(e) => setAgentType(e.target.value)}
        className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
      >
        {AGENT_TYPES.map((t) => (
          <option key={t} value={t}>
            {t}
          </option>
        ))}
      </select>
      <input
        type="text"
        placeholder="Task description (e.g. Implement login page)"
        value={currentTask}
        onChange={(e) => setCurrentTask(e.target.value)}
        required
        className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
      />
      {error && <p className="text-red-400 text-xs">{error}</p>}
      <button
        type="submit"
        disabled={submitting || !currentTask}
        className="bg-[#2a2a4f] hover:bg-[#3a3a5f] disabled:opacity-50 disabled:cursor-not-allowed text-sm px-4 py-1.5 rounded"
      >
        {submitting ? "Creating..." : "Create"}
      </button>
    </form>
  );
}
```

**Step 2: Add the form to EnvironmentDetailPage**

In `web/src/pages/EnvironmentDetailPage.tsx`:

Add import:
```typescript
import { CreateAgentForm } from "../components/CreateAgentForm";
```

In the Agents section header, add the form next to the heading:
```typescript
<div className="flex items-center justify-between mb-3">
  <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide">
    Agents
  </h2>
  <CreateAgentForm envId={envId!} onCreated={loadData} />
</div>
```

**Step 3: Verify it compiles**

```bash
cd web && pnpm build
```

**Step 4: Commit**

```bash
git add web/src/components/CreateAgentForm.tsx web/src/pages/EnvironmentDetailPage.tsx
git commit -m "feat: add Create Agent form to environment detail page"
```

---

### Task 4: End-to-end verification

**Files:** None (verification only)

**Step 1: Build web SPA**

```bash
cd web && pnpm build
```

**Step 2: Run daemon tests**

```bash
cd daemon && cargo test
```

**Step 3: Start daemon and test the full flow**

```bash
cd daemon && cargo run &
```

Wait for "BotGlue daemon listening on http://127.0.0.1:3001", then:

```bash
# Verify SPA loads
curl -s http://localhost:3001/ | head -5

# Create a project
PROJECT_ID=$(curl -s -X POST http://localhost:3001/api/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"test-project","repo_url":"https://github.com/test/repo","default_branch":"main"}' \
  | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['id'])")
echo "Project ID: $PROJECT_ID"

# Create an environment
ENV_ID=$(curl -s -X POST http://localhost:3001/api/environments \
  -H 'Content-Type: application/json' \
  -d "{\"project_id\":\"$PROJECT_ID\",\"branch\":\"feature/test\"}" \
  | python3 -c "import sys,json; print(json.loads(sys.stdin.read())['id'])")
echo "Environment ID: $ENV_ID"

# Create an agent in the environment
curl -s -X POST http://localhost:3001/api/agents \
  -H 'Content-Type: application/json' \
  -d "{\"env_id\":\"$ENV_ID\",\"agent_type\":\"claude\",\"current_task\":\"Implement login page\"}"

# Verify environment detail route (SPA fallback)
curl -s "http://localhost:3001/projects/$PROJECT_ID/environments/$ENV_ID" | head -5

# Test pause
curl -s -X POST "http://localhost:3001/api/environments/$ENV_ID/pause"

# Verify status changed
curl -s "http://localhost:3001/api/environments/$ENV_ID" | python3 -c "import sys,json; print('Status:', json.loads(sys.stdin.read())['status'])"

# Test resume
curl -s -X POST "http://localhost:3001/api/environments/$ENV_ID/resume"

# Verify status changed back
curl -s "http://localhost:3001/api/environments/$ENV_ID" | python3 -c "import sys,json; print('Status:', json.loads(sys.stdin.read())['status'])"

# Test environment delete
curl -s -X DELETE "http://localhost:3001/api/environments/$ENV_ID"

# Verify it's gone
curl -s "http://localhost:3001/api/environments/$ENV_ID"
```

Expected:
- SPA HTML returned for `/projects/.../environments/...` (SPA fallback works)
- Project, environment, and agent creation succeed
- Pause changes status to `paused`
- Resume changes status to `running`
- Delete returns 204 and environment is gone (404 on subsequent get)

**Step 4: Stop daemon and clean up**

```bash
kill %1 2>/dev/null || true
rm -f daemon/botglue.db*
```

No commit needed — verification only.
