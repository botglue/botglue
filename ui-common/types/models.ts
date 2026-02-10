export interface Project {
  id: string;
  name: string;
  repo_url: string;
  default_branch: string;
  notification_prefs: NotificationPrefs;
  project_type: "standard" | "incubator";
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
  idea_id: string | null;
  started_at: string;
  last_activity: string;
}

export interface Idea {
  id: string;
  project_id: string;
  title: string;
  description: string;
  status: "draft" | "active" | "completed" | "archived";
  created_at: string;
  updated_at: string;
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
