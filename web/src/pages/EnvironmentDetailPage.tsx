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
                  <span className="text-sm font-medium">{agent.type}</span>
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
