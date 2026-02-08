import { useState, useEffect } from "react";
import type { Project } from "@botglue/common/types";
import { AgentStatusBadge } from "@botglue/common/components";

const mockProject: Project = {
  id: "1",
  name: "botglue",
  repo_url: "https://github.com/example/botglue",
  default_branch: "main",
  notification_prefs: {
    blocked: true,
    error: true,
    finished: true,
    progress: false,
  },
  created_at: new Date().toISOString(),
};

const statuses = ["running", "blocked", "finished", "error"] as const;

function App() {
  const [daemonStatus, setDaemonStatus] = useState<string>("checking...");

  useEffect(() => {
    fetch("/api/health")
      .then((r) => r.json())
      .then((data) => setDaemonStatus(`${data.status} (v${data.version})`))
      .catch(() => setDaemonStatus("not running"));
  }, []);

  return (
    <div className="min-h-screen bg-[#0a0a0f] text-[#f0f0f5] flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-semibold mb-4">BotGlue</h1>
        <p className="text-[#a0a0b0]">Project: {mockProject.name}</p>
        <div className="flex gap-2 mt-4 justify-center">
          {statuses.map((s) => (
            <AgentStatusBadge key={s} status={s} />
          ))}
        </div>
        <p className="text-[#6b6b7b] mt-4">Daemon: {daemonStatus}</p>
      </div>
    </div>
  );
}

export default App;
