import { useParams } from "react-router";

export function EnvironmentDetailPage() {
  const { projectId, envId } = useParams<{ projectId: string; envId: string }>();
  return (
    <div>
      <h1 className="text-2xl font-semibold mb-4">Environment {envId}</h1>
      <p className="text-[#6b6b7b]">Project: {projectId}</p>
    </div>
  );
}
