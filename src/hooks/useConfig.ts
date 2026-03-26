import { useEffect } from "react";
import { useConfigStore } from "../store/configStore";

export function useConfig() {
  const { config, loading, load, update } = useConfigStore();

  useEffect(() => {
    if (!config) load();
  }, []);

  return { config, loading, update };
}
