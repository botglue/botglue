# BotGlue-Web Design Document

## 1. Overview

This document revises the original BotGlue architecture to postpone the Tauri desktop app and ship a web frontend first. A shared TypeScript library (`ui-common`) ensures components, types, and API client can be reused when the desktop app is built later.

**What changed from the original design:**
- Tauri desktop app → web SPA served by the daemon
- Mobile Tauri app → responsive web (works on mobile browsers)
- Added LLM Gateway as part of the daemon (keeps API keys out of containers)
- Added `AgentDriver` trait for pluggable agent interaction via `podman exec`
- Added configurable port mappings in environment templates

**What stayed the same:**
- Rust daemon, Podman containers, SQLite storage
- Data model (Project, Environment, Agent, AuditEntry)
- Agent Monitor, Notification Service, Autonomous Operations
- Implementation roadmap phases (same order, web instead of desktop)

---

## 2. Architecture Overview

```
botglue/
├── daemon/                # Rust daemon
│   ├── env_manager        # Podman container lifecycle
│   ├── agent_monitor      # Watch agent state via exec
│   ├── llm_gateway        # Proxy LLM calls, inject keys
│   ├── notification       # Browser notifications via WebSocket
│   ├── api_server         # REST + WebSocket for UI
│   └── static_files       # Serves web/dist/
│
├── ui-common/             # Shared TypeScript library
│   ├── components/        # React components
│   ├── api/               # Typed API client
│   └── types/             # Shared interfaces
│
├── web/                   # SPA (React + Vite)
│   ├── src/
│   │   ├── pages/
│   │   ├── layouts/
│   │   └── main.tsx
│   ├── vite.config.ts
│   └── package.json
│
└── docs/
```

```
┌──────────────────────────────────────────────────────┐
│                  BotGlue Daemon                       │
│  ┌──────────┐ ┌──────────┐ ┌───────────────────────┐ │
│  │ API      │ │ LLM      │ │ Static file server    │ │
│  │ Server   │ │ Gateway   │ │ (web/dist/)           │ │
│  └──────────┘ └──────────┘ └───────────────────────┘ │
│  ┌──────────┐ ┌──────────┐ ┌───────────────────────┐ │
│  │ Env      │ │ Agent    │ │ Notification          │ │
│  │ Manager  │ │ Monitor  │ │ Service               │ │
│  └──────────┘ └──────────┘ └───────────────────────┘ │
└──────────────────────┬───────────────────────────────┘
                       │ podman exec
          ┌────────────┼────────────┐
          ▼            ▼            ▼
   ┌───────────┐ ┌───────────┐ ┌───────────┐
   │ Container │ │ Container │ │ Container │
   │ Agent A   │ │ Agent B   │ │ Agent C   │
   │ Branch X  │ │ Branch Y  │ │ Branch Z  │
   │ ports: [] │ │ ports:    │ │ ports:    │
   │           │ │  ui:3000  │ │  api:8080 │
   │           │ │  api:8080 │ │  ws:8081  │
   └───────────┘ └───────────┘ └───────────┘
       LLM calls → daemon:gateway_port (no raw keys in containers)
```

**Container communication:** All interaction with containers happens via `podman exec`. No agent driver process inside containers. Ports are only for dev servers, APIs, and other application endpoints the agent is building.

---

## 3. Shared TypeScript Library (`ui-common`)

### Types (`ui-common/types/`)

```typescript
// models.ts
interface Project {
  id: string;
  name: string;
  repo_url: string;
  default_branch: string;
  notification_prefs: NotificationPrefs;
  created_at: string;
}

interface Environment {
  id: string;
  project_id: string;
  branch: string;
  status: "creating" | "running" | "paused" | "destroyed";
  container_id: string;
  ports: PortMapping[];
  created_at: string;
  last_active: string;
}

interface PortMapping {
  name: string;           // human label: "ui", "api", "ws"
  container_port: number;
  host_port?: number;     // auto-assigned from range if omitted
  protocol?: "http" | "ws";
}

interface Agent {
  id: string;
  env_id: string;
  type: "claude" | "cursor" | "opencode" | "custom";
  status: "running" | "blocked" | "finished" | "error";
  current_task: string;
  blocker: string | null;
  started_at: string;
  last_activity: string;
}

interface AuditEntry {
  id: string;
  env_id: string;
  agent_id: string;
  operation: string;
  command: string;
  output: string;
  exit_code: number;
  timestamp: string;
}

interface LLMUsageEntry {
  env_id: string;
  agent_id: string;
  provider: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  timestamp: string;
}
```

