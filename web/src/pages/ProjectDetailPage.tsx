import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router";
import type { Project, Environment, Agent, Idea } from "@botglue/common/types";
import { api } from "@botglue/common/api";
import { EnvironmentCard, AgentStatusBadge, IdeaStatusBadge } from "@botglue/common/components";
import { CreateEnvironmentForm } from "../components/CreateEnvironmentForm";
import { CreateIdeaForm } from "../components/CreateIdeaForm";

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [agents, setAgents] = useState<Map<string, Agent[]>>(new Map());
  const [ideas, setIdeas] = useState<Idea[]>([]);
  const [ideaAgentCounts, setIdeaAgentCounts] = useState<Map<string, number>>(new Map());
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

      const [envs, ideaList] = await Promise.all([
        api.environments.list(id!),
        api.ideas.list(id!),
      ]);
      setEnvironments(envs);
      setIdeas(ideaList);

      const agentMap = new Map<string, Agent[]>();
      await Promise.all(
        envs.map(async (env) => {
          const envAgents = await api.agents.list(env.id);
          agentMap.set(env.id, envAgents);
        })
      );
      setAgents(agentMap);

      // Count agents per idea
      const counts = new Map<string, number>();
      for (const idea of ideaList) {
        const ideaAgents = await api.agents.list(undefined, idea.id);
        counts.set(idea.id, ideaAgents.length);
      }
      setIdeaAgentCounts(counts);
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
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-semibold">{project.name}</h1>
            {project.project_type === "incubator" && (
              <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border bg-purple-500/20 text-purple-400 border-purple-500/30">
                incubator
              </span>
            )}
          </div>
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

      {/* Ideas */}
      <section className="mb-8">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide">
            Ideas
          </h2>
          <CreateIdeaForm projectId={project.id} onCreated={loadData} />
        </div>
        {ideas.length === 0 ? (
          <p className="text-[#6b6b7b] text-sm">No ideas yet.</p>
        ) : (
          <div className="space-y-2">
            {ideas.map((idea) => (
              <div
                key={idea.id}
                onClick={() => navigate(`/projects/${id}/ideas/${idea.id}`)}
                className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-3 cursor-pointer hover:border-[#2a2a4f] transition-colors"
              >
                <div className="flex items-center gap-3">
                  <IdeaStatusBadge status={idea.status} />
                  <span className="text-sm font-medium flex-1">{idea.title}</span>
                  {(ideaAgentCounts.get(idea.id) || 0) > 0 && (
                    <span className="text-xs text-[#6b6b7b]">
                      {ideaAgentCounts.get(idea.id)} agent{(ideaAgentCounts.get(idea.id) || 0) > 1 ? "s" : ""}
                    </span>
                  )}
                </div>
                {idea.description && (
                  <p className="text-xs text-[#6b6b7b] mt-1 truncate ml-[74px]">
                    {idea.description}
                  </p>
                )}
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Environments */}
      <section>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide">
            Environments
          </h2>
          <CreateEnvironmentForm projectId={project.id} onCreated={loadData} />
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
                    onClick={() => navigate(`/projects/${id}/environments/${env.id}`)}
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
