import React, { useState, useEffect } from "react";
import { Paper, Group, Title, Button, Text, Alert, Loader, Badge } from "@mantine/core";
import { IconRefresh, IconDeviceMobile, IconAlertCircle } from "@tabler/icons-react";
import { useTauri, DeviceInfo } from "../hooks/useTauri";

interface DeviceStatusProps {
  onDeviceChange?: (connected: boolean) => void;
}

export const DeviceStatus: React.FC<DeviceStatusProps> = ({
  onDeviceChange,
}) => {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const api = useTauri();

  const checkDevice = async () => {
    setLoading(true);
    try {
      const info = await api.checkDeviceConnection();
      setDeviceInfo(info);
      onDeviceChange?.(info.connected);
    } catch (error) {
      console.error("Failed to check device:", error);
      setDeviceInfo({
        connected: false,
        device_count: 0,
        error_message: "Failed to check device connection",
      });
      onDeviceChange?.(false);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    checkDevice();
  }, []);

  const getStatusColor = () => {
    if (!deviceInfo) return "gray";
    if (deviceInfo.connected) return "green";
    return "red";
  };

  const getStatusText = () => {
    if (loading) return "Checking...";
    if (!deviceInfo) return "Unknown";
    if (deviceInfo.connected) return "Connected - Device ready for installation";
    if (deviceInfo.device_count === 0) return "No device connected";
    if (deviceInfo.device_count > 1) return "Multiple devices connected";
    return "Device not ready";
  };

  return (
    <Paper withBorder p="md">
      <Group justify="space-between" mb="sm">
        <Title order={3} size="h4">
          Device Status
        </Title>
        <Button
          onClick={checkDevice}
          loading={loading}
          leftSection={<IconRefresh size={16} />}
          variant="light"
          size="sm"
        >
          {loading ? "Checking..." : "Refresh"}
        </Button>
      </Group>

      <Group gap="sm" mb="sm">
        <IconDeviceMobile size={20} />
        <Badge color={getStatusColor()} variant="filled">
          {getStatusText()}
        </Badge>
      </Group>

      {deviceInfo?.error_message && (
        <Alert
          icon={<IconAlertCircle size={16} />}
          title="Connection Error"
          color="red"
          variant="light"
        >
          {deviceInfo.error_message}
        </Alert>
      )}
    </Paper>
  );
};
