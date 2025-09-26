import React, { useState, useEffect } from "react";
import { Paper, Group, Title, Button, Text, Alert, Stack, Checkbox, ScrollArea } from "@mantine/core";
import { IconDownload, IconPlayerStop, IconAlertCircle, IconGitBranch } from "@tabler/icons-react";
import { useTauri, RepositoryInfo } from "../hooks/useTauri";

interface RepositorySelectorProps {
  deviceConnected: boolean;
  installing: boolean;
  onInstall: (selectedRepos: string[]) => void;
  onCancel: () => void;
}

export const RepositorySelector: React.FC<RepositorySelectorProps> = ({
  deviceConnected,
  installing,
  onInstall,
  onCancel,
}) => {
  const [repositories, setRepositories] = useState<RepositoryInfo[]>([]);
  const [selectedRepos, setSelectedRepos] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useTauri();

  const loadRepositories = async () => {
    setLoading(true);
    setError(null);
    try {
      const repos = await api.getAvailableRepositories();
      setRepositories(repos);
    } catch (err) {
      console.error("Failed to load repositories:", err);
      setError(
        err instanceof Error ? err.message : "Failed to load repositories"
      );
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadRepositories();
  }, []);

  const handleRepoToggle = (repoName: string) => {
    setSelectedRepos((prev) =>
      prev.includes(repoName)
        ? prev.filter((r) => r !== repoName)
        : [...prev, repoName]
    );
  };

  const handleSelectAll = () => {
    if (selectedRepos.length === repositories.length) {
      setSelectedRepos([]);
    } else {
      setSelectedRepos(repositories.map((r) => r.name));
    }
  };

  const handleInstallSelected = () => {
    onInstall(selectedRepos);
  };

  const handleInstallAll = () => {
    onInstall([]);
  };

  const canInstall = deviceConnected && !installing;

  return (
    <Paper withBorder p="md">
      <Title order={3} size="h4" mb="sm">
        Available Repositories
      </Title>

      {error && (
        <Alert
          icon={<IconAlertCircle size={16} />}
          title="Error"
          color="red"
          variant="light"
          mb="sm"
        >
          {error}
        </Alert>
      )}

      {loading ? (
        <Text c="dimmed" ta="center" py="xl">
          Loading repositories...
        </Text>
      ) : (
        <Stack gap="md">
          <ScrollArea h={150}>
            {repositories.length === 0 ? (
              <Text c="dimmed" ta="center" py="xl">
                No repositories available
              </Text>
            ) : (
              <Stack gap="xs">
                {repositories.map((repo) => (
                  <Checkbox
                    key={repo.name}
                    checked={selectedRepos.includes(repo.name)}
                    onChange={() => canInstall && handleRepoToggle(repo.name)}
                    disabled={!canInstall}
                    label={
                      <div>
                        <Group gap="xs">
                          <IconGitBranch size={14} />
                          <Text size="sm" fw={500}>
                            {repo.name}
                          </Text>
                        </Group>
                        <Text size="xs" c="dimmed">
                          {repo.owner}/{repo.repo}
                        </Text>
                      </div>
                    }
                  />
                ))}
              </Stack>
            )}
          </ScrollArea>

          {repositories.length > 0 && (
            <Stack gap="sm">
              <Button
                onClick={handleSelectAll}
                disabled={!canInstall}
                variant="outline"
                size="sm"
              >
                {selectedRepos.length === repositories.length
                  ? "Deselect All"
                  : "Select All"}
              </Button>

              <Group gap="xs">
                <Button
                  onClick={handleInstallSelected}
                  disabled={!canInstall || selectedRepos.length === 0}
                  leftSection={<IconDownload size={16} />}
                  color="green"
                >
                  Install Selected ({selectedRepos.length})
                </Button>

                <Button
                  onClick={handleInstallAll}
                  disabled={!canInstall}
                  leftSection={<IconDownload size={16} />}
                  variant="light"
                >
                  Install All
                </Button>

                {installing && (
                  <Button
                    onClick={onCancel}
                    leftSection={<IconPlayerStop size={16} />}
                    color="red"
                  >
                    Cancel
                  </Button>
                )}
              </Group>
            </Stack>
          )}

          {!deviceConnected && (
            <Alert color="orange" variant="light">
              Connect a device to enable installation
            </Alert>
          )}
        </Stack>
      )}
    </Paper>
  );
};
