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

export interface UseTauriAPI {
  checkDeviceConnection: () => Promise<DeviceInfo>;
  listInstalledPackages: () => Promise<PackageInfo[]>;
  installRepositories: (repos: string[]) => Promise<string>;
  getAvailableRepositories: () => Promise<RepositoryInfo[]>;
  cancelInstallation: () => Promise<void>;
}

export const useTauri = (): UseTauriAPI => {
  return {
    checkDeviceConnection: () => invoke("check_device_connection"),
    listInstalledPackages: () => invoke("list_installed_packages"),
    installRepositories: (repos: string[]) =>
      invoke("install_repositories", { repos }),
    getAvailableRepositories: () => invoke("get_available_repositories"),
    cancelInstallation: () => invoke("cancel_installation"),
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
