import React, { useEffect } from "react";
import { Routes, Route, Navigate, useNavigate } from "react-router-dom";
import { ThemeProvider } from "./components/shared/ThemeProvider";
import { Wizard } from "./pages/Wizard";
import { Main } from "./pages/Main";
import { Settings } from "./pages/Settings";
import { useConfigStore } from "./store/configStore";

export default function App() {
  const { config, loading, load } = useConfigStore();
  const navigate = useNavigate();

  useEffect(() => {
    load();
  }, []);

  useEffect(() => {
    if (!loading && config) {
      if (!config.first_run_complete) {
        navigate("/wizard");
      } else {
        navigate("/main");
      }
    }
  }, [loading, config]);

  if (loading) {
    return (
      <div
        className="flex items-center justify-center h-screen"
        style={{ background: "var(--bg)" }}
        data-theme="dark"
      >
        <div className="w-2 h-2 rounded-full bg-[var(--accent)] animate-pulse" />
      </div>
    );
  }

  return (
    <ThemeProvider>
      <Routes>
        <Route path="/wizard" element={<Wizard />} />
        <Route path="/main" element={<Main />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="*" element={<Navigate to="/main" replace />} />
      </Routes>
    </ThemeProvider>
  );
}
