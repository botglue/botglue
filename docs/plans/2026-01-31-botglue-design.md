# BotGlue Design Document

## 1. Overview & Goals

**BotGlue: Command Center for Agent-Assisted Development**

**Vision:** Enable technical builders to manage a portfolio of projects with AI agents, treating development like a VC treats investments - provide direction, unblock when stuck, review results.

**Primary Goals:**

1. **Reduce human bottleneck** - Agents should work autonomously within defined boundaries, only calling for human attention when genuinely needed.

2. **Enable parallel work** - Multiple agents working on multiple projects (or multiple approaches to the same project) simultaneously, without interference.

3. **Minimize context-switching cost** - When you do engage with an agent, getting up to speed should take seconds, not minutes.

**Non-Goals (for V1):**

- Team collaboration features
- Cloud hosting
- Custom agent development framework (we integrate with existing agents)
- Voice input (future roadmap)

**Success Metrics:**

- Time from "agent needs attention" to "agent unblocked" < 2 minutes
- Number of concurrent projects actively managed by single user: 3+
- Environment spin-up time: < 60 seconds
- User reports "I shipped more" in feedback

---

## 2. Architecture Overview

**Core Components:**

```
┌─────────────────────────────────────────────────────────┐
│                    BotGlue Desktop                      │
│         (Tauri app - MacOS/Win/Linux/Android/iOS)       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │  Attention  │  │   Project   │  │  Review         │  │
│  │  Queue UI   │  │   Manager   │  │  Interface      │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  BotGlue Daemon                         │
│              (Background service)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ Environment │  │   Agent     │  │  Notification   │  │
│  │ Manager     │  │   Monitor   │  │  Service        │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              Container Runtime (Podman)                 │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐           │
│  │  Env 1    │  │  Env 2    │  │  Env 3    │   ...     │
│  │  Agent A  │  │  Agent B  │  │  Agent C  │           │
│  │  Branch X │  │  Branch Y │  │  Branch Z │           │
│  └───────────┘  └───────────┘  └───────────┘           │
└─────────────────────────────────────────────────────────┘
```

**Three layers:**

1. **Desktop UI** - What you see. Attention queue, project list, review interface.
2. **Daemon** - Background service managing environments, monitoring agents, sending notifications.
3. **Container Runtime** - Isolated environments where agents actually run.

---

## 3. Environment Manager

**Purpose:** Create, manage, and destroy isolated development environments where agents work.

**Core Responsibilities:**

1. **Environment Creation**
   - Clone repo into isolated container
   - Set up branch (new or existing)
   - Install dependencies (detect package manager, run install)
   - Start dev servers if needed

2. **Environment Templates**
   - Save environment configs per project (ports, env vars, startup commands)
   - Replicate environments for parallel experiments
   - "Fork" an environment to try a different approach

3. **Resource Management**
   - Track CPU/memory per environment
   - Auto-pause idle environments
   - Hard limits to prevent runaway processes

**Environment Lifecycle:**

```
create → running → paused → destroyed
           ↑         │
           └─────────┘
          (on access)
```

**Key APIs:**

```
env.create(repo, branch, template?)  → env_id
env.fork(env_id, new_branch)         → new_env_id
env.pause(env_id)
env.resume(env_id)
env.destroy(env_id)
env.exec(env_id, command)            → output
env.logs(env_id)                     → stream
```

**Storage:**
- Each environment gets its own volume
- Persists across pause/resume
- Destroyed when environment destroyed (or optionally archived)

---

## 4. Agent Monitor

**Purpose:** Watch what agents are doing, detect when they need human attention, and track their state.

**Core Responsibilities:**

1. **Agent State Detection**
   - Running (actively working)
   - Blocked (waiting for human input)
   - Finished (task complete, awaiting review)
   - Error (hit a problem, needs guidance)

2. **Detection Methods:**
   - Parse agent CLI output for prompts/questions
   - Watch for idle patterns (no output for X seconds after question)
   - Hook into agent APIs where available (Claude Code hooks, etc.)
   - Monitor process state (running, exited, exit code)

3. **Context Capture**
   - Grab last N lines of agent output when state changes
   - Extract the question/blocker when agent is waiting
   - Summarize what agent was working on (for quick context)

**Agent Record:**

```
{
  agent_id: string
  env_id: string
  project: string
  status: "running" | "blocked" | "finished" | "error"
  started_at: timestamp
  last_activity: timestamp
  current_task: string (extracted/summarized)
  blocker: string | null (the question or error)
  output_tail: string[] (last 50 lines)
}
```

**Events Emitted:**

```
agent.blocked   → trigger notification
agent.finished  → trigger notification
agent.error     → trigger notification
agent.progress  → update UI (no notification)
```

---

## 5. Notification Service

**Purpose:** Alert you when agents need attention, without overwhelming you with noise.

**Core Responsibilities:**

