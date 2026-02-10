import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router";
import type { Idea, Agent, Environment, Project } from "@botglue/common/types";
import { api } from "@botglue/common/api";
import { AgentStatusBadge, IdeaStatusBadge } from "@botglue/common/components";

const AGENT_TYPES = ["claude", "cursor", "opencode", "custom"];

export function IdeaDetailPage() {
  const { projectId, ideaId } = useParams<{
    projectId: string;
    ideaId: string;
  }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [idea, setIdea] = useState<Idea | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState(false);

  // Assign agent form
  const [showAssignForm, setShowAssignForm] = useState(false);
  const [assignEnvId, setAssignEnvId] = useState("");
  const [assignType, setAssignType] = useState("claude");
  const [assignTask, setAssignTask] = useState("");
  const [assignLoading, setAssignLoading] = useState(false);

  // Graduate form
  const [showGraduateForm, setShowGraduateForm] = useState(false);
  const [gradName, setGradName] = useState("");
  const [gradRepoUrl, setGradRepoUrl] = useState("");
  const [gradLoading, setGradLoading] = useState(false);

  useEffect(() => {
    if (projectId && ideaId) loadData();
  }, [projectId, ideaId]);

  async function loadData() {
    try {
      setLoading(true);
      setError(null);
      const [proj, i, agentList, envList] = await Promise.all([
        api.projects.get(projectId!),
        api.ideas.get(ideaId!),
        api.agents.list(undefined, ideaId!),
        api.environments.list(projectId!),
      ]);
      setProject(proj);
      setIdea(i);
      setAgents(agentList);
      setEnvironments(envList);
      if (envList.length > 0 && !assignEnvId) {
        setAssignEnvId(envList[0].id);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load idea");
    } finally {
      setLoading(false);
    }
  }

  async function handleStatusChange(status: string) {
    setActionLoading(true);
    try {
      await api.ideas.updateStatus(ideaId!, status);
      await loadData();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to update status");
    } finally {
      setActionLoading(false);
    }
  }

  async function handleAssignAgent(e: React.FormEvent) {
    e.preventDefault();
    setAssignLoading(true);
    try {
      await api.agents.create({
        env_id: assignEnvId,
        agent_type: assignType,
        current_task: assignTask,
        idea_id: ideaId!,
      });
      setAssignTask("");
      setShowAssignForm(false);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to assign agent");
    } finally {
      setAssignLoading(false);
    }
  }

  async function handleGraduate(e: React.FormEvent) {
    e.preventDefault();
    setGradLoading(true);
    try {
      const newProject = await api.ideas.graduate(ideaId!, {
        name: gradName,
        repo_url: gradRepoUrl,
      });
      navigate(`/projects/${newProject.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to graduate idea");
    } finally {
      setGradLoading(false);
    }
  }

  if (loading) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Idea</h1>
        <p className="text-[#6b6b7b]">Loading...</p>
      </div>
    );
  }

  if (error || !idea) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Idea</h1>
        <p className="text-red-400">{error || "Idea not found"}</p>
        <button
          onClick={() => navigate(`/projects/${projectId}`)}
          className="mt-2 text-sm text-[#a0a0b0] hover:text-[#f0f0f5] underline"
        >
          Back to Project
        </button>
      </div>
    );
  }

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
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-semibold">{idea.title}</h1>
            <IdeaStatusBadge status={idea.status} />
          </div>
        </div>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      {/* Idea Info */}
      <section className="mb-8 rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4">
        <div className="grid gap-2 text-sm">
          {idea.description && (
            <div>
              <span className="text-[#6b6b7b]">Description:</span>
              <p className="mt-1">{idea.description}</p>
            </div>
          )}
          <div>
            <span className="text-[#6b6b7b]">Created:</span>{" "}
            <span>{new Date(idea.created_at).toLocaleString()}</span>
          </div>
          <div>
            <span className="text-[#6b6b7b]">Updated:</span>{" "}
            <span>{new Date(idea.updated_at).toLocaleString()}</span>
          </div>
        </div>
      </section>

      {/* Status Actions */}
      <section className="mb-8">
        <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide mb-3">
          Status Actions
        </h2>
        <div className="flex flex-wrap gap-2">
          {idea.status === "draft" && (
            <button
              onClick={() => handleStatusChange("active")}
              disabled={actionLoading}
              className="text-sm text-green-400/70 hover:text-green-400 border border-green-400/30 hover:border-green-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Start Working
            </button>
          )}
          {idea.status === "active" && (
            <button
              onClick={() => handleStatusChange("completed")}
              disabled={actionLoading}
              className="text-sm text-blue-400/70 hover:text-blue-400 border border-blue-400/30 hover:border-blue-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Mark Complete
            </button>
          )}
          {idea.status !== "archived" && (
            <button
              onClick={() => handleStatusChange("archived")}
              disabled={actionLoading}
              className="text-sm text-[#6b6b7b] hover:text-[#a0a0b0] border border-[#2a2a4f] hover:border-[#3a3a5f] rounded px-3 py-1 disabled:opacity-50"
            >
              Archive
            </button>
          )}
        </div>
      </section>

      {/* Agents */}
      <section className="mb-8">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide">
            Agents
          </h2>
          {!showAssignForm && (
            <button
              onClick={() => setShowAssignForm(true)}
              className="text-sm text-[#a0a0b0] hover:text-[#f0f0f5] border border-dashed border-[#2a2a4f] rounded-lg px-4 py-2"
            >
              + Assign Agent
            </button>
          )}
        </div>

        {showAssignForm && (
          <form
            onSubmit={handleAssignAgent}
            className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 space-y-3 mb-4"
          >
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium">Assign Agent</h3>
              <button
                type="button"
                onClick={() => setShowAssignForm(false)}
                className="text-[#6b6b7b] hover:text-[#f0f0f5] text-sm"
              >
                Cancel
              </button>
            </div>
            <select
              value={assignEnvId}
              onChange={(e) => setAssignEnvId(e.target.value)}
              className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
            >
              {environments.length === 0 && (
                <option value="">No environments available</option>
              )}
              {environments.map((env) => (
                <option key={env.id} value={env.id}>
                  {env.branch} ({env.status})
                </option>
              ))}
            </select>
            <select
              value={assignType}
              onChange={(e) => setAssignType(e.target.value)}
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
              placeholder="Task description"
              value={assignTask}
              onChange={(e) => setAssignTask(e.target.value)}
              required
              className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
            />
            <button
              type="submit"
              disabled={assignLoading || !assignTask || !assignEnvId}
              className="bg-[#2a2a4f] hover:bg-[#3a3a5f] disabled:opacity-50 disabled:cursor-not-allowed text-sm px-4 py-1.5 rounded"
            >
              {assignLoading ? "Assigning..." : "Assign"}
            </button>
          </form>
        )}

        {agents.length === 0 ? (
          <p className="text-[#6b6b7b] text-sm">No agents assigned to this idea.</p>
        ) : (
          <div className="space-y-2">
            {agents.map((agent) => (
              <div
                key={agent.id}
                onClick={() =>
                  navigate(`/projects/${projectId}/agents/${agent.id}`)
                }
                className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-3 flex items-center gap-3 cursor-pointer hover:border-[#2a2a4f] transition-colors"
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

      {/* Graduate (only for incubator projects) */}
      {project?.project_type === "incubator" &&
        idea.status !== "archived" && (
          <section>
            <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide mb-3">
              Graduate to Project
            </h2>
            {!showGraduateForm ? (
              <button
                onClick={() => setShowGraduateForm(true)}
                className="text-sm text-purple-400/70 hover:text-purple-400 border border-purple-400/30 hover:border-purple-400/50 rounded px-3 py-1"
              >
                Graduate Idea
              </button>
            ) : (
              <form
                onSubmit={handleGraduate}
                className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 space-y-3"
              >
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-medium">
                    Create new project from this idea
                  </h3>
                  <button
                    type="button"
                    onClick={() => setShowGraduateForm(false)}
                    className="text-[#6b6b7b] hover:text-[#f0f0f5] text-sm"
                  >
                    Cancel
                  </button>
                </div>
                <input
                  type="text"
                  placeholder="Project name"
                  value={gradName}
                  onChange={(e) => setGradName(e.target.value)}
                  required
                  className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
                />
                <input
                  type="text"
                  placeholder="Repository URL"
                  value={gradRepoUrl}
                  onChange={(e) => setGradRepoUrl(e.target.value)}
                  required
                  className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
                />
                <button
                  type="submit"
                  disabled={gradLoading || !gradName || !gradRepoUrl}
                  className="bg-purple-500/20 hover:bg-purple-500/30 text-purple-400 border border-purple-500/30 disabled:opacity-50 disabled:cursor-not-allowed text-sm px-4 py-1.5 rounded"
                >
                  {gradLoading ? "Graduating..." : "Graduate"}
                </button>
              </form>
            )}
          </section>
        )}
    </div>
  );
}
