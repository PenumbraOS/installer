import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

export interface DeviceInfo {
  connected: boolean;
  error_message?: string;
}

export interface PackageInfo {
  package_name: string;
  version?: string;
}

export interface RepositoryInfo {
  name: string;
  owner: string;
  repo: string;
  description?: string;
}

export type AdbSource =
  | {
      type: "local_copy";
      stored_path: string;
      original_filename: string;
    }
  | {
      type: "remote_server";
      url: string;
    };

export interface SetupConfig {
  adb_source?: AdbSource | null;
  github_token?: string | null;
}

export interface UseTauriAPI {
  checkDeviceConnection: () => Promise<DeviceInfo>;
  listInstalledPackages: () => Promise<PackageInfo[]>;
  installRepositories: (repos: string[]) => Promise<string>;
  getAvailableRepositories: () => Promise<RepositoryInfo[]>;
  cancelInstallation: () => Promise<void>;
  loadSetupConfig: () => Promise<SetupConfig>;
  setAdbKeyFromFile: (path: string) => Promise<SetupConfig>;
  setAdbKeyFromBytes: (filename: string, data: number[]) => Promise<SetupConfig>;
  setAdbKeyRemote: (url: string) => Promise<SetupConfig>;
  clearAdbKey: () => Promise<SetupConfig>;
  setGithubToken: (token?: string | null) => Promise<SetupConfig>;
}

export const useTauri = (): UseTauriAPI => {
  return {
    checkDeviceConnection: () => invoke("check_device_connection"),
    listInstalledPackages: () => invoke("list_installed_packages"),
    installRepositories: (repos: string[]) =>
      invoke("install_repositories", { repos }),
    getAvailableRepositories: () => invoke("get_available_repositories"),
    cancelInstallation: () => invoke("cancel_installation"),
    loadSetupConfig: () => invoke("load_setup_config"),
    setAdbKeyFromFile: (path: string) =>
      invoke("set_adb_key_from_file", { path }),
    setAdbKeyFromBytes: (filename: string, data: number[]) =>
      invoke("set_adb_key_from_bytes", { filename, data }),
    setAdbKeyRemote: (url: string) =>
      invoke("set_adb_key_remote", { url }),
    clearAdbKey: () => invoke("clear_adb_key"),
    setGithubToken: (token?: string | null) =>
      invoke("set_github_token", { token: token ?? null }),
  };
};

// Event listening for real-time updates
export const useInstallationProgress = (
  callback: (message: string) => void
) => {
  useEffect(() => {
    const unlisten = listen("installation_progress", (event) => {
      callback(event.payload as string);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [callback]);
};
