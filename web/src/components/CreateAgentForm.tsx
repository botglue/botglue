import { useState } from "react";
import { api } from "@botglue/common/api";

interface CreateAgentFormProps {
  envId: string;
  onCreated: () => void;
}

const AGENT_TYPES = ["claude", "cursor", "opencode", "custom"];

export function CreateAgentForm({ envId, onCreated }: CreateAgentFormProps) {
  const [open, setOpen] = useState(false);
  const [agentType, setAgentType] = useState("claude");
  const [currentTask, setCurrentTask] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.agents.create({
        env_id: envId,
        agent_type: agentType,
        current_task: currentTask,
      });
      setCurrentTask("");
      setAgentType("claude");
      setOpen(false);
      onCreated();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create agent");
    } finally {
      setSubmitting(false);
    }
  }

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        className="text-sm text-[#a0a0b0] hover:text-[#f0f0f5] border border-dashed border-[#2a2a4f] rounded-lg px-4 py-2"
      >
        + New Agent
      </button>
    );
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 space-y-3"
    >
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">New Agent</h3>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="text-[#6b6b7b] hover:text-[#f0f0f5] text-sm"
        >
          Cancel
        </button>
      </div>
      <select
        value={agentType}
        onChange={(e) => setAgentType(e.target.value)}
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
        placeholder="Task description (e.g. Implement login page)"
        value={currentTask}
        onChange={(e) => setCurrentTask(e.target.value)}
        required
        className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
      />
      {error && <p className="text-red-400 text-xs">{error}</p>}
      <button
        type="submit"
        disabled={submitting || !currentTask}
        className="bg-[#2a2a4f] hover:bg-[#3a3a5f] disabled:opacity-50 disabled:cursor-not-allowed text-sm px-4 py-1.5 rounded"
      >
        {submitting ? "Creating..." : "Create"}
      </button>
    </form>
  );
}
