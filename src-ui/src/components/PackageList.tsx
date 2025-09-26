import React, { useState, useEffect } from "react";
import {
  Paper,
  Group,
  Title,
  Button,
  Text,
  Alert,
  Stack,
  Badge,
  ScrollArea,
} from "@mantine/core";
import { IconRefresh, IconPackage, IconAlertCircle } from "@tabler/icons-react";
import { useTauri, PackageInfo } from "../hooks/useTauri";

interface PackageListProps {
  deviceConnected: boolean;
}

export const PackageList: React.FC<PackageListProps> = ({
  deviceConnected,
}) => {
  const [packages, setPackages] = useState<PackageInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const api = useTauri();

  const loadPackages = async () => {
    if (!deviceConnected) {
      setPackages([]);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const packageList = await api.listInstalledPackages();
      setPackages(packageList);
    } catch (err) {
      console.error("Failed to load packages:", err);
      setError(err instanceof Error ? err.message : "Failed to load packages");
      setPackages([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadPackages();
  }, [deviceConnected]);

  return (
    <Paper withBorder p="md">
      <Group justify="space-between" mb="sm">
        <Title order={3} size="h4">
          Installed Packages
        </Title>
        <Button
          onClick={loadPackages}
          disabled={loading || !deviceConnected}
          loading={loading}
          leftSection={<IconRefresh size={16} />}
          variant="light"
          size="sm"
        >
          {loading ? "Loading..." : "Refresh"}
        </Button>
      </Group>

      {!deviceConnected && (
        <Alert color="blue" variant="light">
          Connect a device to view installed packages
        </Alert>
      )}

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

      {deviceConnected && !loading && !error && (
        <ScrollArea h={200}>
          {packages.length === 0 ? (
            <Text c="dimmed" ta="center" py="xl">
              No relevant packages found
            </Text>
          ) : (
            <Stack gap="xs">
              {packages.map((pkg, index) => (
                <Paper key={index} withBorder p="sm">
                  <Group justify="space-between" align="center">
                    <Group gap="xs">
                      <IconPackage size={16} />
                      <Text ff="monospace" size="sm" fw={500}>
                        {pkg.package_name}
                      </Text>
                    </Group>
                    {pkg.version && <Badge size="sm">{pkg.version}</Badge>}
                  </Group>
                </Paper>
              ))}
            </Stack>
          )}
        </ScrollArea>
      )}
    </Paper>
  );
};
