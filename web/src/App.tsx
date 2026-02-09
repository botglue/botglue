import { Routes, Route } from "react-router";
import { AppLayout } from "./layouts/AppLayout";
import { DashboardPage } from "./pages/DashboardPage";
import { ProjectDetailPage } from "./pages/ProjectDetailPage";
import { EnvironmentDetailPage } from "./pages/EnvironmentDetailPage";

function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<DashboardPage />} />
        <Route path="projects/:id" element={<ProjectDetailPage />} />
        <Route path="projects/:projectId/environments/:envId" element={<EnvironmentDetailPage />} />
      </Route>
    </Routes>
  );
}

export default App;
