import { Button, Stack, Title } from "@mantine/core";

export const NoDevice: React.FC<{
  onReload: () => void;
}> = ({ onReload }) => {
  return (
    <Stack align="center">
      <Title order={2}>No Ai Pin Connected</Title>
      <Button onClick={onReload}>Try Again</Button>
    </Stack>
  );
};
