import React, { useEffect } from "react";
import { useConfigStore } from "../../store/configStore";

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const config = useConfigStore((s) => s.config);

  useEffect(() => {
    if (!config) return;
    const theme =
      config.theme === "System"
        ? window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light"
        : config.theme.toLowerCase();
    document.documentElement.setAttribute("data-theme", theme);
  }, [config?.theme]);

  return <>{children}</>;
}
