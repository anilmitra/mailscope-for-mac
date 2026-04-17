import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import Layout from "./components/layout/Layout";
import Dashboard from "./pages/Dashboard";
import SeedMailboxes from "./pages/SeedMailboxes";
import NewTestRun from "./pages/NewTestRun";
import RunDetail from "./pages/RunDetail";
import Trends from "./pages/Trends";
import Settings from "./pages/Settings";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<Dashboard />} />
          <Route path="mailboxes" element={<SeedMailboxes />} />
          <Route path="runs/new" element={<NewTestRun />} />
          <Route path="runs/:id" element={<RunDetail />} />
          <Route path="trends" element={<Trends />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
