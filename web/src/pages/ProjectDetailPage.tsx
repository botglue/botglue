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
