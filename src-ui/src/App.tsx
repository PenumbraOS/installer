import React, { useState } from "react";
import { DeviceStatus } from "./components/DeviceStatus";
import { PackageList } from "./components/PackageList";
import { RepositorySelector } from "./components/RepositorySelector";
import { ConsoleOutput } from "./components/ConsoleOutput";
import { useTauri } from "./hooks/useTauri";

export const App: React.FC<{}> = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const [installing, setInstalling] = useState(false);
  const api = useTauri();

  const handleInstall = async (selectedRepos: string[]) => {
    setInstalling(true);
    try {
      await api.installRepositories(selectedRepos);
    } catch (error) {
      console.error("Installation failed:", error);
    } finally {
      setInstalling(false);
    }
  };

  const handleCancel = async () => {
    try {
      await api.cancelInstallation();
    } catch (error) {
      console.error("Failed to cancel installation:", error);
    } finally {
      setInstalling(false);
    }
  };

  return (
    <div
      style={{
        minHeight: "100vh",
        backgroundColor: "#f5f5f5",
        padding: "16px",
      }}
    >
      <div
        style={{
          maxWidth: "800px",
          margin: "0 auto",
        }}
      >
        <header
          style={{
            marginBottom: "24px",
            textAlign: "center",
          }}
        >
          <h1
            style={{
              margin: 0,
              fontSize: "28px",
              fontWeight: "700",
              color: "#333",
              marginBottom: "8px",
            }}
          >
            Penumbra Installer
          </h1>
          <p
            style={{
              margin: 0,
              fontSize: "16px",
              color: "#666",
            }}
          >
            Install and manage Android packages via ADB
          </p>
        </header>

        <DeviceStatus onDeviceChange={setDeviceConnected} />

        <PackageList deviceConnected={deviceConnected} />

        <RepositorySelector
          deviceConnected={deviceConnected}
          installing={installing}
          onInstall={handleInstall}
          onCancel={handleCancel}
        />

        <ConsoleOutput installing={installing} />
      </div>
    </div>
  );
};
