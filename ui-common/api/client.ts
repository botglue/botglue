import type {
  Project,
  Environment,
  Agent,
} from "../types";

class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
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
      container_id?: string;
      ports?: { name: string; container_port: number; host_port?: number; protocol?: string }[];
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
