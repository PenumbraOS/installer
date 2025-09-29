import React from "react";
import {
  MantineProvider,
  AppShell,
  Container,
  Stack,
  Title,
  Text,
} from "@mantine/core";
import "@mantine/core/styles.css";
import { SetupWizard } from "./components/setup/SetupWizard";

export const App: React.FC = () => {
  return (
    <MantineProvider defaultColorScheme="dark">
      <AppShell padding="md">
        <Container size="md">
          <Stack gap="xl">
            <Stack gap="xs" align="center">
              <span />
              <Title order={1}>PenumbraOS Installer</Title>
              <Text size="sm" c="dimmed">
                Follow the wizard to provide the required credentials and connect your Ai Pin.
              </Text>
            </Stack>

            <SetupWizard />
          </Stack>
        </Container>
      </AppShell>
    </MantineProvider>
  );
};
