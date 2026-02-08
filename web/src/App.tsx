import type { Project } from "@botglue/common/types";

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

function App() {
  return (
    <div className="min-h-screen bg-[#0a0a0f] text-[#f0f0f5] flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-semibold mb-4">BotGlue</h1>
        <p className="text-[#a0a0b0]">Project: {mockProject.name}</p>
        <p className="text-[#6b6b7b] mt-2">Scaffolding complete</p>
      </div>
    </div>
  );
}

export default App;
