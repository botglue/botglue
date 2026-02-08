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
    <div>
      <h1>BotGlue</h1>
      <p>Project: {mockProject.name}</p>
      <p>Status: scaffolding complete</p>
    </div>
  );
}

export default App;