1. **Notification Delivery**
   - Desktop push notifications (native OS)
   - Mobile push (via Tauri mobile app)
   - Sound/vibration options
   - Badge count on app icon

2. **Notification Types (priority order):**

   | Type | Urgency | Example |
   |------|---------|---------|
   | Blocked | High | "Agent needs answer: Which auth library?" |
   | Error | High | "Agent hit error in project-x" |
   | Finished | Medium | "Agent completed task in project-y" |
   | Confirmation | Medium | "Agent wants to deploy to staging" |
   | Progress | Low | "Agent 50% through migration" (optional) |

3. **Noise Reduction:**
   - Batch rapid-fire notifications (3+ in 10 seconds → "3 agents need attention")
   - Focus mode: queue notifications, deliver summary at intervals
   - Per-project notification preferences
   - Snooze project temporarily

4. **Attention Queue:**
   - All pending notifications in priority order
   - One-click jump to agent context
   - Mark as "will handle later" (removes from active queue, keeps in backlog)

**User Flow:**

```
Notification arrives → Tap/click → See agent context → Respond → Agent unblocked
                                                              ↓
                                        (average time: < 2 min)
```

---

## 6. Review Interface

**Purpose:** Make it fast to understand what an agent changed and provide feedback.

**Core Responsibilities:**

1. **Change Visualization**
   - File tree with change indicators (added/modified/deleted)
   - Side-by-side diff view
   - Inline diff view (toggle)
   - Syntax highlighting per language

2. **Navigation:**
   - Jump between changed files
   - Expand/collapse unchanged sections
   - Filter: show only additions / deletions / modifications
   - Future: voice navigation ("next file", "show only tests")

3. **Context Panel:**
   - Agent's summary of what it did
   - Commit messages (if agent committed)
   - Link to original task/prompt that started the work
   - Time spent, files touched, lines changed

4. **Feedback Actions:**

   | Action | Result |
   |--------|--------|
   | Approve | Merge to target branch, close task |
   | Approve + Deploy | Merge and trigger deploy pipeline |
   | Request Changes | Send feedback to agent, agent continues |
   | Reject | Discard changes, optionally restart with new direction |
   | Discuss | Open chat with agent about specific code |

5. **Inline Comments:**
   - Click on a line, leave a comment
   - Comments sent to agent as feedback
   - Agent can respond, iterate

**Voice Interaction (Future Roadmap):**

| Phase | Capability |
|-------|------------|
| V1 (no voice) | Click/keyboard navigation, typed feedback |
| V2 (voice nav) | "Next file", "show tests", "collapse this" |
| V3 (voice conversation) | "This class has too many responsibilities. Should we split it?" → Agent responds with analysis and proposal |

**Future: Voice Conversation**
> Point at code, ask questions, get answers. "Why did you use a factory pattern here?" "What happens if this throws?" "Try extracting this into a separate service." Review becomes dialogue.

---

## 7. Autonomous Operations

**Purpose:** Let agents perform infrastructure tasks (build, test, restart, deploy) without waiting for human button-clicks.

**Core Responsibilities:**

1. **Allowed Operations (per environment):**
   - Build/compile
   - Run tests
   - Restart dev server
   - Install dependencies
   - Deploy to staging (with optional confirmation gate)
   - Run arbitrary commands (within sandbox)

2. **Permission Model:**

   | Operation | Default | Configurable |
   |-----------|---------|--------------|
   | Build | Auto-approve | ✓ |
   | Test | Auto-approve | ✓ |
   | Restart dev | Auto-approve | ✓ |
   | Install deps | Notify | ✓ |
   | Deploy staging | Require confirm | ✓ |
   | Deploy prod | Always confirm | ✗ |
   | Arbitrary shell | Sandboxed auto | ✓ |

3. **Sandbox Boundaries:**
   - Network: only allowed endpoints (localhost, staging, approved APIs)
   - Filesystem: only within environment volume
   - Secrets: injected via BotGlue, not visible to agent as plaintext
   - Resource limits: CPU, memory, disk caps

4. **Audit Log:**
   - Every operation logged with timestamp, agent, command, output
   - Review what agent did after the fact
   - Revert capability (git-based, restore from commit)

**Agent Integration:**

```
# Agent calls BotGlue API instead of running directly
botglue exec "npm run build"    → runs in sandbox, returns output
botglue deploy staging          → triggers confirmation if required
botglue restart                 → restarts dev server, returns new PID
```

---

## 8. Data Model

**Purpose:** Define how BotGlue stores projects, environments, agents, and state.

**Core Entities:**

```
Project
├── id: string
├── name: string
├── repo_url: string
├── default_branch: string
├── environment_template: EnvironmentTemplate
├── notification_prefs: NotificationPrefs
└── created_at: timestamp

Environment
├── id: string
├── project_id: string
├── branch: string
├── status: "creating" | "running" | "paused" | "destroyed"
├── container_id: string
├── ports: { internal: external }
├── created_at: timestamp
└── last_active: timestamp

Agent
├── id: string
├── env_id: string
├── type: "claude" | "cursor" | "opencode" | "custom"
├── status: "running" | "blocked" | "finished" | "error"
├── current_task: string
├── blocker: string | null
├── started_at: timestamp
└── last_activity: timestamp

AuditEntry
├── id: string
├── env_id: string
├── agent_id: string
├── operation: string
├── command: string
├── output: string
├── exit_code: int
└── timestamp: timestamp
```

