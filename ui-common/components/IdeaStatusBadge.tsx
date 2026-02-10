import type { Idea } from "../types";

const statusColors: Record<Idea["status"], string> = {
  draft: "bg-[#333]/50 text-[#999] border-[#444]",
  active: "bg-green-500/20 text-green-400 border-green-500/30",
  completed: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  archived: "bg-[#333]/30 text-[#666] border-[#333]",
};

interface IdeaStatusBadgeProps {
  status: Idea["status"];
}

export function IdeaStatusBadge({ status }: IdeaStatusBadgeProps) {
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${statusColors[status]}`}
    >
      {status}
    </span>
  );
}
