import type {
  Project,
  Environment,
  Agent,
  Idea,
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
    create: (data: {
      name: string;
      repo_url: string;
      default_branch: string;
      project_type?: string;
    }) =>
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
    exec: (id: string, command: string) =>
      request<{ output: string; exit_code: number }>(`/api/environments/${id}/exec`, {
        method: "POST",
        body: JSON.stringify({ command }),
      }),
  },

  ideas: {
    list: (projectId: string) =>
      request<Idea[]>(`/api/ideas?project_id=${projectId}`),
    get: (id: string) => request<Idea>(`/api/ideas/${id}`),
    create: (data: { project_id: string; title: string; description?: string }) =>
      request<Idea>("/api/ideas", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    update: (id: string, data: { title: string; description: string }) =>
      request<void>(`/api/ideas/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    updateStatus: (id: string, status: string) =>
      request<void>(`/api/ideas/${id}/status`, {
        method: "PUT",
        body: JSON.stringify({ status }),
      }),
    graduate: (id: string, data: { name: string; repo_url: string }) =>
      request<Project>(`/api/ideas/${id}/graduate`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    delete: (id: string) =>
      request<void>(`/api/ideas/${id}`, { method: "DELETE" }),
  },

  agents: {
    list: (envId?: string, ideaId?: string) => {
      const params = new URLSearchParams();
      if (envId) params.set("env_id", envId);
      if (ideaId) params.set("idea_id", ideaId);
      const qs = params.toString();
      return request<Agent[]>(qs ? `/api/agents?${qs}` : "/api/agents");
    },
    get: (id: string) => request<Agent>(`/api/agents/${id}`),
    create: (data: {
      env_id: string;
      agent_type: string;
      current_task: string;
      idea_id?: string;
    }) =>
      request<Agent>("/api/agents", {
        method: "POST",
        body: JSON.stringify({
          env_id: data.env_id,
          type: data.agent_type,
          current_task: data.current_task,
          idea_id: data.idea_id,
        }),
      }),
    updateStatus: (id: string, status: string, blocker?: string) =>
      request<void>(`/api/agents/${id}`, {
        method: "PATCH",
        body: JSON.stringify({ status, blocker }),
      }),
    delete: (id: string) =>
      request<void>(`/api/agents/${id}`, { method: "DELETE" }),
  },
};

export { ApiError };