### API Client (`ui-common/api/`)

```typescript
class BotGlueClient {
  constructor(baseUrl?: string); // defaults to window.location.origin

  projects: {
    list(): Promise<Project[]>;
    get(id: string): Promise<Project>;
    create(params: CreateProjectParams): Promise<Project>;
  };

  environments: {
    list(projectId: string): Promise<Environment[]>;
    create(params: CreateEnvParams): Promise<Environment>;
    fork(envId: string, newBranch: string): Promise<Environment>;
    pause(envId: string): Promise<void>;
    resume(envId: string): Promise<void>;
    destroy(envId: string): Promise<void>;
    exec(envId: string, command: string): Promise<ExecResult>;
  };

  agents: {
    list(envId?: string): Promise<Agent[]>;
    get(id: string): Promise<Agent>;
  };

  subscribe(handler: (event: AgentEvent) => void): () => void;
}
```

### Components (`ui-common/components/`)

Presentational React components, styled with Tailwind. Accept data via props, emit callbacks. No routing, no layout, no global state.

- `AttentionQueueItem` - agent needing attention (status badge, blocker text, action buttons)
- `AgentStatusBadge` - colored pill showing running/blocked/finished/error
- `DiffViewer` - file diff display (wrapping `react-diff-viewer` or similar)
- `EnvironmentCard` - environment status with branch, ports, resource usage, controls
- `ProjectCard` - project overview with active environment count
- `AuditLog` - scrollable list of audit entries

---

## 4. Daemon API Surface

Single HTTP server with three concerns:

### REST API

```
GET    /api/projects
POST   /api/projects
GET    /api/projects/:id

GET    /api/environments?project_id=
POST   /api/environments
POST   /api/environments/:id/fork
POST   /api/environments/:id/pause
POST   /api/environments/:id/resume
DELETE /api/environments/:id
POST   /api/environments/:id/exec     # { command: string } → { output, exit_code }

GET    /api/agents?env_id=
GET    /api/agents/:id

GET    /api/audit?env_id=&limit=
```

### WebSocket (`/api/ws`)

```typescript
type AgentEvent =
  | { type: "agent.blocked";  agent_id: string; blocker: string }
  | { type: "agent.finished"; agent_id: string; summary: string }
  | { type: "agent.error";    agent_id: string; error: string }
  | { type: "agent.progress"; agent_id: string; output_tail: string[] }
  | { type: "env.status";     env_id: string;   status: EnvironmentStatus }
```

### LLM Gateway (`/llm/v1/...`)

```
POST /llm/v1/messages    → proxied to Anthropic API (key injected)
POST /llm/v1/chat        → proxied to OpenAI-compatible API (key injected)
```

Containers reach the gateway via `http://host.containers.internal:<daemon_port>/llm/v1/...`.

### Static Files

Everything else serves `web/dist/` with SPA fallback (all routes → `index.html`).

---

## 5. Web App Pages

Five pages for MVP:

**Dashboard** (`/`)
- Attention queue at top - agents needing you, sorted by priority (blocked > error > finished)
- Active environments grouped by project
- Real-time updates via WebSocket

**Project Detail** (`/projects/:id`)
- Project settings (repo URL, default branch, notification prefs)
- Environment template config (port mappings, env vars, startup commands)
- List of environments for this project

