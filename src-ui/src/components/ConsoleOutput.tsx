import React, { useState, useEffect, useRef } from "react";
import { Paper, Group, Title, Button, Text, ScrollArea, Code, ActionIcon } from "@mantine/core";
import { IconTrash, IconDownload, IconTerminal } from "@tabler/icons-react";
import { useInstallationProgress } from "../hooks/useTauri";

interface ConsoleOutputProps {
  installing: boolean;
}

export const ConsoleOutput: React.FC<ConsoleOutputProps> = ({ installing }) => {
  const [output, setOutput] = useState<string[]>([]);
  const outputRef = useRef<HTMLDivElement>(null);

  const addMessage = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setOutput((prev) => [...prev, `[${timestamp}] ${message}`]);
  };

  useInstallationProgress(addMessage);

  const clearOutput = () => {
    setOutput([]);
  };

  const exportLogs = () => {
    const logsText = output.join("\n");
    const blob = new Blob([logsText], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `penumbra-installer-logs-${Date.now()}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (outputRef.current) {
      outputRef.current.scrollTop = outputRef.current.scrollHeight;
    }
  }, [output]);

  // Add initial message when installation starts
  useEffect(() => {
    if (installing && output.length === 0) {
      addMessage("Ready to start installation...");
    }
  }, [installing]);

  return (
    <Paper withBorder p="md">
      <Group justify="space-between" mb="sm">
        <Title order={3} size="h4">
          Installation Progress
        </Title>
        <Group gap="xs">
          <ActionIcon
            onClick={clearOutput}
            disabled={output.length === 0}
            color="red"
            variant="filled"
            size="sm"
            title="Clear output"
          >
            <IconTrash size={14} />
          </ActionIcon>
          <ActionIcon
            onClick={exportLogs}
            disabled={output.length === 0}
            color="blue"
            variant="filled"
            size="sm"
            title="Export logs"
          >
            <IconDownload size={14} />
          </ActionIcon>
        </Group>
      </Group>

      <ScrollArea
        h={200}
        viewportRef={outputRef}
        style={{
          backgroundColor: "#1a1a1a",
          borderRadius: "4px",
          border: "1px solid #333",
        }}
        p="sm"
      >
        {output.length === 0 ? (
          <Text c="dimmed" fs="italic" ta="center" py="xl">
            <Group gap="xs" justify="center">
              <IconTerminal size={16} />
              Installation output will appear here...
            </Group>
          </Text>
        ) : (
          <>
            {output.map((line, index) => (
              <Code
                key={index}
                block
                c="gray.0"
                bg="transparent"
                style={{
                  marginBottom: "2px",
                  whiteSpace: "pre-wrap",
                  wordBreak: "break-word",
                  fontFamily: 'Monaco, Consolas, "Courier New", monospace',
                  fontSize: "13px",
                  lineHeight: "1.4",
                }}
              >
                {line}
              </Code>
            ))}
            {installing && (
              <Text
                c="green"
                fw={700}
                mt="xs"
                style={{
                  fontFamily: 'Monaco, Consolas, "Courier New", monospace',
                  fontSize: "13px",
                }}
              >
                ● Installation in progress...
              </Text>
            )}
          </>
        )}
      </ScrollArea>
    </Paper>
  );
};
