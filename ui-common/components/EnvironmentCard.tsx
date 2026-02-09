import type { Environment } from "../types";

const statusColors: Record<Environment["status"], string> = {
  creating: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  running: "bg-green-500/20 text-green-400 border-green-500/30",
  paused: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
  destroyed: "bg-[#333]/50 text-[#666] border-[#333]",
};

interface EnvironmentCardProps {
  environment: Environment;
  agentCount: number;
  onClick?: () => void;
}

export function EnvironmentCard({
  environment,
  agentCount,
  onClick,
}: EnvironmentCardProps) {
  return (
    <div
      onClick={onClick}
      className={`rounded-lg border border-[#1a1a2f] bg-[#0e0e1a] p-3 ${
        onClick ? "cursor-pointer hover:border-[#2a2a4f]" : ""
      }`}
    >
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-mono">{environment.branch}</span>
        <span
          className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border ${statusColors[environment.status]}`}
        >
          {environment.status}
        </span>
      </div>
      <div className="flex gap-4 text-xs text-[#6b6b7b]">
        <span>{agentCount} {agentCount === 1 ? "agent" : "agents"}</span>
        {environment.ports.length > 0 && (
          <span>{environment.ports.length} {environment.ports.length === 1 ? "port" : "ports"}</span>
        )}
      </div>
    </div>
  );
}
