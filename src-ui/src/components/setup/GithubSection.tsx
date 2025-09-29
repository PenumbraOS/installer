import React from "react";
import { Stack, Text, Alert, PasswordInput, Badge } from "@mantine/core";

export interface GithubSectionProps {
  introText?: string;
  footer?: React.ReactNode;
  error?: string | null;
  tokenValue: string;
  onTokenChange: (value: string) => void;
  saving: boolean;
  storedTokenPreview?: string | null;
}

export const GithubSection: React.FC<GithubSectionProps> = ({
  introText,
  footer,
  error,
  tokenValue,
  onTokenChange,
  saving,
  storedTokenPreview,
}) => {
  return (
    <Stack gap="lg" mt={introText ? "sm" : undefined}>
      {introText && <Text size="sm">{introText}</Text>}
      {error && <Alert color="red">{error}</Alert>}
      <PasswordInput
        label="GitHub token"
        placeholder="ghp_xxxxxxxxx"
        value={tokenValue}
        onChange={(event) => onTokenChange(event.currentTarget.value)}
        autoComplete="off"
        disabled={saving}
      />
      {storedTokenPreview && (
        <Badge color="grape" variant="light" w="fit-content">
          Stored ({storedTokenPreview})
        </Badge>
      )}
      {footer}
    </Stack>
  );
};
