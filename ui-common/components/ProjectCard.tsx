import type { Project } from "../types";

interface ProjectCardProps {
  project: Project;
  environmentCount: number;
  agentCount: number;
  onClick?: () => void;
}

export function ProjectCard({
  project,
  environmentCount,
  agentCount,
  onClick,
}: ProjectCardProps) {
  return (
    <div
      onClick={onClick}
      className={`rounded-lg border border-[#1a1a2f] bg-[#12121f] p-4 ${
        onClick ? "cursor-pointer hover:border-[#2a2a4f]" : ""
      }`}
    >
      <h3 className="text-lg font-medium mb-1">{project.name}</h3>
      <p className="text-sm text-[#6b6b7b] mb-3 truncate">{project.repo_url}</p>
      <div className="flex gap-4 text-sm text-[#a0a0b0]">
        <span>
          {environmentCount} {environmentCount === 1 ? "env" : "envs"}
        </span>
        <span>
          {agentCount} {agentCount === 1 ? "agent" : "agents"}
        </span>
      </div>
    </div>
  );
}