**Storage:**
- SQLite for metadata (ships with app, zero config)
- Volumes for environment filesystems (managed by Podman)
- Logs in append-only files, rotated daily

---

## 9. Tech Stack

**Frontend (Tauri app):**
- **Framework:** SvelteKit or SolidJS (lightweight, fast)
- **Styling:** Tailwind CSS
- **State:** Built-in reactivity (no Redux overhead)
- **Diff rendering:** Monaco editor (same as VS Code) or custom

**Backend (Daemon):**
- **Language:** Rust (same as Tauri, single toolchain)
- **API:** Local HTTP + WebSocket for real-time updates
- **Database:** SQLite via rusqlite
- **Process management:** tokio for async

**Container Runtime:**
- **Primary:** Podman (rootless, daemonless)
- **Fallback:** Docker if Podman unavailable
- **Image base:** Customizable, sensible defaults per stack (Node, Python, Go, etc.)

**Agent Integration:**
- **Claude Code:** Hook into CLI events, parse output
- **Cursor/Windsurf:** Monitor process, parse terminal output
- **OpenCode:** Same as Claude Code
- **Custom:** Plugin API for other agents

**Mobile (Tauri mobile):**
- Shared Rust core
- Native UI shell per platform
- Push notifications via OS-native channels

---

## 10. Implementation Roadmap

**Phase 1: Foundation (MVP)**
- Single environment creation (Podman)
- Single agent monitoring (Claude Code first)
- Basic attention queue (blocked/finished notifications)
- Desktop app shell (Tauri + basic UI)
- Local SQLite storage

**Deliverable:** Can run one agent in isolated environment, get notified when it needs you.

---

**Phase 2: Multi-Project**
- Multiple environments simultaneously
- Multiple agents tracked in parallel
- Project management UI
- Environment templates (save/reuse configs)
- Notification batching and focus mode

**Deliverable:** Manage 3+ projects with agents, attention queue works at scale.

---

**Phase 3: Review & Feedback**
- Diff visualization interface
- Inline comments → agent feedback loop
- Approve/reject/request changes flow
- Audit log viewer

**Deliverable:** Full review workflow without leaving BotGlue.

---

**Phase 4: Autonomous Operations**
- Permission model for build/test/deploy
- Sandbox boundaries enforced
- Secrets management
- Deploy to staging with confirmation gate

**Deliverable:** Agents self-serve on infra within boundaries.

---

**Phase 5: Mobile + Polish**
- Mobile app (iOS/Android)
- Push notifications
- Voice navigation (V2)
- Voice conversation (V3)

**Deliverable:** Review and unblock agents from anywhere.

---

## 11. Open Questions

Capturing decisions to revisit as we build:

1. **Agent detection method** - Parsing CLI output is fragile. Should we prioritize agents with proper hook APIs (Claude Code) or build generic detection?

2. **Environment templates** - Ship default templates per stack, or require user to configure from scratch?

3. **Mobile scope** - Full review on mobile, or just notifications + quick unblock?

4. **Multi-machine** - If user has agents running on home server, how does mobile connect? VPN? Relay server?

5. **Team features timing** - When does "Pro tier with teams" make sense to build? After 100 users? 1000?

6. **Voice provider** - Build on Whisper? Partner with Wispr/Willow? Wait for better local models?

---

## Competitive Landscape

**Agent Orchestration Frameworks:**
- LangGraph, CrewAI, Semantic Kernel - code-first SDKs for agent workflows
- Google Antigravity (2025) - "mission control for multi-agent workflows" - enterprise-focused
- These are building blocks, not end-user experience

**Voice Coding Tools:**
- Wispr Flow - learns technical vocab, 2x typing speed
- Willow Voice - context-aware, learns patterns
- Superwhisper - local/private, on-device
- None integrated with agent orchestration

**Sandbox/Isolation:**
- Daytona - 90ms environment creation, enterprise
- Container Use - early stage, "multiple agents without interference"
- Runloop - enterprise devbox, 10K+ parallel instances
- Agent Sandbox - Kubernetes-based, Google-backed

**Attention Queue for Agent Supervision:**
- Gap in market. No product specifically for "supervise multiple AI agents, get notified when they need you."

**BotGlue Differentiation:**
- Self-hosted (vs. cloud-hosted, pay-per-use)
- Solo dev / indie hacker focused (vs. enterprise)
- Integrated experience (vs. separate tools for voice, sandbox, orchestration)
- "Portfolio manager" mental model (vs. "orchestration framework")
