import React, { useState } from "react";
import {
  MantineProvider,
  AppShell,
  Container,
  Title,
  Stack,
} from "@mantine/core";
import { DeviceStatus } from "./components/DeviceStatus";
import { PackageList } from "./components/PackageList";
import { RepositorySelector } from "./components/RepositorySelector";
import { ConsoleOutput } from "./components/ConsoleOutput";
import { useTauri } from "./hooks/useTauri";
import "@mantine/core/styles.css";

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
    <MantineProvider defaultColorScheme="dark">
      <AppShell padding="md">
        <Container size="md">
          <Stack gap="xl">
            <Stack gap="xs" align="center">
              <span />
              <Title order={1}>PenumbraOS Installer</Title>
            </Stack>

            <DeviceStatus onDeviceChange={setDeviceConnected} />
            <PackageList deviceConnected={deviceConnected} />
            <RepositorySelector
              deviceConnected={deviceConnected}
              installing={installing}
              onInstall={handleInstall}
              onCancel={handleCancel}
            />
            <ConsoleOutput installing={installing} />
          </Stack>
        </Container>
      </AppShell>
    </MantineProvider>
  );
};
