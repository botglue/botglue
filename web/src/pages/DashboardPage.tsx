import { useState, useEffect } from "react";
import { useNavigate } from "react-router";
import type { Project, Environment, Agent } from "@botglue/common/types";
import { api } from "@botglue/common/api";
import {
  ProjectCard,
  EnvironmentCard,
  AgentStatusBadge,
} from "@botglue/common/components";
import { CreateProjectForm } from "../components/CreateProjectForm";

interface ProjectData {
  project: Project;
  environments: Environment[];
  agents: Agent[];
}

export function DashboardPage() {
  const [projects, setProjects] = useState<ProjectData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      setLoading(true);
      setError(null);
      const projectList = await api.projects.list();

      const data = await Promise.all(
        projectList.map(async (project) => {
          const environments = await api.environments.list(project.id);
          const agentLists = await Promise.all(
            environments.map((env) => api.agents.list(env.id))
          );
          const agents = agentLists.flat();
          return { project, environments, agents };
        })
      );

      setProjects(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load data");
    } finally {
      setLoading(false);
    }
  }

  const attentionAgents = projects
    .flatMap((p) =>
      p.agents
        .filter((a) => a.status === "blocked" || a.status === "error" || a.status === "finished")
        .map((a) => ({
          agent: a,
          project: p.project,
          environment: p.environments.find((e) => e.id === a.env_id),
        }))
    )
    .sort((a, b) => {
      const priority: Record<string, number> = { blocked: 0, error: 1, finished: 2 };
      return (priority[a.agent.status] ?? 3) - (priority[b.agent.status] ?? 3);
    });

  if (loading) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Dashboard</h1>
        <p className="text-[#6b6b7b]">Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Dashboard</h1>
        <p className="text-red-400">{error}</p>
        <button
          onClick={loadData}
          className="mt-2 text-sm text-[#a0a0b0] hover:text-[#f0f0f5] underline"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-semibold">Dashboard</h1>
        <CreateProjectForm onCreated={loadData} />
      </div>

      {/* Attention Queue */}
      {attentionAgents.length > 0 && (
        <section className="mb-8">
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide mb-3">
            Needs Attention
          </h2>
          <div className="space-y-2">
            {attentionAgents.map(({ agent, project, environment }) => (
              <div
                key={agent.id}
                className="flex items-center gap-3 rounded-lg border border-[#1a1a2f] bg-[#12121f] p-3"
              >
                <AgentStatusBadge status={agent.status} />
                <div className="flex-1 min-w-0">
                  <span className="text-sm font-medium">{project.name}</span>
                  {environment && (
                    <span className="text-[#6b6b7b] text-sm"> / {environment.branch}</span>
                  )}
                  <p className="text-xs text-[#6b6b7b] truncate">
                    {agent.blocker || agent.current_task}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Projects */}
      {projects.length === 0 ? (
        <p className="text-[#6b6b7b]">No projects yet. Create one to get started.</p>
      ) : (
        <section>
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide mb-3">
            Projects
          </h2>
          <div className="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
            {projects.map(({ project, environments, agents }) => (
              <div key={project.id}>
                <ProjectCard
                  project={project}
                  environmentCount={environments.length}
                  agentCount={agents.length}
                  onClick={() => navigate(`/projects/${project.id}`)}
                />
                {environments.length > 0 && (
                  <div className="mt-2 ml-2 space-y-1">
                    {environments.map((env) => (
                      <EnvironmentCard
                        key={env.id}
                        environment={env}
                        agentCount={agents.filter((a) => a.env_id === env.id).length}
                        onClick={() => navigate(`/projects/${project.id}/environments/${env.id}`)}
                      />
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
