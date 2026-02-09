import { useState } from "react";
import { api } from "@botglue/common/api";

interface CreateEnvironmentFormProps {
  projectId: string;
  onCreated: () => void;
}

export function CreateEnvironmentForm({
  projectId,
  onCreated,
}: CreateEnvironmentFormProps) {
  const [open, setOpen] = useState(false);
  const [branch, setBranch] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await api.environments.create({
        project_id: projectId,
        branch,
      });
      setBranch("");
      setOpen(false);
      onCreated();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to create environment"
      );
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
        + New Environment
      </button>
    );
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 space-y-3"
    >
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">New Environment</h3>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="text-[#6b6b7b] hover:text-[#f0f0f5] text-sm"
        >
          Cancel
        </button>
      </div>
      <input
        type="text"
        placeholder="Branch name (e.g. feature/login)"
        value={branch}
        onChange={(e) => setBranch(e.target.value)}
        required
        className="w-full bg-[#0a0a0f] border border-[#2a2a4f] rounded px-3 py-1.5 text-sm focus:outline-none focus:border-[#4a4a6f]"
      />
      {error && <p className="text-red-400 text-xs">{error}</p>}
      <button
        type="submit"
        disabled={submitting || !branch}
        className="bg-[#2a2a4f] hover:bg-[#3a3a5f] disabled:opacity-50 disabled:cursor-not-allowed text-sm px-4 py-1.5 rounded"
      >
        {submitting ? "Creating..." : "Create"}
      </button>
    </form>
  );
}
