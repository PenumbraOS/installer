import { Alert, Button, List, Stack, Text, Title } from "@mantine/core";
import { IconAlertCircle } from "@tabler/icons-react";

interface NoDeviceProps {
  onReload: () => void;
  loading?: boolean;
  message?: string;
}

export const NoDevice: React.FC<NoDeviceProps> = ({
  onReload,
  loading = false,
  message,
}) => {
  return (
    <Stack gap="md">
      <Title order={2}>Connect Your Ai Pin</Title>
      <Text size="sm" c="dimmed">
        Plug the device into this computer using USB and enable USB debugging.
        If the device prompts for authorization, accept it to continue.
      </Text>
      {message && (
        <Alert color="red" icon={<IconAlertCircle size={16} />}>
          {message}
        </Alert>
      )}
      <List size="sm" spacing="xs">
        <List.Item>Make sure Developer Options and USB debugging are enabled.</List.Item>
        <List.Item>Use the supplied cable or a high-quality USB data cable.</List.Item>
        <List.Item>Keep the Ai Pin awake during the connection attempt.</List.Item>
      </List>
      <Button onClick={onReload} loading={loading} maw={200}>
        Try Again
      </Button>
    </Stack>
  );
};
