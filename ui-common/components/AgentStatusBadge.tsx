import type { Agent } from "../types";

const statusColors: Record<Agent["status"], string> = {
  running: "bg-green-500/20 text-green-400 border-green-500/30",
  blocked: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
  finished: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  error: "bg-red-500/20 text-red-400 border-red-500/30",
};

interface AgentStatusBadgeProps {
  status: Agent["status"];
}

export function AgentStatusBadge({ status }: AgentStatusBadgeProps) {
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${statusColors[status]}`}
    >
      {status}
    </span>
  );
}