**Environment Detail** (`/environments/:id`)
- Agent status and live output tail (last 50 lines, streaming)
- Port links (click to open `localhost:<host_port>` in new tab)
- Controls: pause, resume, destroy, fork
- Exec console - run arbitrary commands in the container

**Review** (`/environments/:id/review`)
- Diff viewer showing what the agent changed (git diff from branch point)
- Agent's summary of what it did
- Action buttons: approve, request changes, reject
- Inline comments (sent back to agent as feedback)

**Settings** (`/settings`)
- LLM provider keys (stored by daemon, never sent to containers)
- Default notification preferences
- Global port range config

---

## 6. Environment Templates & Configuration

```typescript
interface EnvironmentTemplate {
  name: string;
  repo_url: string;
  base_image?: string;              // default per stack
  ports: PortMapping[];             // pre-declared, bound at container start
  env_vars: Record<string, string>; // non-secret env vars
  setup_commands: string[];         // run once after clone: ["npm install"]
  startup_commands: string[];       // run on every resume: ["npm run dev"]
  agent_type: "claude" | "cursor" | "opencode" | "custom";
  llm_provider: string;            // which key to use from settings
}
```

**Flow:**
1. User creates a project, fills in the template
2. On "create environment", daemon creates Podman container with port bindings, `LLM_GATEWAY_URL` env var, clones repo, runs setup commands
3. User starts agent - daemon execs the agent CLI inside the container
4. Agent makes LLM calls through the gateway (no keys inside container)

**Port auto-assignment:** daemon picks from a configurable range (default `10000-11000`), tracks which are in use, frees on destroy.

---

## 7. Agent Lifecycle via Exec

No agent driver process. The daemon manages agents through `podman exec`.

**Starting:** `podman exec -d <container_id> claude --task "..."` with stdout/stderr redirected to a log file.

**Monitoring:** daemon streams the log file via a long-running exec, applies heuristics:

| Signal | Detected State |
|--------|---------------|
| Output flowing, commands running | `running` |
| Agent printed a question, idle for N seconds | `blocked` |
| Process exited with code 0 | `finished` |
| Process exited with non-zero code | `error` |
| Claude Code hook fires `agent.blocked` | `blocked` (preferred) |

**Sending input:** via exec writing to agent's stdin or using agent-specific mechanisms (Claude Code hooks).

**AgentDriver trait** (Rust side):

```rust
trait AgentDriver {
    fn start(env: &Environment, task: &str) -> Result<AgentHandle>;
    fn read_output(handle: &AgentHandle) -> Stream<String>;
    fn detect_state(output: &str) -> AgentState;
    fn send_input(handle: &AgentHandle, input: &str) -> Result<()>;
}
```

MVP implements `ClaudeCodeDriver` only. The trait makes it extensible later.

---

## 8. LLM Gateway

Reverse proxy inside the daemon. Three responsibilities:

**Key injection:** strips any auth from incoming request, adds the real key, forwards to provider.

**Provider routing:**
```
/llm/v1/messages    → Anthropic
/llm/v1/chat        → OpenAI-compatible (OpenAI, Groq, Ollama, etc.)
```

The container's env vars determine which endpoint the agent uses. The daemon knows which key to inject based on the path and environment's `llm_provider` config.

**Usage tracking:** logs every call with env_id, agent_id, provider, model, token counts. Gives cost visibility per project/agent.

**Security:**
- Gateway bound to localhost only
- Validates requests come from known container IPs
- API keys stored in daemon config, never in containers
- Container destruction immediately revokes access (no tokens to expire)

---

## 9. Open Questions

1. **Log streaming implementation** - `podman exec tail -f` vs Podman's log API vs mounting a shared volume for logs?
2. **Agent stdin interaction** - how reliable is writing to a detached process's stdin via exec? May need a PTY wrapper.
3. **Container networking** - `host.containers.internal` works on Podman? Need to verify cross-platform (Linux vs macOS).
4. **Browser notifications** - sufficient for MVP, or do we need a fallback (email, webhook)?
5. **Template sharing** - should users be able to export/import environment templates?
