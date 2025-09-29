import { useCallback, useEffect, useState } from "react";
import { SetupConfig, useTauri } from "./useTauri";

export const useSetupConfig = () => {
  const api = useTauri();
  const [config, setConfig] = useState<SetupConfig | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const current = await api.loadSetupConfig();
      setConfig(current);
      return current;
    } finally {
      setLoading(false);
    }
  }, [api]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const setAdbFromFile = useCallback(
    async (path: string) => {
      const updated = await api.setAdbKeyFromFile(path);
      setConfig(updated);
      return updated;
    },
    [api]
  );

  const setAdbFromBytes = useCallback(
    async (filename: string, data: number[]) => {
      const updated = await api.setAdbKeyFromBytes(filename, data);
      setConfig(updated);
      return updated;
    },
    [api]
  );

  const setAdbRemote = useCallback(
    async (url: string) => {
      const updated = await api.setAdbKeyRemote(url);
      setConfig(updated);
      return updated;
    },
    [api]
  );

  const clearAdb = useCallback(async () => {
    const updated = await api.clearAdbKey();
    setConfig(updated);
    return updated;
  }, [api]);

  const setGithubToken = useCallback(
    async (token?: string | null) => {
      const updated = await api.setGithubToken(token ?? null);
      setConfig(updated);
      return updated;
    },
    [api]
  );

  return {
    config,
    loading,
    refresh,
    setAdbFromFile,
    setAdbFromBytes,
    setAdbRemote,
    clearAdb,
    setGithubToken,
  };
};
