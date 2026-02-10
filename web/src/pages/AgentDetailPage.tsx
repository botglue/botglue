import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router";
import type { Agent, Environment, Idea } from "@botglue/common/types";
import { api } from "@botglue/common/api";
import { AgentStatusBadge, IdeaStatusBadge } from "@botglue/common/components";

export function AgentDetailPage() {
  const { projectId, agentId } = useParams<{
    projectId: string;
    agentId: string;
  }>();
  const navigate = useNavigate();
  const [agent, setAgent] = useState<Agent | null>(null);
  const [environment, setEnvironment] = useState<Environment | null>(null);
  const [idea, setIdea] = useState<Idea | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState(false);
  const [blockerInput, setBlockerInput] = useState("");

  useEffect(() => {
    if (agentId) loadData();
  }, [agentId]);

  async function loadData() {
    try {
      setLoading(true);
      setError(null);
      const a = await api.agents.get(agentId!);
      setAgent(a);

      const env = await api.environments.get(a.env_id);
      setEnvironment(env);

      if (a.idea_id) {
        try {
          const i = await api.ideas.get(a.idea_id);
          setIdea(i);
        } catch {
          // idea may have been deleted
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load agent");
    } finally {
      setLoading(false);
    }
  }

  async function handleStatusChange(status: string, blocker?: string) {
    setActionLoading(true);
    try {
      await api.agents.updateStatus(agentId!, status, blocker);
      await loadData();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to update status");
    } finally {
      setActionLoading(false);
    }
  }

  if (loading) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Agent</h1>
        <p className="text-[#6b6b7b]">Loading...</p>
      </div>
    );
  }

  if (error || !agent) {
    return (
      <div>
        <h1 className="text-2xl font-semibold mb-4">Agent</h1>
        <p className="text-red-400">{error || "Agent not found"}</p>
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
          {environment && (
            <button
              onClick={() =>
                navigate(`/projects/${projectId}/environments/${environment.id}`)
              }
              className="text-sm text-[#6b6b7b] hover:text-[#a0a0b0] mb-2"
            >
              &larr; {environment.branch}
            </button>
          )}
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-semibold">{agent.type}</h1>
            <AgentStatusBadge status={agent.status} />
          </div>
        </div>
      </div>

      {error && <p className="text-red-400 text-sm mb-4">{error}</p>}

      {/* Agent Info */}
      <section className="mb-8 rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4">
        <div className="grid gap-3 text-sm">
          <div>
            <span className="text-[#6b6b7b]">Current task:</span>
            <p className="mt-1">{agent.current_task || "No task set"}</p>
          </div>

          {agent.blocker && (
            <div className="rounded-md bg-yellow-500/10 border border-yellow-500/20 p-3">
              <span className="text-yellow-400 text-xs font-medium uppercase tracking-wide">
                Blocker
              </span>
              <p className="mt-1 text-yellow-200">{agent.blocker}</p>
            </div>
          )}

          {idea && (
            <div>
              <span className="text-[#6b6b7b]">Idea:</span>{" "}
              <button
                onClick={() =>
                  navigate(`/projects/${projectId}/ideas/${idea.id}`)
                }
                className="text-blue-400 hover:text-blue-300 underline"
              >
                {idea.title}
              </button>
              <span className="ml-2">
                <IdeaStatusBadge status={idea.status} />
              </span>
            </div>
          )}

          {environment && (
            <div>
              <span className="text-[#6b6b7b]">Environment:</span>{" "}
              <button
                onClick={() =>
                  navigate(
                    `/projects/${projectId}/environments/${environment.id}`
                  )
                }
                className="text-blue-400 hover:text-blue-300 underline"
              >
                {environment.branch}
              </button>
            </div>
          )}

          <div>
            <span className="text-[#6b6b7b]">Started:</span>{" "}
            <span>{new Date(agent.started_at).toLocaleString()}</span>
          </div>
          <div>
            <span className="text-[#6b6b7b]">Last activity:</span>{" "}
            <span>{new Date(agent.last_activity).toLocaleString()}</span>
          </div>
        </div>
      </section>

      {/* Status Actions */}
      <section>
        <h2 className="text-sm font-medium text-[#a0a0b0] uppercase tracking-wide mb-3">
          Actions
        </h2>
        <div className="flex flex-wrap items-end gap-3">
          {agent.status !== "running" && (
            <button
              onClick={() => handleStatusChange("running")}
              disabled={actionLoading}
              className="text-sm text-green-400/70 hover:text-green-400 border border-green-400/30 hover:border-green-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Mark Running
            </button>
          )}
          {agent.status !== "finished" && (
            <button
              onClick={() => handleStatusChange("finished")}
              disabled={actionLoading}
              className="text-sm text-blue-400/70 hover:text-blue-400 border border-blue-400/30 hover:border-blue-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Mark Finished
            </button>
          )}
          {agent.status !== "error" && (
            <button
              onClick={() => handleStatusChange("error")}
              disabled={actionLoading}
              className="text-sm text-red-400/70 hover:text-red-400 border border-red-400/30 hover:border-red-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Mark Error
            </button>
          )}
          <div className="flex items-end gap-2">
            <input
              type="text"
              placeholder="Blocker reason..."
              value={blockerInput}
              onChange={(e) => setBlockerInput(e.target.value)}
              className="bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1 text-sm focus:outline-none focus:border-[#4a4a6f]"
            />
            <button
              onClick={() => {
                handleStatusChange("blocked", blockerInput || undefined);
                setBlockerInput("");
              }}
              disabled={actionLoading}
              className="text-sm text-yellow-400/70 hover:text-yellow-400 border border-yellow-400/30 hover:border-yellow-400/50 rounded px-3 py-1 disabled:opacity-50"
            >
              Mark Blocked
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}
