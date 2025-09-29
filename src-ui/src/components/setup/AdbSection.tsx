import React from "react";
import {
  Stack,
  Text,
  Tabs,
  FileButton,
  Button,
  Alert,
  Code,
  TextInput,
} from "@mantine/core";
import { IconServer, IconUpload } from "@tabler/icons-react";
import { AdbSource } from "../../hooks/useTauri";

export interface AdbSectionProps {
  introText?: string;
  footer?: React.ReactNode;
  error?: string | null;
  adbMode: "file" | "remote";
  onModeChange: (mode: "file" | "remote") => void;
  saving: boolean;
  pendingFileName: string | null;
  pendingFileSource: string | null;
  adbSource?: AdbSource | null;
  remoteUrl: string;
  onRemoteUrlChange: (value: string) => void;
  remoteUrlNeedsSave: boolean;
  onFilePicked: (file: File | null) => void;
  fileInputResetRef: React.MutableRefObject<(() => void) | null>;
}

const isLocalSource = (
  source?: AdbSource | null
): source is Extract<AdbSource, { type: "local_copy" }> =>
  source?.type === "local_copy";

const isRemoteSource = (
  source?: AdbSource | null
): source is Extract<AdbSource, { type: "remote_server" }> =>
  source?.type === "remote_server";

export const AdbSection: React.FC<AdbSectionProps> = ({
  introText,
  footer,
  error,
  adbMode,
  onModeChange,
  saving,
  pendingFileName,
  pendingFileSource,
  adbSource,
  remoteUrl,
  onRemoteUrlChange,
  remoteUrlNeedsSave,
  onFilePicked,
  fileInputResetRef,
}) => {
  return (
    <Stack gap="lg" mt={introText ? "sm" : undefined}>
      {introText && <Text size="sm">{introText}</Text>}
      {error && <Alert color="red">{error}</Alert>}
      <Tabs value={adbMode} onChange={(value) => onModeChange((value as "file" | "remote") ?? "file")} keepMounted={false}>
        <Tabs.List>
          <Tabs.Tab value="file" leftSection={<IconUpload size={16} />}>
            Use a key file
          </Tabs.Tab>
          <Tabs.Tab value="remote" leftSection={<IconServer size={16} />}>
            Use signing server
          </Tabs.Tab>
        </Tabs.List>
        <Tabs.Panel value="file" pt="md">
          <Stack gap="md">
            <FileButton
              onChange={onFilePicked}
              resetRef={fileInputResetRef}
              disabled={saving}
              accept="*"
            >
              {(props) => (
                <Button {...props} loading={saving} leftSection={<IconUpload size={16} />}
                >
                  Choose file
                </Button>
              )}
            </FileButton>
            {pendingFileName && (
              <Alert color="blue" variant="light">
                Ready to save <strong>{pendingFileName}</strong>.
                {pendingFileSource && (
                  <Text size="xs" mt="xs">
                    <Code>{pendingFileSource}</Code>
                  </Text>
                )}
              </Alert>
            )}
            {isLocalSource(adbSource) && (
              <Alert color="teal" variant="light">
                Using <strong>{adbSource.original_filename}</strong> stored at <Code>{adbSource.stored_path}</Code>
              </Alert>
            )}
          </Stack>
        </Tabs.Panel>
        <Tabs.Panel value="remote" pt="md">
          <Stack gap="md">
            <Text size="sm" c="dimmed">
              Provide your signing server URL. We'll save it when you apply changes.
            </Text>
            <TextInput
              label="Signing server URL"
              placeholder="https://signing.example.com/api"
              value={remoteUrl}
              onChange={(event) => onRemoteUrlChange(event.currentTarget.value)}
              disabled={saving}
            />
            {remoteUrlNeedsSave && (
              <Alert color="blue" variant="light">
                Ready to save the updated URL.
              </Alert>
            )}
            {isRemoteSource(adbSource) && (
              <Alert color="teal" variant="light">
                Using signing server at <Code>{adbSource.url}</Code>
              </Alert>
            )}
          </Stack>
        </Tabs.Panel>
      </Tabs>
      {footer}
    </Stack>
  );
};
